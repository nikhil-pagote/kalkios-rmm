use core::marker::PhantomData;

use crate::{
    Arch,
    FrameAllocator,
    FrameCount,
    MemoryArea,
    PhysicalAddress,
};

pub struct BumpAllocator<A> {
    areas: &'static [MemoryArea],
    offset: usize,
    phantom: PhantomData<A>,
}

impl<A: Arch> BumpAllocator<A> {
    pub fn new(areas: &'static [MemoryArea], offset: usize) -> Self {
        Self {
            areas,
            offset,
            phantom: PhantomData,
        }
    }

    pub fn areas(&self) -> &'static [MemoryArea] {
        self.areas
    }

    pub fn offset(&self) -> usize {
        self.offset
    }
}

impl<A: Arch> FrameAllocator for BumpAllocator<A> {
    unsafe fn allocate(&mut self, count: FrameCount) -> Option<PhysicalAddress> {
        //TODO: support allocation of multiple pages
        if count.data() != 1 {
            return None;
        }

        let mut offset = self.offset;
        for area in self.areas.iter() {
            if offset < area.size {
                self.offset += A::PAGE_SIZE;
                return Some(area.base.add(offset));
            }
            offset -= area.size;
        }
        None
    }

    unsafe fn free(&mut self, _address: PhysicalAddress, _count: FrameCount) {
        unimplemented!();
    }
}
