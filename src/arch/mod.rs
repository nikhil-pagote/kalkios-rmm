use core::ptr;

use crate::MemoryArea;

pub mod emulate;
pub mod x86_64;

pub trait Arch {
    const PAGE_SHIFT: usize;
    const PAGE_ENTRY_SHIFT: usize;
    const PAGE_LEVELS: usize;

    const ENTRY_ADDRESS_SHIFT: usize;
    const ENTRY_FLAG_PRESENT: usize;
    const ENTRY_FLAG_WRITABLE: usize;
    const ENTRY_FLAG_USER: usize;
    const ENTRY_FLAG_HUGE: usize;
    const ENTRY_FLAG_GLOBAL: usize;
    const ENTRY_FLAG_NO_EXEC: usize;

    const PAGE_SIZE: usize = 1 << Self::PAGE_SHIFT;
    const PAGE_OFFSET_MASK: usize = Self::PAGE_SIZE - 1;
    const PAGE_ADDRESS_SHIFT: usize = Self::PAGE_LEVELS * Self::PAGE_ENTRY_SHIFT + Self::PAGE_SHIFT;
    const PAGE_ADDRESS_SIZE: usize = 1 << Self::PAGE_ADDRESS_SHIFT;
    const PAGE_ADDRESS_MASK: usize = Self::PAGE_ADDRESS_SIZE - Self::PAGE_SIZE;
    const PAGE_ENTRY_SIZE: usize = 1 << (Self::PAGE_SHIFT - Self::PAGE_ENTRY_SHIFT);
    const PAGE_ENTRIES: usize = 1 << Self::PAGE_ENTRY_SHIFT;
    const PAGE_ENTRY_MASK: usize = Self::PAGE_ENTRIES - 1;

    const ENTRY_ADDRESS_SIZE: usize = 1 << Self::ENTRY_ADDRESS_SHIFT;
    const ENTRY_ADDRESS_MASK: usize = Self::ENTRY_ADDRESS_SIZE - Self::PAGE_SIZE;
    const ENTRY_FLAGS_MASK: usize = !Self::ENTRY_ADDRESS_MASK;

    unsafe fn init() -> &'static [MemoryArea];

    #[inline(always)]
    unsafe fn read<T>(address: usize) -> T {
        ptr::read(address as *const T)
    }

    #[inline(always)]
    unsafe fn write<T>(address: usize, value: T) {
        ptr::write(address as *mut T, value)
    }

    unsafe fn invalidate(address: usize);

    unsafe fn invalidate_all();

    unsafe fn table() -> usize;

    unsafe fn set_table(address: usize);
}
