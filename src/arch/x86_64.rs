use crate::{
    Arch,
    MemoryArea
};

pub struct X8664Arch;

impl Arch for X8664Arch {
    const PAGE_SHIFT: usize = 12; // 4096 bytes
    const PAGE_ENTRY_SHIFT: usize = 9; // 512 entries, 8 bytes each
    const PAGE_LEVELS: usize = 4; // PML4, PDP, PD, PT

    const ENTRY_FLAG_PRESENT: usize = 1 << 0;
    const ENTRY_FLAG_WRITABLE: usize = 1 << 1;
    const ENTRY_FLAG_USER: usize = 1 << 2;
    const ENTRY_FLAG_HUGE: usize = 1 << 7;
    const ENTRY_FLAG_GLOBAL: usize = 1 << 8;
    const ENTRY_FLAG_NO_EXEC: usize = 1 << 63;

    unsafe fn init() -> &'static [MemoryArea] {
        unimplemented!()
    }

    #[inline(always)]
    unsafe fn invalidate(address: usize) {
        //TODO: invlpg address
        unimplemented!();
    }

    #[inline(always)]
    unsafe fn invalidate_all() {
        //TODO: mov cr3, cr3
        unimplemented!();
    }

    #[inline(always)]
    unsafe fn table() -> usize {
        //TODO: return cr3
        unimplemented!();
    }

    #[inline(always)]
    unsafe fn set_table(address: usize) {
        //TODO: mov cr3, address
        unimplemented!();
    }
}
