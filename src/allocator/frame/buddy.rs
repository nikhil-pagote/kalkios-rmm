use core::{
    marker::PhantomData,
    mem,
};

use crate::{
    Arch,
    BumpAllocator,
    FrameAllocator,
    FrameCount,
    PhysicalAddress,
    VirtualAddress,
};

#[derive(Clone, Copy, Debug)]
#[repr(packed)]
struct BuddyEntry {
    base: PhysicalAddress,
    size: usize,
    map: PhysicalAddress,
}

impl BuddyEntry {
    pub fn empty() -> Self {
        Self {
            base: PhysicalAddress::new(0),
            size: 0,
            map: PhysicalAddress::new(0),
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(packed)]
struct BuddyMapFooter {
    next: PhysicalAddress,
    //TODO: index of last known free bit
}

pub struct BuddyAllocator<A> {
    table_virt: VirtualAddress,
    phantom: PhantomData<A>,
}

impl<A: Arch> BuddyAllocator<A> {
    const BUDDY_ENTRIES: usize = A::PAGE_SIZE / mem::size_of::<BuddyEntry>();
    const MAP_PAGE_BYTES: usize = (A::PAGE_SIZE - mem::size_of::<BuddyMapFooter>());
    const MAP_PAGE_BITS: usize = Self::MAP_PAGE_BYTES * 8;

    pub unsafe fn new(mut bump_allocator: BumpAllocator<A>) -> Option<Self> {
        // Allocate buddy table
        let table_phys = bump_allocator.allocate_one()?;
        let table_virt = A::phys_to_virt(table_phys);
        for i in 0 .. (A::PAGE_SIZE / mem::size_of::<BuddyEntry>()) {
            let virt = table_virt.add(i * mem::size_of::<BuddyEntry>());
            A::write(virt, BuddyEntry::empty());
        }

        let mut allocator = Self {
            table_virt,
            phantom: PhantomData,
        };

        // Add areas to buddy table, combining areas when possible
        for area in bump_allocator.areas().iter() {
            for i in 0 .. (A::PAGE_SIZE / mem::size_of::<BuddyEntry>()) {
                let virt = table_virt.add(i * mem::size_of::<BuddyEntry>());
                let mut entry = A::read::<BuddyEntry>(virt);
                let inserted = if area.base.add(area.size) == entry.base {
                    // Combine entry at start
                    entry.base = area.base;
                    entry.size += area.size;
                    true
                } else if area.base == entry.base.add(entry.size) {
                    // Combine entry at end
                    entry.size += area.size;
                    true
                } else if entry.size == 0 {
                    // Create new entry
                    entry.base = area.base;
                    entry.size = area.size;
                    true
                } else {
                    false
                };
                if inserted {
                    A::write(virt, entry);
                    break;
                }
            }
        }

        //TODO: sort areas?

        // Allocate buddy maps
        for i in 0 .. Self::BUDDY_ENTRIES {
            let virt = table_virt.add(i * mem::size_of::<BuddyEntry>());
            let mut entry = A::read::<BuddyEntry>(virt);
            if entry.size > 0 {
                let pages = entry.size / A::PAGE_SIZE;
                let map_pages = (pages + (Self::MAP_PAGE_BITS - 1)) / Self::MAP_PAGE_BITS;
                for _ in 0 .. map_pages {
                    let map_phys = bump_allocator.allocate_one()?;
                    let map_virt = A::phys_to_virt(map_phys);
                    A::write(map_virt.add(Self::MAP_PAGE_BYTES), BuddyMapFooter {
                        next: entry.map,
                    });
                    entry.map = map_phys;
                }

                A::write(virt, entry);
            }
        }

        // Mark unused areas as free
        let mut area_offset = bump_allocator.offset();
        for area in bump_allocator.areas().iter() {
            if area_offset < area.size {
                let area_base = area.base.add(area_offset);
                let area_size = area.size - area_offset;
                allocator.free(area_base, FrameCount::new(area_size / A::PAGE_SIZE));
                area_offset = 0;
            } else {
                area_offset -= area.size;
            }
        }

        Some(allocator)
    }
}

impl<A: Arch> FrameAllocator for BuddyAllocator<A> {
    unsafe fn allocate(&mut self, count: FrameCount) -> Option<PhysicalAddress> {
        if self.table_virt.data() == 0 {
            return None;
        }

        for entry_i in 0 .. Self::BUDDY_ENTRIES {
            let virt = self.table_virt.add(entry_i * mem::size_of::<BuddyEntry>());
            let entry = A::read::<BuddyEntry>(virt);

            let mut start_map_phys = entry.map;
            let mut start_offset = 0;
            let mut start_i = 0;
            let mut start_bit = 0;
            let mut start_page_phys = PhysicalAddress::new(0);
            let mut found = 0;

            //TODO: improve performance
            let mut map_phys = start_map_phys;
            let mut offset = start_offset;
            'find: while map_phys.data() != 0 {
                let map_virt = A::phys_to_virt(map_phys);
                for i in 0 .. Self::MAP_PAGE_BYTES {
                    let map_byte_virt = map_virt.add(i);
                    let value: u8 = A::read(map_byte_virt);
                    if (value & u8::MAX) != 0 {
                        for bit in 0..8 {
                            if (value & (1 << bit)) != 0 {
                                if found == 0 {
                                    start_map_phys = map_phys;
                                    start_offset = offset;
                                    start_i = i;
                                    start_bit = bit;
                                    start_page_phys = entry.base.add(offset + bit * A::PAGE_SIZE);
                                }
                                found += 1;
                                if found == count.data() {
                                    break 'find;
                                }
                            } else {
                                found = 0;
                            }
                        }
                    } else {
                        found = 0;
                    }
                    offset += A::PAGE_SIZE * 8;
                }

                let footer = A::read::<BuddyMapFooter>(map_virt.add(Self::MAP_PAGE_BYTES));
                map_phys = footer.next;
            }

            if found == count.data() {
                map_phys = start_map_phys;
                offset = start_offset;
                found = 0;
                while map_phys.data() != 0 {
                    let map_virt = A::phys_to_virt(map_phys);
                    for i in start_i .. Self::MAP_PAGE_BYTES {
                        let map_byte_virt = map_virt.add(i);
                        let mut value: u8 = A::read(map_byte_virt);
                        if (value & u8::MAX) != 0 {
                            for bit in start_bit..8 {
                                if (value & (1 << bit)) != 0 {
                                    value &= !(1 << bit);
                                    A::write(map_byte_virt, value);

                                    let page_phys = entry.base.add(offset + bit * A::PAGE_SIZE);
                                    let page_virt = A::phys_to_virt(page_phys);
                                    A::write_bytes(page_virt, 0, A::PAGE_SIZE);

                                    found += 1;
                                    if found == count.data() {
                                        return Some(start_page_phys);
                                    }
                                } else {
                                    panic!("check your logic, bit was set");
                                }
                            }
                            start_bit = 0;
                        } else {
                            panic!("check your logic, byte was set");
                        }
                        offset += A::PAGE_SIZE * 8;
                    }
                    start_i = 0;

                    let footer = A::read::<BuddyMapFooter>(map_virt.add(Self::MAP_PAGE_BYTES));
                    map_phys = footer.next;
                }
            }
        }
        None
    }

    unsafe fn free(&mut self, base: PhysicalAddress, count: FrameCount) {
        if self.table_virt.data() == 0 {
            return;
        }

        let size = count.data() * A::PAGE_SIZE;
        for i in 0 .. Self::BUDDY_ENTRIES {
            let virt = self.table_virt.add(i * mem::size_of::<BuddyEntry>());
            let entry = A::read::<BuddyEntry>(virt);
            if base >= entry.base && base.add(size) <= entry.base.add(entry.size) {
                //TODO: Correct logic
                for page in 0 .. count.data() {
                    let page_base = base.add(page * A::PAGE_SIZE);
                    let index = (page_base.data() - entry.base.data()) / A::PAGE_SIZE;
                    let mut map_page = index / Self::MAP_PAGE_BITS;
                    let map_bit = index % Self::MAP_PAGE_BITS;

                    //TODO: improve performance
                    let mut map_phys = entry.map;
                    loop {
                        if map_phys.data() == 0 { unimplemented!("map_phys.data() == 0") }
                        let map_virt = A::phys_to_virt(map_phys);
                        if map_page == 0 {
                            let map_byte_virt = map_virt.add(map_bit / 8);
                            let mut value: u8 = A::read(map_byte_virt);
                            value |= 1 << (map_bit % 8);
                            A::write(map_byte_virt, value);
                            break;
                        } else {
                            let footer = A::read::<BuddyMapFooter>(map_virt.add(Self::MAP_PAGE_BYTES));
                            map_phys = footer.next;
                            map_page -= 1;
                        }
                    }
                }
            }
        }
    }
}
