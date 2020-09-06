use core::marker::PhantomData;

use crate::{
    Arch,
    PhysicalAddress,
    VirtualAddress,
    PAGE_ENTRIES,
    PAGE_ENTRY_MASK,
    PAGE_ENTRY_SHIFT,
    PAGE_ENTRY_SIZE,
    PAGE_LEVELS,
    PAGE_SHIFT,
};
use super::PageEntry;

pub struct PageTable<A> {
    base: VirtualAddress,
    phys: PhysicalAddress,
    level: usize,
    phantom: PhantomData<A>,
}

impl<A: Arch> PageTable<A> {
    pub unsafe fn new(base: VirtualAddress, phys: PhysicalAddress, level: usize) -> Self {
        Self { base, phys, level, phantom: PhantomData }
    }

    pub unsafe fn top() -> Self {
        Self::new(
            VirtualAddress::new(0),
            PhysicalAddress::new(A::table()),
            PAGE_LEVELS - 1
        )
    }

    pub fn base(&self) -> VirtualAddress {
        self.base
    }

    pub fn phys(&self) -> PhysicalAddress {
        self.phys
    }

    pub fn level(&self) -> usize {
        self.level
    }

    pub unsafe fn virt(&self) -> VirtualAddress {
        // Recursive mapping
        let mut addr = 0xFFFF_FFFF_FFFF_F000;
        for level in (self.level + 1 .. PAGE_LEVELS).rev() {
            let index = (self.base.0 >> (level * PAGE_ENTRY_SHIFT + PAGE_SHIFT)) & PAGE_ENTRY_MASK;
            addr <<= PAGE_ENTRY_SHIFT;
            addr |= index << PAGE_SHIFT;
        }
        VirtualAddress(addr)

        // Identity mapping
        //VirtualAddress(self.phys.0)
    }

    pub fn entry_base(&self, i: usize) -> Option<VirtualAddress> {
        if i < PAGE_ENTRIES {
            Some(VirtualAddress(
                self.base.0 + (i << (self.level * PAGE_ENTRY_SHIFT + PAGE_SHIFT))
            ))
        } else {
            None
        }
    }

    pub unsafe fn entry_virt(&self, i: usize) -> Option<VirtualAddress> {
        if i < PAGE_ENTRIES {
            Some(VirtualAddress(
                self.virt().0 + i * PAGE_ENTRY_SIZE
            ))
        } else {
            None
        }
    }

    pub unsafe fn entry(&self, i: usize) -> Option<PageEntry> {
        let addr = self.entry_virt(i)?;
        Some(PageEntry::new(A::read::<usize>(addr.0)))
    }

    pub unsafe fn set_entry(&mut self, i: usize, entry: PageEntry) -> Option<()> {
        let addr = self.entry_virt(i)?;
        A::write::<usize>(addr.0, entry.data());
        Some(())
    }

    pub unsafe fn next(&self, i: usize) -> Option<Self> {
        if self.level > 0 {
            let entry = self.entry(i)?;
            if entry.present() {
                return Some(PageTable::new(
                    self.entry_base(i)?,
                    entry.address(),
                    self.level - 1
                ));
            }
        }
        None
    }
}
