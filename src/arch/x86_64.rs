use crate::{
    Arch,
    MemoryArea,
    PhysicalAddress,
    VirtualAddress,
};

#[derive(Clone, Copy)]
pub struct X8664Arch;

impl Arch for X8664Arch {
    const PAGE_SHIFT: usize = 12; // 4096 bytes
    const PAGE_ENTRY_SHIFT: usize = 9; // 512 entries, 8 bytes each
    const PAGE_LEVELS: usize = 4; // PML4, PDP, PD, PT

    const ENTRY_ADDRESS_SHIFT: usize = 52;
    const ENTRY_FLAG_PRESENT: usize = 1 << 0;
    const ENTRY_FLAG_WRITABLE: usize = 1 << 1;
    const ENTRY_FLAG_USER: usize = 1 << 2;
    const ENTRY_FLAG_HUGE: usize = 1 << 7;
    const ENTRY_FLAG_GLOBAL: usize = 1 << 8;
    const ENTRY_FLAG_NO_EXEC: usize = 1 << 63;

    const PHYS_OFFSET: usize = Self::PAGE_NEGATIVE_MASK + (Self::PAGE_ADDRESS_SIZE >> 1); // PML4 slot 256 and onwards

    unsafe fn init() -> &'static [MemoryArea] {
        unimplemented!("X8664Arch::init unimplemented");
    }

    #[inline(always)]
    unsafe fn invalidate(address: VirtualAddress) {
        asm!("invlpg [{0}]", in(reg) address.data());
    }

    #[inline(always)]
    unsafe fn table() -> PhysicalAddress {
        let address: usize;
        asm!("mov {0}, cr3", out(reg) address);
        PhysicalAddress::new(address)
    }

    #[inline(always)]
    unsafe fn set_table(address: PhysicalAddress) {
        asm!("mov cr3, {0}", in(reg) address.data());
    }
}

#[cfg(test)]
mod tests {
    use crate::Arch;
    use super::X8664Arch;

    #[test]
    fn constants() {
        assert_eq!(X8664Arch::PAGE_SIZE, 4096);
        assert_eq!(X8664Arch::PAGE_OFFSET_MASK, 0xFFF);
        assert_eq!(X8664Arch::PAGE_ADDRESS_SHIFT, 48);
        assert_eq!(X8664Arch::PAGE_ADDRESS_SIZE, 0x0001_0000_0000_0000);
        assert_eq!(X8664Arch::PAGE_ADDRESS_MASK, 0x0000_FFFF_FFFF_F000);
        assert_eq!(X8664Arch::PAGE_ENTRY_SIZE, 8);
        assert_eq!(X8664Arch::PAGE_ENTRIES, 512);
        assert_eq!(X8664Arch::PAGE_ENTRY_MASK, 0x1FF);
        assert_eq!(X8664Arch::PAGE_NEGATIVE_MASK, 0xFFFF_0000_0000_0000);

        assert_eq!(X8664Arch::ENTRY_ADDRESS_SIZE, 0x0010_0000_0000_0000);
        assert_eq!(X8664Arch::ENTRY_ADDRESS_MASK, 0x000F_FFFF_FFFF_F000);
        assert_eq!(X8664Arch::ENTRY_FLAGS_MASK, 0xFFF0_0000_0000_0FFF);

        assert_eq!(X8664Arch::PHYS_OFFSET, 0xFFFF_8000_0000_0000);
    }
}
