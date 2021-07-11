use crate::{
    Arch,
    MemoryArea,
    PhysicalAddress,
    TableKind,
    VirtualAddress,
};

#[derive(Clone, Copy)]
pub struct AArch64Arch;

impl Arch for AArch64Arch {
    const PAGE_SHIFT: usize = 12; // 4096 bytes
    const PAGE_ENTRY_SHIFT: usize = 9; // 512 entries, 8 bytes each
    const PAGE_LEVELS: usize = 4; // L0, L1, L2, L3

    //TODO
    const ENTRY_ADDRESS_SHIFT: usize = 52;
    const ENTRY_FLAG_DEFAULT_PAGE: usize
        = Self::ENTRY_FLAG_PRESENT
        | 1 << 1 // Page flag
        | 1 << 10 // Access flag
        ;
    const ENTRY_FLAG_DEFAULT_TABLE: usize
        = Self::ENTRY_FLAG_PRESENT
        | 1 << 1 // Table flag
        | 1 << 10 // Access flag
        ;
    const ENTRY_FLAG_PRESENT: usize = 1 << 0;
    const ENTRY_FLAG_READONLY: usize = 1 << 7;
    const ENTRY_FLAG_READWRITE: usize = 0;
    const ENTRY_FLAG_USER: usize = 1 << 6;
    // This sets both userspace and privileged execute never
    //TODO: Separate the two?
    const ENTRY_FLAG_NO_EXEC: usize = 0b11 << 53;
    const ENTRY_FLAG_EXEC: usize = 0;

    //TODO: adjust to match x86_64?
    const PHYS_OFFSET: usize = 0xfffffe0000000000;

    unsafe fn init() -> &'static [MemoryArea] {
        unimplemented!("AArch64Arch::init unimplemented");
    }

    #[inline(always)]
    unsafe fn invalidate(address: VirtualAddress) {
        //TODO: can one address be invalidated?
        Self::invalidate_all();
    }

    #[inline(always)]
    unsafe fn invalidate_all() {
        asm!("tlbi vmalle1is");
    }

    #[inline(always)]
    unsafe fn table() -> PhysicalAddress {
        let address: usize;
        //TODO: set this dynamically
        let table_kind = TableKind::Kernel;
        match table_kind {
            TableKind::User => {
                asm!("mrs {0}, ttbr0_el1", out(reg) address);
            },
            TableKind::Kernel => {
                asm!("mrs {0}, ttbr1_el1", out(reg) address);
            }
        }
        PhysicalAddress::new(address)
    }

    #[inline(always)]
    unsafe fn set_table(address: PhysicalAddress) {
        //TODO: set this dynamically
        let table_kind = TableKind::Kernel;
        match table_kind {
            TableKind::User => {
                asm!("msr ttbr0_el1, {0}", in(reg) address.data());
            },
            TableKind::Kernel => {
                asm!("msr ttbr1_el1, {0}", in(reg) address.data());
            }
        }
        //TODO: Does this need to be called?
        Self::invalidate_all();
    }

    fn virt_is_valid(address: VirtualAddress) -> bool {
        // FIXME
        true
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

        assert_eq!(AArch64Arch::PHYS_OFFSET, 0xFFFF_FE00_0000_0000);
    }
}
