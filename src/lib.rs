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
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct PhysicalAddress(usize);

impl PhysicalAddress {
    pub const fn new(address: usize) -> Self {
        Self(address)
    }

    pub fn data(&self) -> usize {
        self.0
    }
}

// Virtual memory address
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct VirtualAddress(usize);

impl VirtualAddress {
    pub const fn new(address: usize) -> Self {
        Self(address)
    }

    pub fn data(&self) -> usize {
        self.0
    }
}

pub struct MemoryArea {
    base: PhysicalAddress,
    size: usize,
}
