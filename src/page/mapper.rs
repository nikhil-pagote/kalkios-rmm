use core::marker::PhantomData;

use crate::{
    Arch,
    FrameAllocator,
    PageEntry,
    PageFlags,
    PageFlush,
    PageTable,
    PhysicalAddress,
    TableKind,
    VirtualAddress,
};

pub struct PageMapper<A, F> {
    table_addr: PhysicalAddress,
    allocator: F,
    _phantom: PhantomData<fn() -> A>,
}

impl<A: Arch, F: FrameAllocator> PageMapper<A, F> {
    pub unsafe fn new(table_addr: PhysicalAddress, allocator: F) -> Self {
        Self {
            table_addr,
            allocator,
            _phantom: PhantomData,
        }
    }

    pub unsafe fn create(mut allocator: F) -> Option<Self> {
        let table_addr = allocator.allocate_one()?;
        Some(Self::new(table_addr, allocator))
    }

    pub unsafe fn current(allocator: F) -> Self {
        let table_addr = A::table();
        Self::new(table_addr, allocator)
    }
    pub fn is_current(&self) -> bool {
        unsafe { self.table().phys() == A::table() }
    }

    pub unsafe fn make_current(&self) {
        A::set_table(self.table_addr);
    }

    pub fn table(&self) -> PageTable<A> {
        // SAFETY: The only way to initialize a PageMapper is via new(), and we assume it upholds
        // all necessary invariants for this to be safe.
        unsafe {
            PageTable::new(
                VirtualAddress::new(0),
                self.table_addr,
                A::PAGE_LEVELS - 1
            )
        }
    }

    pub unsafe fn remap(&mut self, virt: VirtualAddress, flags: PageFlags<A>) -> Option<PageFlush<A>> {
        self.visit(virt, |p1, i| {
            let mut entry = p1.entry(i)?;
            entry.set_flags(flags);
            p1.set_entry(i, entry);
            Some(PageFlush::new(virt))
        }).flatten()
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
                        let flags = A::ENTRY_FLAG_READWRITE | A::ENTRY_FLAG_DEFAULT_TABLE | if virt.kind() == TableKind::User { A::ENTRY_FLAG_USER } else { 0 };
                        table.set_entry(i, PageEntry::new(next_phys.data() | flags));
                        table.next(i)?
                    }
                };
                table = next;
            }
        }
    }
    pub unsafe fn map_linearly(&mut self, phys: PhysicalAddress, flags: PageFlags<A>) -> Option<(VirtualAddress, PageFlush<A>)> {
        let virt = A::phys_to_virt(phys);
        self.map_phys(virt, phys, flags).map(|flush| (virt, flush))
    }
    fn visit<T>(&self, virt: VirtualAddress, f: impl FnOnce(&mut PageTable<A>, usize) -> T) -> Option<T> {
        let mut table = self.table();
        unsafe {
            loop {
                let i = table.index_of(virt)?;
                if table.level() == 0 {
                    return Some(f(&mut table, i));
                } else {
                    table = table.next(i)?;
                }
            }
        }
    }
    pub fn translate(&self, virt: VirtualAddress) -> Option<(PhysicalAddress, PageFlags<A>)> {
        let entry = self.visit(virt, |p1, i| unsafe { p1.entry(i) })??;
        Some((entry.address().ok()?, entry.flags()))
    }

    pub unsafe fn unmap(&mut self, virt: VirtualAddress, unmap_parents: bool) -> Option<PageFlush<A>> {
        let (old, _, flush) = self.unmap_phys(virt, unmap_parents)?;
        self.allocator.free_one(old);
        Some(flush)
    }

    pub unsafe fn unmap_phys(&mut self, virt: VirtualAddress, _unmap_parents: bool) -> Option<(PhysicalAddress, PageFlags<A>, PageFlush<A>)> {
        //TODO: verify virt is aligned
        let mut table = self.table();
        let level = table.level();
        //TODO: use unmap_parents
        unmap_phys_inner(virt, &mut table, level, false, &mut self.allocator).map(|(pa, pf)| (pa, pf, PageFlush::new(virt)))
    }
}
unsafe fn unmap_phys_inner<A: Arch>(virt: VirtualAddress, table: &mut PageTable<A>, initial_level: usize, unmap_parents: bool, allocator: &mut impl FrameAllocator) -> Option<(PhysicalAddress, PageFlags<A>)> {
    let i = table.index_of(virt)?;

    if table.level() == 0 {
        let entry_opt = table.entry(i);
        table.set_entry(i, PageEntry::new(0));
        let entry = entry_opt?;

        Some((entry.address().ok()?, entry.flags()))
    } else {
        let mut subtable = table.next(i)?;

        let res = unmap_phys_inner(virt, &mut subtable, initial_level, unmap_parents, allocator)?;

        if unmap_parents {
            // TODO: Use a counter? This would reduce the remaining number of available bits, but could be
            // faster (benchmark is needed).
            let is_still_populated = (0..A::PAGE_ENTRIES).map(|j| subtable.entry(j).expect("must be within bounds")).any(|e| e.present());

            if !is_still_populated {
                allocator.free_one(table.phys());
                table.set_entry(i, PageEntry::new(0));
            }
        }

        Some(res)
    }
}
impl<A, F: core::fmt::Debug> core::fmt::Debug for PageMapper<A, F> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PageMapper")
            .field("frame", &self.table_addr)
            .field("allocator", &self.allocator)
            .finish()
    }
}
