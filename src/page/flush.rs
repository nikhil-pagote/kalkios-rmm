use core::{
    marker::PhantomData,
    mem,
};

use crate::{
    Arch,
    VirtualAddress,
};

pub trait Flusher<A> {
    fn consume(&mut self, flush: PageFlush<A>);
}

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

    pub fn flush(self) {
        unsafe { A::invalidate_all(); }
    }

    pub unsafe fn ignore(self) {
        mem::forget(self);
    }
}
impl<A: Arch> Flusher<A> for PageFlushAll<A> {
    fn consume(&mut self, flush: PageFlush<A>) {
        unsafe { flush.ignore(); }
    }
}
impl<A: Arch, T: Flusher<A> + ?Sized> Flusher<A> for &mut T {
    fn consume(&mut self, flush: PageFlush<A>) {
        <T as Flusher<A>>::consume(self, flush)
    }
}
impl<A: Arch> Flusher<A> for () {
    fn consume(&mut self, _: PageFlush<A>) {}
}
