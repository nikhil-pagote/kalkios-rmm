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
    pub fn new(data: usize) -> Self {
        Self { data, phantom: PhantomData }
    }

    pub fn data(&self) -> usize {
        self.data
    }

    pub fn address(&self) -> PhysicalAddress {
        PhysicalAddress(self.data & A::ENTRY_ADDRESS_MASK)
    }

    pub fn flags(&self) -> usize {
        self.data & A::ENTRY_FLAGS_MASK
    }

    pub fn present(&self) -> bool {
        self.data & A::ENTRY_FLAG_PRESENT != 0
    }
}
