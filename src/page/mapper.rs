use core::marker::PhantomData;

use crate::{
    Arch,
    FrameAllocator,
    PageEntry,
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
    pub unsafe fn new(allocator: &'f mut F) -> Option<Self> {
        let table_addr = allocator.allocate_one()?;
        Some(Self {
            table_addr,
            allocator,
            phantom: PhantomData,
        })
    }

    pub unsafe fn activate(&mut self) {
        A::set_table(self.table_addr);
    }

    pub unsafe fn table(&self) -> PageTable<A> {
        PageTable::new(
            VirtualAddress::new(0),
            self.table_addr,
            A::PAGE_LEVELS - 1
        )
    }

    pub unsafe fn map(&mut self, virt: VirtualAddress, entry: PageEntry<A>) -> Option<()> {
        let mut table = self.table();
        loop {
            let i = table.index_of(virt)?;
            if table.level() == 0 {
                //TODO: check for overwriting entry
                table.set_entry(i, entry);
                return Some(());
            } else {
                let next_opt = table.next(i);
                let next = match next_opt {
                    Some(some) => some,
                    None => {
                        let phys = self.allocator.allocate_one()?;
                        //TODO: correct flags?
                        table.set_entry(i, PageEntry::new(phys.data() | A::ENTRY_FLAG_WRITABLE | A::ENTRY_FLAG_PRESENT));
                        table.next(i)?
                    }
                };
                table = next;
            }
        }
    }
}
