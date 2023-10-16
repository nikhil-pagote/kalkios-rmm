use core::marker::PhantomData;

use crate::{
    Arch,
    FrameAllocator,
    FrameCount,
    FrameUsage,
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
        let mut offset = self.offset;
        for area in self.areas.iter() {
            if offset < area.size {
                if area.size - offset < count.data() {
                    return None;
                }

                let page_phys = area.base.add(offset);
                let page_virt = A::phys_to_virt(page_phys);
                A::write_bytes(page_virt, 0, count.data() * A::PAGE_SIZE);
                self.offset += count.data() * A::PAGE_SIZE;
                return Some(page_phys);
            }
            offset -= area.size;
        }
        None
    }

    unsafe fn free(&mut self, _address: PhysicalAddress, _count: FrameCount) {
        unimplemented!("BumpAllocator::free not implemented");
    }

    unsafe fn usage(&self) -> FrameUsage {
        let mut total = 0;
        for area in self.areas.iter() {
            total += area.size >> A::PAGE_SHIFT;
        }
        let used = self.offset >> A::PAGE_SHIFT;
        FrameUsage::new(FrameCount::new(used), FrameCount::new(total))
    }
}
