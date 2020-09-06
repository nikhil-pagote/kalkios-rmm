pub use crate::{
    arch::Arch,
    arch::emulate::*,
    page::{
        PageEntry,
        PageTable
    },
};

mod arch;
mod page;

// Physical memory address
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct PhysicalAddress(usize);

impl PhysicalAddress {
    pub const fn new(address: usize) -> Self {
        Self(address)
    }

    pub unsafe fn data(&self) -> usize {
        self.0
    }

    pub const fn aligned(&self) -> bool {
        self.0 & PAGE_OFFSET_MASK == 0
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

    pub unsafe fn data(&self) -> usize {
        self.0
    }

    pub const fn aligned(&self) -> bool {
        self.0 & PAGE_OFFSET_MASK == 0
    }
}

pub struct MemoryArea {
    base: PhysicalAddress,
    size: usize,
}
