use crate::PhysicalAddress;

pub use self::buddy::*;
pub use self::bump::*;

mod buddy;
mod bump;

pub trait FrameAllocator {
    unsafe fn allocate(&mut self, count: usize) -> Option<PhysicalAddress>;
    unsafe fn free(&mut self, address: PhysicalAddress, count: usize);
}
