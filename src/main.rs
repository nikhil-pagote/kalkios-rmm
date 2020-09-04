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

// Physical memory address
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
pub unsafe fn machine_read_u8(address: usize) -> u8 {
    MACHINE.as_ref().unwrap().read_u8(address)
}

#[inline(always)]
pub unsafe fn machine_write_u8(address: usize, value: u8) {
    MACHINE.as_mut().unwrap().write_u8(address, value)
}

#[inline(always)]
pub unsafe fn machine_invalidate(address: usize) {
    MACHINE.as_mut().unwrap().invalidate(address);
}

#[inline(always)]
pub unsafe fn machine_invalidate_all() {
    MACHINE.as_mut().unwrap().invalidate_all();
}

#[inline(always)]
pub unsafe fn machine_set_table(address: usize) {
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

    pub fn read_phys_usize(&self, phys: usize) -> usize {
        if phys + mem::size_of::<usize>() <= self.memory.len() {
            unsafe {
                ptr::read(self.memory.as_ptr().add(phys) as *const usize)
            }
        } else {
            panic!("read_phys_usize {:X} outside of memory", phys);
        }
    }

    pub fn write_phys_usize(&mut self, phys: usize, value: usize) {
        if phys + mem::size_of::<usize>() <= self.memory.len() {
            unsafe {
                ptr::write(self.memory.as_mut_ptr().add(phys) as *mut usize, value);
            }
        } else {
            panic!("write_phys_usize {:X} outside of memory", phys);
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

    pub fn read_u8(&self, virt: usize) -> u8 {
        if let Some((phys, _flags)) = self.translate(virt) {
            self.memory[phys]
        } else {
            panic!("read_u8: {:X} not present", virt);
        }
    }

    pub fn write_u8(&mut self, virt: usize, value: u8) {
        if let Some((phys, flags)) = self.translate(virt) {
            if flags & ENTRY_WRITABLE != 0 {
                self.memory[phys] = value;
            } else {
                panic!("write_u8: {:X} not writable", virt);
            }
        } else {
            panic!("write_u8: {:X} not present", virt);
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
            let e3 = self.read_phys_usize(a4 + i4 * PAGE_ENTRY_SIZE);
            let f3 = e3 & ENTRY_FLAGS_MASK;
            if f3 & ENTRY_PRESENT == 0 { continue; }

            // Page directory pointer
            let a3 = e3 & ENTRY_ADDRESS_MASK;
            for i3 in 0..PAGE_ENTRIES {
                let e2 = self.read_phys_usize(a3 + i3 * PAGE_ENTRY_SIZE);
                let f2 = e2 & ENTRY_FLAGS_MASK;
                if f2 & ENTRY_PRESENT == 0 { continue; }

                // Page directory
                let a2 = e2 & ENTRY_ADDRESS_MASK;
                for i2 in 0..PAGE_ENTRIES {
                    let e1 = self.read_phys_usize(a2 + i2 * PAGE_ENTRY_SIZE);
                    let f1 = e1 & ENTRY_FLAGS_MASK;
                    if f1 & ENTRY_PRESENT == 0 { continue; }

                    // Page table
                    let a1 = e1 & ENTRY_ADDRESS_MASK;
                    for i1 in 0..PAGE_ENTRIES {
                        let e = self.read_phys_usize(a1 + i1 * PAGE_ENTRY_SIZE);
                        let f = e & ENTRY_FLAGS_MASK;
                        if f & ENTRY_PRESENT == 0 { continue; }

                        // Page
                        let a = e & ENTRY_ADDRESS_MASK;
                        let page =
                            (i4 << 39) |
                            (i3 << 30) |
                            (i2 << 21) |
                            (i1 << 12);
                        println!("map {:X} to {:X}, {:X}", page, a, f);
                        self.map.insert(page, e);
                    }
                }
            }
        }
    }

    pub fn set_table(&mut self, address: usize) {
        self.table_addr = address;
        self.invalidate_all();
    }
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
            machine.write_phys_usize(pml4, pdp | ENTRY_PRESENT);

            // PDP link to PD
            let pd = pdp + PAGE_SIZE;
            machine.write_phys_usize(pdp, pd | ENTRY_PRESENT);

            // PD link to PT
            let pt = pd + PAGE_SIZE;
            machine.write_phys_usize(pd, pt | ENTRY_PRESENT);

            // PT links to frames
            for i in 0..PAGE_ENTRIES {
                let page = i * PAGE_SIZE;
                let flags = if page >= megabyte {
                    ENTRY_WRITABLE | ENTRY_PRESENT
                } else {
                    ENTRY_PRESENT
                };
                machine.write_phys_usize(pt + i * PAGE_ENTRY_SIZE, page | flags);
            }

            MACHINE = Some(machine);

            // Set table to pml4
            machine_set_table(pml4);
        }

        // Test read
        println!("{:X} = {:X}", megabyte, machine_read_u8(megabyte));

        // Test write
        machine_write_u8(megabyte, 0x5A);

        // Test read
        println!("{:X} = {:X}", megabyte, machine_read_u8(megabyte));
    }
}
