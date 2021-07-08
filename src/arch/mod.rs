use core::ptr;

use crate::{
    MemoryArea,
    PhysicalAddress,
    TableKind,
    VirtualAddress,
};

pub use self::aarch64::AArch64Arch;
pub use self::riscv64::{
    RiscV64Sv39Arch,
    RiscV64Sv48Arch
};
#[cfg(feature = "std")]
pub use self::emulate::EmulateArch;
pub use self::x86_64::X8664Arch;

mod aarch64;
mod riscv64;
#[cfg(feature = "std")]
mod emulate;
mod x86_64;

pub trait Arch: Clone + Copy {
    const PAGE_SHIFT: usize;
    const PAGE_ENTRY_SHIFT: usize;
    const PAGE_LEVELS: usize;

    const ENTRY_ADDRESS_SHIFT: usize;
    const ENTRY_FLAG_DEFAULT_PAGE: usize;
    const ENTRY_FLAG_DEFAULT_TABLE: usize;
    const ENTRY_FLAG_PRESENT: usize;
    const ENTRY_FLAG_READONLY: usize;
    const ENTRY_FLAG_READWRITE: usize;
    const ENTRY_FLAG_USER: usize;
    const ENTRY_FLAG_NO_EXEC: usize;
    const ENTRY_FLAG_EXEC: usize;

    const PHYS_OFFSET: usize;

    const PAGE_SIZE: usize = 1 << Self::PAGE_SHIFT;
    const PAGE_OFFSET_MASK: usize = Self::PAGE_SIZE - 1;
    const PAGE_ADDRESS_SHIFT: usize = Self::PAGE_LEVELS * Self::PAGE_ENTRY_SHIFT + Self::PAGE_SHIFT;
    const PAGE_ADDRESS_SIZE: usize = 1 << Self::PAGE_ADDRESS_SHIFT;
    const PAGE_ADDRESS_MASK: usize = Self::PAGE_ADDRESS_SIZE - Self::PAGE_SIZE;
    const PAGE_ENTRY_SIZE: usize = 1 << (Self::PAGE_SHIFT - Self::PAGE_ENTRY_SHIFT);
    const PAGE_ENTRIES: usize = 1 << Self::PAGE_ENTRY_SHIFT;
    const PAGE_ENTRY_MASK: usize = Self::PAGE_ENTRIES - 1;
    const PAGE_NEGATIVE_MASK: usize = !(Self::PAGE_ADDRESS_SIZE - 1);

    const ENTRY_ADDRESS_SIZE: usize = 1 << Self::ENTRY_ADDRESS_SHIFT;
    const ENTRY_ADDRESS_MASK: usize = Self::ENTRY_ADDRESS_SIZE - Self::PAGE_SIZE;
    const ENTRY_FLAGS_MASK: usize = !Self::ENTRY_ADDRESS_MASK;

    unsafe fn init() -> &'static [MemoryArea];

    #[inline(always)]
    unsafe fn read<T>(address: VirtualAddress) -> T {
        ptr::read(address.data() as *const T)
    }

    #[inline(always)]
    unsafe fn write<T>(address: VirtualAddress, value: T) {
        ptr::write(address.data() as *mut T, value)
    }

    #[inline(always)]
    unsafe fn write_bytes(address: VirtualAddress, value: u8, count: usize) {
        ptr::write_bytes(address.data() as *mut u8, value, count)
    }

    unsafe fn invalidate(address: VirtualAddress);

    #[inline(always)]
    unsafe fn invalidate_all() {
        Self::set_table(Self::table());
    }

    unsafe fn table() -> PhysicalAddress;

    unsafe fn set_table(address: PhysicalAddress);

    #[inline(always)]
    unsafe fn phys_to_virt(phys: PhysicalAddress) -> VirtualAddress {
        VirtualAddress::new(phys.data() + Self::PHYS_OFFSET)
    }

    fn virt_is_valid(address: VirtualAddress) -> bool;
}
