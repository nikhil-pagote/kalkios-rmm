use core::{
    mem,
    ptr,
};
use std::collections::BTreeMap;

//TODO: should this be a constant?
pub const PAGE_SHIFT: usize = 12;
pub const PAGE_SIZE: usize = 1 << PAGE_SHIFT;
pub const PAGE_OFFSET_MASK: usize = PAGE_SIZE - 1;
pub const PAGE_ADDRESS_MASK: usize = !PAGE_OFFSET_MASK;
pub const PAGE_ENTRY_SIZE: usize = mem::size_of::<usize>();
pub const PAGE_ENTRIES: usize = PAGE_SIZE / PAGE_ENTRY_SIZE;
pub const PAGE_LEVELS: usize = 4;

// Physical memory address
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct PhysicalAddress(usize);

impl PhysicalAddress {
    pub const fn new(address: usize) -> Self {
        Self(address)
    }

    pub const fn aligned(&self) -> bool {
        self.0 & PAGE_OFFSET_MASK == 0
    }
}

// Physical memory frame
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct Frame(PhysicalAddress);

impl Frame {
    pub fn new(address: PhysicalAddress) -> Option<Self> {
        if address.aligned() {
            Some(Frame(address))
        } else {
            None
        }
    }
}

// Virtual memory address
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct VirtualAddress(usize);

impl VirtualAddress {
    pub const fn new(address: usize) -> Self {
        Self(address)
    }

    pub const fn aligned(&self) -> bool {
        self.0 & PAGE_OFFSET_MASK == 0
    }
}

// Virtual memory page
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct Page(VirtualAddress);

impl Page {
    pub fn new(address: VirtualAddress) -> Option<Self> {
        if address.aligned() {
            Some(Self(address))
        } else {
            None
        }
    }
}

const ENTRY_PRESENT: usize = 1 << 0;
const ENTRY_WRITABLE: usize = 1 << 1;
const ENTRY_ADDRESS_MASK: usize = PAGE_ADDRESS_MASK;
const ENTRY_FLAGS_MASK: usize = !ENTRY_ADDRESS_MASK;

static mut MACHINE: Option<Machine> = None;

#[inline(always)]
pub unsafe fn arch_read<T>(address: usize) -> T {
    MACHINE.as_ref().unwrap().read(address)
}

#[inline(always)]
pub unsafe fn arch_write<T>(address: usize, value: T) {
    MACHINE.as_mut().unwrap().write(address, value)
}

#[inline(always)]
pub unsafe fn arch_invalidate(address: usize) {
    MACHINE.as_mut().unwrap().invalidate(address);
}

#[inline(always)]
pub unsafe fn arch_invalidate_all() {
    MACHINE.as_mut().unwrap().invalidate_all();
}

#[inline(always)]
pub unsafe fn arch_get_table() -> usize {
    MACHINE.as_mut().unwrap().get_table()
}

#[inline(always)]
pub unsafe fn arch_set_table(address: usize) {
    MACHINE.as_mut().unwrap().set_table(address);
}

pub struct Machine {
    pub memory: Box<[u8]>,
    pub map: BTreeMap<usize, usize>,
    pub table_addr: usize,
}

impl Machine {
    pub fn new(memory_size: usize) -> Self {
        Self {
            memory: vec![0; memory_size].into_boxed_slice(),
            map: BTreeMap::new(),
            table_addr: 0,
        }
    }

    pub fn read_phys<T>(&self, phys: usize) -> T {
        let size = mem::size_of::<T>();
        if phys + size <= self.memory.len() {
            unsafe {
                ptr::read(self.memory.as_ptr().add(phys) as *const T)
            }
        } else {
            panic!("read_phys: 0x{:X} size 0x{:X} outside of memory", phys, size);
        }
    }

    pub fn write_phys<T>(&mut self, phys: usize, value: T) {
        let size = mem::size_of::<T>();
        if phys + size <= self.memory.len() {
            unsafe {
                ptr::write(self.memory.as_mut_ptr().add(phys) as *mut T, value);
            }
        } else {
            panic!("write_phys: 0x{:X} size 0x{:X} outside of memory", phys, size);
        }
    }

    pub fn translate(&self, virt: usize) -> Option<(usize, usize)> {
        let page = virt & PAGE_ADDRESS_MASK;
        let offset = virt & PAGE_OFFSET_MASK;
        let phys = self.map.get(&page)?;
        Some((
            (phys & ENTRY_ADDRESS_MASK) + offset,
            phys & ENTRY_FLAGS_MASK,
        ))
    }

    pub fn read<T>(&self, virt: usize) -> T {
        //TODO: allow reading past page boundaries
        let size = mem::size_of::<T>();
        if (virt & PAGE_ADDRESS_MASK) != ((virt + size - 1) & PAGE_ADDRESS_MASK) {
            panic!("read: 0x{:X} size 0x{:X} passes page boundary", virt, size);
        }

        if let Some((phys, _flags)) = self.translate(virt) {
            self.read_phys(phys)
        } else {
            panic!("read: 0x{:X} size 0x{:X} not present", virt, size);
        }
    }

    pub fn write<T>(&mut self, virt: usize, value: T) {
        //TODO: allow writing past page boundaries
        let size = mem::size_of::<T>();
        if (virt & PAGE_ADDRESS_MASK) != ((virt + size - 1) & PAGE_ADDRESS_MASK) {
            panic!("write: 0x{:X} size 0x{:X} passes page boundary", virt, size);
        }

        if let Some((phys, flags)) = self.translate(virt) {
            if flags & ENTRY_WRITABLE != 0 {
                self.write_phys(phys, value);
            } else {
                panic!("write: 0x{:X} size 0x{:X} not writable", virt, size);
            }
        } else {
            panic!("write: 0x{:X} size 0x{:X} not present", virt, size);
        }
    }

    pub fn invalidate(&mut self, _address: usize) {
        unimplemented!();
    }

    pub fn invalidate_all(&mut self) {
        self.map.clear();

        // PML4
        let a4 = self.table_addr;
        for i4 in 0..PAGE_ENTRIES {
            let e3 = self.read_phys::<usize>(a4 + i4 * PAGE_ENTRY_SIZE);
            let f3 = e3 & ENTRY_FLAGS_MASK;
            if f3 & ENTRY_PRESENT == 0 { continue; }

            // Page directory pointer
            let a3 = e3 & ENTRY_ADDRESS_MASK;
            for i3 in 0..PAGE_ENTRIES {
                let e2 = self.read_phys::<usize>(a3 + i3 * PAGE_ENTRY_SIZE);
                let f2 = e2 & ENTRY_FLAGS_MASK;
                if f2 & ENTRY_PRESENT == 0 { continue; }

                // Page directory
                let a2 = e2 & ENTRY_ADDRESS_MASK;
                for i2 in 0..PAGE_ENTRIES {
                    let e1 = self.read_phys::<usize>(a2 + i2 * PAGE_ENTRY_SIZE);
                    let f1 = e1 & ENTRY_FLAGS_MASK;
                    if f1 & ENTRY_PRESENT == 0 { continue; }

                    // Page table
                    let a1 = e1 & ENTRY_ADDRESS_MASK;
                    for i1 in 0..PAGE_ENTRIES {
                        let e = self.read_phys::<usize>(a1 + i1 * PAGE_ENTRY_SIZE);
                        let f = e & ENTRY_FLAGS_MASK;
                        if f & ENTRY_PRESENT == 0 { continue; }

                        // Page
                        let a = e & ENTRY_ADDRESS_MASK;
                        let page =
                            (i4 << 39) |
                            (i3 << 30) |
                            (i2 << 21) |
                            (i1 << 12);
                        println!("map 0x{:X} to 0x{:X}, 0x{:X}", page, a, f);
                        self.map.insert(page, e);
                    }
                }
            }
        }
    }

    pub fn get_table(&self) -> usize {
        self.table_addr
    }

    pub fn set_table(&mut self, address: usize) {
        self.table_addr = address;
        self.invalidate_all();
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct PageEntry(usize);

impl PageEntry {
    pub fn address(&self) -> PhysicalAddress {
        PhysicalAddress(self.0 & ENTRY_ADDRESS_MASK)
    }

    pub fn present(&self) -> bool {
        self.0 & ENTRY_PRESENT != 0
    }
}

pub struct PageTable {
    base: VirtualAddress,
    phys: PhysicalAddress,
    level: usize
}

impl PageTable {
    pub unsafe fn new(base: VirtualAddress, phys: PhysicalAddress, level: usize) -> Self {
        Self { base, phys, level }
    }

    pub unsafe fn top() -> Self {
        Self::new(
            VirtualAddress::new(0),
            PhysicalAddress::new(arch_get_table()),
            PAGE_LEVELS - 1
        )
    }

    pub fn base(&self) -> VirtualAddress {
        self.base
    }

    pub fn phys(&self) -> PhysicalAddress {
        self.phys
    }

    pub fn level(&self) -> usize {
        self.level
    }

    pub unsafe fn virt(&self) -> VirtualAddress {
        //TODO: Recursive mapping

        // Identity mapping
        VirtualAddress(self.phys.0)
    }

    pub fn entry_base(&self, i: usize) -> Option<VirtualAddress> {
        if i < PAGE_ENTRIES {
            Some(VirtualAddress(
                self.base.0 + (i << (self.level * 9 + PAGE_SHIFT))
            ))
        } else {
            None
        }
    }

    pub unsafe fn entry_virt(&self, i: usize) -> Option<VirtualAddress> {
        if i < PAGE_ENTRIES {
            Some(VirtualAddress(
                self.virt().0 + i * PAGE_ENTRY_SIZE
            ))
        } else {
            None
        }
    }

    pub unsafe fn entry(&self, i: usize) -> Option<PageEntry> {
        let addr = self.entry_virt(i)?;
        Some(PageEntry(arch_read::<usize>(addr.0)))
    }

    pub unsafe fn set_entry(&mut self, i: usize, entry: PageEntry) -> Option<()> {
        let addr = self.entry_virt(i)?;
        arch_write::<usize>(addr.0, entry.0);
        Some(())
    }

    pub unsafe fn next(&self, i: usize) -> Option<Self> {
        if self.level > 0 {
            let entry = self.entry(i)?;
            if entry.present() {
                return Some(PageTable::new(
                    self.entry_base(i)?,
                    entry.address(),
                    self.level - 1
                ));
            }
        }
        None
    }
}

unsafe fn dump_tables(table: PageTable) {
    let level = table.level();
    for i in 0..PAGE_ENTRIES {
        if level == 0 {
            if let Some(entry) = table.entry(i) {
                if entry.present() {
                    let base = table.entry_base(i).unwrap();
                    println!("0x{:X}: 0x{:X}", base.0, entry.address().0);
                }
            }
        } else {
            if let Some(next) = table.next(i) {
                dump_tables(next);
            }
        }
    }
}

pub struct MemoryArea {
    base: PhysicalAddress,
    size: usize,
}

fn main() {
    let memory_size = 64 * 1024 * 1024;

    unsafe {
        let megabyte = 0x100000;

        // Create machine with PAGE_ENTRIES pages identity mapped (2 MiB on x86_64)
        // Pages over 1 MiB will be mapped writable
        {
            let mut machine = Machine::new(memory_size);

            // PML4 link to PDP
            let pml4 = 0;
            let pdp = pml4 + PAGE_SIZE;
            let flags = ENTRY_WRITABLE | ENTRY_PRESENT;
            machine.write_phys::<usize>(pml4, pdp | flags);

            // Recursive mapping
            machine.write_phys::<usize>(pml4 + (PAGE_ENTRIES - 1) * PAGE_ENTRY_SIZE, pml4 | flags);

            // PDP link to PD
            let pd = pdp + PAGE_SIZE;
            machine.write_phys::<usize>(pdp, pd | flags);

            // PD link to PT
            let pt = pd + PAGE_SIZE;
            machine.write_phys::<usize>(pd, pt | flags);

            // PT links to frames
            for i in 0..PAGE_ENTRIES {
                let page = i * PAGE_SIZE;
                machine.write_phys::<usize>(pt + i * PAGE_ENTRY_SIZE, page | flags);
            }

            MACHINE = Some(machine);

            // Set table to pml4
            arch_set_table(pml4);
        }

        // Debug table
        dump_tables(PageTable::top());

        // Test read
        println!("0x{:X} = 0x{:X}", megabyte, arch_read::<u8>(megabyte));

        // Test write
        arch_write::<u8>(megabyte, 0x5A);

        // Test read
        println!("0x{:X} = 0x{:X}", megabyte, arch_read::<u8>(megabyte));

        // Initialize memory allocator
        let areas = [MemoryArea {
            base: PhysicalAddress::new(megabyte),
            size: megabyte,
        }];
    }
}
