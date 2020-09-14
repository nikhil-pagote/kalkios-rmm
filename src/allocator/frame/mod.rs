use crate::PhysicalAddress;

pub use self::buddy::*;
pub use self::bump::*;

mod buddy;
mod bump;

#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct FrameCount(usize);

impl FrameCount {
    pub fn new(count: usize) -> Self {
        Self(count)
    }

    pub fn data(&self) -> usize {
        self.0
    }
}

pub struct FrameUsage {
    used: FrameCount,
    total: FrameCount,
}

impl FrameUsage {
    pub fn new(used: FrameCount, total: FrameCount) -> Self {
        Self { used, total }
    }
    
    pub fn used(&self) -> FrameCount {
        self.used
    }

    pub fn free(&self) -> FrameCount {
        FrameCount(self.total.0 - self.used.0)
    }

    pub fn total(&self) -> FrameCount {
        self.total
    }
}

pub trait FrameAllocator {
    unsafe fn allocate(&mut self, count: FrameCount) -> Option<PhysicalAddress>;

    unsafe fn free(&mut self, address: PhysicalAddress, count: FrameCount);

    unsafe fn allocate_one(&mut self) -> Option<PhysicalAddress> {
        self.allocate(FrameCount::new(1))
    }

    unsafe fn free_one(&mut self, address: PhysicalAddress) {
        self.free(address, FrameCount::new(1));
    }

    unsafe fn usage(&self) -> FrameUsage;
}
