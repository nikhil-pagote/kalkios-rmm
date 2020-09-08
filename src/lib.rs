#![cfg_attr(not(feature = "std"), no_std)]
#![feature(asm)]

pub use crate::{
    allocator::*,
    arch::*,
    page::*,
};

mod allocator;
mod arch;
mod page;

pub const KILOBYTE: usize = 1024;
pub const MEGABYTE: usize = KILOBYTE * KILOBYTE;
pub const GIGABYTE: usize = KILOBYTE * MEGABYTE;
pub const TERABYTE: usize = KILOBYTE * GIGABYTE;

// Physical memory address
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct PhysicalAddress(usize);

impl PhysicalAddress {
    #[inline(always)]
    pub const fn new(address: usize) -> Self {
        Self(address)
    }

    #[inline(always)]
    pub fn data(&self) -> usize {
        self.0
    }

    #[inline(always)]
    pub fn add(self, offset: usize) -> Self {
        Self(self.0 + offset)
    }
}

// Virtual memory address
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct VirtualAddress(usize);

impl VirtualAddress {
    #[inline(always)]
    pub const fn new(address: usize) -> Self {
        Self(address)
    }

    #[inline(always)]
    pub fn data(&self) -> usize {
        self.0
    }

    #[inline(always)]
    pub fn add(self, offset: usize) -> Self {
        Self(self.0 + offset)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MemoryArea {
    pub base: PhysicalAddress,
    pub size: usize,
}
