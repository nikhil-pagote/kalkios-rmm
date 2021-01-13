use crate::{
    Arch,
    MemoryArea,
    PhysicalAddress,
    VirtualAddress,
};

#[derive(Clone, Copy)]
pub struct AArch64Arch;

impl Arch for AArch64Arch {
    const PAGE_SHIFT: usize = 12; // 4096 bytes
    const PAGE_ENTRY_SHIFT: usize = 9; // 512 entries, 8 bytes each
    const PAGE_LEVELS: usize = 4; // PML4, PDP, PD, PT

    //TODO
    const ENTRY_ADDRESS_SHIFT: usize = 52;
    const ENTRY_FLAG_PRESENT: usize = 1 << 0;
    const ENTRY_FLAG_WRITABLE: usize = 1 << 1;
    const ENTRY_FLAG_USER: usize = 1 << 2;
    const ENTRY_FLAG_HUGE: usize = 1 << 7;
    const ENTRY_FLAG_GLOBAL: usize = 1 << 8;
    const ENTRY_FLAG_NO_EXEC: usize = 1 << 63;

    const PHYS_OFFSET: usize = Self::PAGE_NEGATIVE_MASK + (Self::PAGE_ADDRESS_SIZE >> 1); // PML4 slot 256 and onwards

    unsafe fn init() -> &'static [MemoryArea] {
        unimplemented!("AArch64Arch::init unimplemented");
    }

    #[inline(always)]
    unsafe fn invalidate(address: VirtualAddress) {
        unimplemented!()
    }

    #[inline(always)]
    unsafe fn table() -> PhysicalAddress {
        unimplemented!()
    }

    #[inline(always)]
    unsafe fn set_table(address: PhysicalAddress) {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use crate::Arch;
    use super::AArch64Arch;

    #[test]
    fn constants() {
        assert_eq!(AArch64Arch::PAGE_SIZE, 4096);
        assert_eq!(AArch64Arch::PAGE_OFFSET_MASK, 0xFFF);
        assert_eq!(AArch64Arch::PAGE_ADDRESS_SHIFT, 48);
        assert_eq!(AArch64Arch::PAGE_ADDRESS_SIZE, 0x0001_0000_0000_0000);
        assert_eq!(AArch64Arch::PAGE_ADDRESS_MASK, 0x0000_FFFF_FFFF_F000);
        assert_eq!(AArch64Arch::PAGE_ENTRY_SIZE, 8);
        assert_eq!(AArch64Arch::PAGE_ENTRIES, 512);
        assert_eq!(AArch64Arch::PAGE_ENTRY_MASK, 0x1FF);
        assert_eq!(AArch64Arch::PAGE_NEGATIVE_MASK, 0xFFFF_0000_0000_0000);

        assert_eq!(AArch64Arch::ENTRY_ADDRESS_SIZE, 0x0010_0000_0000_0000);
        assert_eq!(AArch64Arch::ENTRY_ADDRESS_MASK, 0x000F_FFFF_FFFF_F000);
        assert_eq!(AArch64Arch::ENTRY_FLAGS_MASK, 0xFFF0_0000_0000_0FFF);

        assert_eq!(AArch64Arch::PHYS_OFFSET, 0xFFFF_8000_0000_0000);
    }
}
