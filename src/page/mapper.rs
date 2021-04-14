use core::marker::PhantomData;

use crate::{
    Arch,
    FrameAllocator,
    PageEntry,
    PageFlags,
    PageFlush,
    PageTable,
    PhysicalAddress,
    VirtualAddress,
};

pub struct PageMapper<'f, A, F> {
    table_addr: PhysicalAddress,
    allocator: &'f mut F,
    phantom: PhantomData<A>,
}

impl<'f, A: Arch, F: FrameAllocator> PageMapper<'f, A, F> {
    pub unsafe fn new(table_addr: PhysicalAddress, allocator: &'f mut F) -> Self {
        Self {
            table_addr,
            allocator,
            phantom: PhantomData,
        }
    }

    pub unsafe fn create(allocator: &'f mut F) -> Option<Self> {
        let table_addr = allocator.allocate_one()?;
        Some(Self::new(table_addr, allocator))
    }

    pub unsafe fn current(allocator: &'f mut F) -> Self {
        let table_addr = A::table();
        Self::new(table_addr, allocator)
    }

    pub unsafe fn make_current(&mut self) {
        A::set_table(self.table_addr);
    }

    pub unsafe fn table(&self) -> PageTable<A> {
        PageTable::new(
            VirtualAddress::new(0),
            self.table_addr,
            A::PAGE_LEVELS - 1
        )
    }

    pub unsafe fn map(&mut self, virt: VirtualAddress, flags: PageFlags<A>) -> Option<PageFlush<A>> {
        let phys = self.allocator.allocate_one()?;
        self.map_phys(virt, phys, flags)
    }

    pub unsafe fn map_phys(&mut self, virt: VirtualAddress, phys: PhysicalAddress, flags: PageFlags<A>) -> Option<PageFlush<A>> {
        //TODO: verify virt and phys are aligned
        //TODO: verify flags have correct bits
        let entry = PageEntry::new(phys.data() | flags.data());
        let mut table = self.table();
        loop {
            let i = table.index_of(virt)?;
            if table.level() == 0 {
                //TODO: check for overwriting entry
                table.set_entry(i, entry);
                return Some(PageFlush::new(virt));
            } else {
                let next_opt = table.next(i);
                let next = match next_opt {
                    Some(some) => some,
                    None => {
                        let next_phys = self.allocator.allocate_one()?;
                        //TODO: correct flags?
                        table.set_entry(i, PageEntry::new(next_phys.data() | A::ENTRY_FLAG_READWRITE | A::ENTRY_FLAG_DEFAULT_TABLE));
                        table.next(i)?
                    }
                };
                table = next;
            }
        }
    }

    pub unsafe fn unmap(&mut self, virt: VirtualAddress) -> Option<PageFlush<A>> {
        let (old, flush) = self.unmap_phys(virt)?;
        self.allocator.free_one(old.address());
        Some(flush)
    }

    pub unsafe fn unmap_phys(&mut self, virt: VirtualAddress) -> Option<(PageEntry<A>, PageFlush<A>)> {
        //TODO: verify virt is aligned
        let mut table = self.table();
        //TODO: unmap parents
        loop {
            let i = table.index_of(virt)?;
            if table.level() == 0 {
                let entry_opt = table.entry(i);
                table.set_entry(i, PageEntry::new(0));
                let entry = entry_opt?;
                return Some((entry, PageFlush::new(virt)));
            } else {
                table = table.next(i)?;
            }
        }
    }
}
