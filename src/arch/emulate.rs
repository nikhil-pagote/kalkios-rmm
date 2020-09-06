use core::{
    marker::PhantomData,
    mem,
    ptr,
};
use std::collections::BTreeMap;

use crate::{
    Arch,
    PhysicalAddress,
    MemoryArea,
    arch::x86_64::X8664Arch,
};

pub struct EmulateArch;

impl Arch for EmulateArch {
    const PAGE_SHIFT: usize = X8664Arch::PAGE_SHIFT;
    const PAGE_ENTRY_SHIFT: usize = X8664Arch::PAGE_ENTRY_SHIFT;
    const PAGE_LEVELS: usize = X8664Arch::PAGE_LEVELS;

    const ENTRY_FLAG_PRESENT: usize = X8664Arch::ENTRY_FLAG_PRESENT;
    const ENTRY_FLAG_WRITABLE: usize = X8664Arch::ENTRY_FLAG_WRITABLE;
    const ENTRY_FLAG_USER: usize = X8664Arch::ENTRY_FLAG_USER;
    const ENTRY_FLAG_HUGE: usize = X8664Arch::ENTRY_FLAG_HUGE;
    const ENTRY_FLAG_GLOBAL: usize = X8664Arch::ENTRY_FLAG_GLOBAL;
    const ENTRY_FLAG_NO_EXEC: usize = X8664Arch::ENTRY_FLAG_NO_EXEC;

    unsafe fn init() -> &'static [MemoryArea] {
        // Create machine with PAGE_ENTRIES pages identity mapped (2 MiB on x86_64)
        // Pages over 1 MiB will be mapped writable
        let mut machine = Machine::new(MEMORY_SIZE);

        // PML4 link to PDP
        let pml4 = 0;
        let pdp = pml4 + Self::PAGE_SIZE;
        let flags = Self::ENTRY_FLAG_WRITABLE | Self::ENTRY_FLAG_PRESENT;
        machine.write_phys::<usize>(pml4, pdp | flags);

        // Recursive mapping
        machine.write_phys::<usize>(pml4 + (Self::PAGE_ENTRIES - 1) * Self::PAGE_ENTRY_SIZE, pml4 | flags);

        // PDP link to PD
        let pd = pdp + Self::PAGE_SIZE;
        machine.write_phys::<usize>(pdp, pd | flags);

        // PD link to PT
        let pt = pd + Self::PAGE_SIZE;
        machine.write_phys::<usize>(pd, pt | flags);

        // PT links to frames
        for i in 0..Self::PAGE_ENTRIES {
            let page = i * Self::PAGE_SIZE;
            machine.write_phys::<usize>(pt + i * Self::PAGE_ENTRY_SIZE, page | flags);
        }

        MACHINE = Some(machine);

        // Set table to pml4
        EmulateArch::set_table(pml4);

        &MEMORY_AREAS
    }

    #[inline(always)]
    unsafe fn read<T>(address: usize) -> T {
        MACHINE.as_ref().unwrap().read(address)
    }

    #[inline(always)]
    unsafe fn write<T>(address: usize, value: T) {
        MACHINE.as_mut().unwrap().write(address, value)
    }

    #[inline(always)]
    unsafe fn invalidate(address: usize) {
        MACHINE.as_mut().unwrap().invalidate(address);
    }

    #[inline(always)]
    unsafe fn invalidate_all() {
        MACHINE.as_mut().unwrap().invalidate_all();
    }

    #[inline(always)]
    unsafe fn table() -> usize {
        MACHINE.as_mut().unwrap().get_table()
    }

    #[inline(always)]
    unsafe fn set_table(address: usize) {
        MACHINE.as_mut().unwrap().set_table(address);
    }
}

const MEGABYTE: usize = 1024 * 1024;
const MEMORY_SIZE: usize = 64 * MEGABYTE;
static MEMORY_AREAS: [MemoryArea; 1] = [
    MemoryArea {
        base: PhysicalAddress::new(MEGABYTE),
        size: MEMORY_SIZE,
    }
];

static mut MACHINE: Option<Machine<EmulateArch>> = None;

struct Machine<A> {
    memory: Box<[u8]>,
    map: BTreeMap<usize, usize>,
    table_addr: usize,
    phantom: PhantomData<A>,
}

impl<A: Arch> Machine<A> {
    fn new(memory_size: usize) -> Self {
        Self {
            memory: vec![0; memory_size].into_boxed_slice(),
            map: BTreeMap::new(),
            table_addr: 0,
            phantom: PhantomData,
        }
    }

    fn read_phys<T>(&self, phys: usize) -> T {
        let size = mem::size_of::<T>();
        if phys + size <= self.memory.len() {
            unsafe {
                ptr::read(self.memory.as_ptr().add(phys) as *const T)
            }
        } else {
            panic!("read_phys: 0x{:X} size 0x{:X} outside of memory", phys, size);
        }
    }

    fn write_phys<T>(&mut self, phys: usize, value: T) {
        let size = mem::size_of::<T>();
        if phys + size <= self.memory.len() {
            unsafe {
                ptr::write(self.memory.as_mut_ptr().add(phys) as *mut T, value);
            }
        } else {
            panic!("write_phys: 0x{:X} size 0x{:X} outside of memory", phys, size);
        }
    }

    fn translate(&self, virt: usize) -> Option<(usize, usize)> {
        let page = virt & A::PAGE_ADDRESS_MASK;
        let offset = virt & A::PAGE_OFFSET_MASK;
        let phys = self.map.get(&page)?;
        Some((
            (phys & A::ENTRY_ADDRESS_MASK) + offset,
            phys & A::ENTRY_FLAGS_MASK,
        ))
    }

    fn read<T>(&self, virt: usize) -> T {
        //TODO: allow reading past page boundaries
        let size = mem::size_of::<T>();
        if (virt & A::PAGE_ADDRESS_MASK) != ((virt + (size - 1)) & A::PAGE_ADDRESS_MASK) {
            panic!("read: 0x{:X} size 0x{:X} passes page boundary", virt, size);
        }

        if let Some((phys, _flags)) = self.translate(virt) {
            self.read_phys(phys)
        } else {
            panic!("read: 0x{:X} size 0x{:X} not present", virt, size);
        }
    }

    fn write<T>(&mut self, virt: usize, value: T) {
        //TODO: allow writing past page boundaries
        let size = mem::size_of::<T>();
        if (virt & A::PAGE_ADDRESS_MASK) != ((virt + (size - 1)) & A::PAGE_ADDRESS_MASK) {
            panic!("write: 0x{:X} size 0x{:X} passes page boundary", virt, size);
        }

        if let Some((phys, flags)) = self.translate(virt) {
            if flags & A::ENTRY_FLAG_WRITABLE != 0 {
                self.write_phys(phys, value);
            } else {
                panic!("write: 0x{:X} size 0x{:X} not writable", virt, size);
            }
        } else {
            panic!("write: 0x{:X} size 0x{:X} not present", virt, size);
        }
    }

    fn invalidate(&mut self, _address: usize) {
        unimplemented!();
    }

    fn invalidate_all(&mut self) {
        self.map.clear();

        // PML4
        let a4 = self.table_addr;
        for i4 in 0..A::PAGE_ENTRIES {
            let e3 = self.read_phys::<usize>(a4 + i4 * A::PAGE_ENTRY_SIZE);
            let f3 = e3 & A::ENTRY_FLAGS_MASK;
            if f3 & A::ENTRY_FLAG_PRESENT == 0 { continue; }

            // Page directory pointer
            let a3 = e3 & A::ENTRY_ADDRESS_MASK;
            for i3 in 0..A::PAGE_ENTRIES {
                let e2 = self.read_phys::<usize>(a3 + i3 * A::PAGE_ENTRY_SIZE);
                let f2 = e2 & A::ENTRY_FLAGS_MASK;
                if f2 & A::ENTRY_FLAG_PRESENT == 0 { continue; }

                // Page directory
                let a2 = e2 & A::ENTRY_ADDRESS_MASK;
                for i2 in 0..A::PAGE_ENTRIES {
                    let e1 = self.read_phys::<usize>(a2 + i2 * A::PAGE_ENTRY_SIZE);
                    let f1 = e1 & A::ENTRY_FLAGS_MASK;
                    if f1 & A::ENTRY_FLAG_PRESENT == 0 { continue; }

                    // Page table
                    let a1 = e1 & A::ENTRY_ADDRESS_MASK;
                    for i1 in 0..A::PAGE_ENTRIES {
                        let e = self.read_phys::<usize>(a1 + i1 * A::PAGE_ENTRY_SIZE);
                        let f = e & A::ENTRY_FLAGS_MASK;
                        if f & A::ENTRY_FLAG_PRESENT == 0 { continue; }

                        // Page
                        let a = e & A::ENTRY_ADDRESS_MASK;
                        let page =
                            if i4 >= 256 { 0xFFFF_0000_0000_0000 } else { 0 } |
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

    fn get_table(&self) -> usize {
        self.table_addr
    }

    fn set_table(&mut self, address: usize) {
        self.table_addr = address;
        self.invalidate_all();
    }
}
