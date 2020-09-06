use core::marker::PhantomData;

use crate::{
    Arch,
    PhysicalAddress,
};

#[derive(Clone, Copy, Debug)]
pub struct PageEntry<A> {
    data: usize,
    phantom: PhantomData<A>,
}

impl<A: Arch> PageEntry<A> {
    #[inline(always)]
    pub fn new(data: usize) -> Self {
        Self { data, phantom: PhantomData }
    }

    #[inline(always)]
    pub fn data(&self) -> usize {
        self.data
    }

    #[inline(always)]
    pub fn address(&self) -> PhysicalAddress {
        PhysicalAddress(self.data & A::ENTRY_ADDRESS_MASK)
    }

    #[inline(always)]
    pub fn flags(&self) -> usize {
        self.data & A::ENTRY_FLAGS_MASK
    }

    #[inline(always)]
    pub fn present(&self) -> bool {
        self.data & A::ENTRY_FLAG_PRESENT != 0
    }
}
