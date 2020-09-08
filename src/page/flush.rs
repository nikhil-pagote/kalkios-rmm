use core::{
    marker::PhantomData,
    mem,
};

use crate::{
    Arch,
    VirtualAddress,
};

#[must_use = "The page table must be flushed, or the changes unsafely ignored"]
pub struct PageFlush<A> {
    virt: VirtualAddress,
    phantom: PhantomData<A>,
}

impl<A: Arch> PageFlush<A> {
    pub fn new(virt: VirtualAddress) -> Self {
        Self {
            virt,
            phantom: PhantomData,
        }
    }

    pub fn flush(self) {
        unsafe { A::invalidate(self.virt); }
    }

    pub unsafe fn ignore(self) {
        mem::forget(self);
    }
}

#[must_use = "The page table must be flushed, or the changes unsafely ignored"]
pub struct PageFlushAll<A> {
    phantom: PhantomData<A>,
}

impl <A: Arch> PageFlushAll<A> {
    pub fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }

    pub fn consume(&self, flush: PageFlush<A>) {
        unsafe { flush.ignore(); }
    }

    pub fn flush(self) {
        unsafe { A::invalidate_all(); }
    }

    pub unsafe fn ignore(self) {
        mem::forget(self);
    }
}
