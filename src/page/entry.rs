use crate::{
    PhysicalAddress,
    ENTRY_ADDRESS_MASK,
    ENTRY_FLAG_PRESENT,
};

#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct PageEntry(usize);

impl PageEntry {
    pub unsafe fn new(data: usize) -> Self {
        Self(data)
    }

    pub fn data(&self) -> usize {
        self.0
    }

    pub fn address(&self) -> PhysicalAddress {
        PhysicalAddress(self.0 & ENTRY_ADDRESS_MASK)
    }

    pub fn present(&self) -> bool {
        self.0 & ENTRY_FLAG_PRESENT != 0
    }
}
