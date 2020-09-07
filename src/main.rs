use rmm::{
    KILOBYTE,
    MEGABYTE,
    GIGABYTE,
    TERABYTE,
    Arch,
    EmulateArch,
    MemoryArea,
    PageEntry,
    PageTable,
    PhysicalAddress,
    VirtualAddress,
};

use core::marker::PhantomData;

pub fn format_size(size: usize) -> String {
    if size >= 2 * TERABYTE {
        format!("{} TB", size / TERABYTE)
    } else if size >= 2 * GIGABYTE {
        format!("{} GB", size / GIGABYTE)
    } else if size >= 2 * MEGABYTE {
        format!("{} MB", size / MEGABYTE)
    } else if size >= 2 * KILOBYTE {
        format!("{} KB", size / KILOBYTE)
    } else {
        format!("{} B", size)
    }
}

unsafe fn dump_tables<A: Arch>(table: PageTable<A>) {
    let level = table.level();
    for i in 0..A::PAGE_ENTRIES {
        if level == 0 {
            if let Some(entry) = table.entry(i) {
                if entry.present() {
                    let base = table.entry_base(i).unwrap();
                    println!("0x{:X}: 0x{:X}", base.data(), entry.address().data());
                }
            }
        } else {
            if let Some(next) = table.next(i) {
                dump_tables(next);
            }
        }
    }
}

pub struct BumpAllocator<A> {
    areas: &'static [MemoryArea],
    offset: usize,
    phantom: PhantomData<A>,
}

impl<A: Arch> BumpAllocator<A> {
    pub fn new(areas: &'static [MemoryArea]) -> Self {
        Self {
            areas,
            offset: 0,
            phantom: PhantomData,
        }
    }

    pub fn allocate(&mut self) -> Option<PhysicalAddress> {
        let mut offset = self.offset;
        for area in self.areas.iter() {
            if offset < area.size {
                self.offset += A::PAGE_SIZE;
                return Some(area.base.add(offset));
            }
            offset -= area.size;
        }
        None
    }
}

pub struct Mapper<A> {
    table_addr: PhysicalAddress,
    allocator: BumpAllocator<A>,
}

impl<A: Arch> Mapper<A> {
    pub unsafe fn new(mut allocator: BumpAllocator<A>) -> Option<Self> {
        let table_addr = allocator.allocate()?;
        Some(Self {
            table_addr,
            allocator,
        })
    }

    pub unsafe fn map(&mut self, virt: VirtualAddress, entry: PageEntry<A>) -> Option<()> {
        let mut table = PageTable::new(
            VirtualAddress::new(0),
            self.table_addr,
            A::PAGE_LEVELS - 1
        );
        loop {
            let i = table.index_of(virt)?;
            if table.level() == 0 {
                //TODO: check for overwriting entry
                table.set_entry(i, entry);
                return Some(());
            } else {
                let next_opt = table.next(i);
                let next = match next_opt {
                    Some(some) => some,
                    None => {
                        let phys = self.allocator.allocate()?;
                        //TODO: correct flags?
                        table.set_entry(i, PageEntry::new(phys.data() | A::ENTRY_FLAG_WRITABLE | A::ENTRY_FLAG_PRESENT));
                        table.next(i)?
                    }
                };
                table = next;
            }
        }
    }
}

unsafe fn new_tables<A: Arch>(areas: &'static [MemoryArea]) {
    // First, calculate how much memory we have
    let mut size = 0;
    for area in areas.iter() {
        size += area.size;
    }

    println!("Memory: {}", format_size(size));

    // Create a basic allocator for the first pages
    let allocator = BumpAllocator::<A>::new(areas);

    // Map all physical areas at PHYS_OFFSET
    let mut mapper = Mapper::new(allocator).expect("failed to create Mapper");
    for area in areas.iter() {
        for i in 0..area.size / A::PAGE_SIZE {
            let phys = area.base.add(i * A::PAGE_SIZE);
            let virt = A::phys_to_virt(phys);
            mapper.map(
                virt,
                PageEntry::new(phys.data() | A::ENTRY_FLAG_WRITABLE | A::ENTRY_FLAG_PRESENT)
            ).expect("failed to map frame");
        }
    }

    A::set_table(mapper.table_addr);

    let used = mapper.allocator.offset;
    println!("Used: {}", format_size(used));
}

unsafe fn inner<A: Arch>() {
    let areas = A::init();

    // Debug table
    //dump_tables(PageTable::<A>::top());

    new_tables::<A>(areas);

    //dump_tables(PageTable::<A>::top());


    for i in &[1, 2, 4, 8, 16, 32] {
        let phys = PhysicalAddress::new(i * MEGABYTE);
        let virt = A::phys_to_virt(phys);

        // Test read
        println!("0x{:X} (0x{:X}) = 0x{:X}", virt.data(), phys.data(), A::read::<u8>(virt));

        // Test write
        A::write::<u8>(virt, 0x5A);

        // Test read
        println!("0x{:X} (0x{:X}) = 0x{:X}", virt.data(), phys.data(), A::read::<u8>(virt));
    }
}

fn main() {
    unsafe {
        inner::<EmulateArch>();
    }
}
