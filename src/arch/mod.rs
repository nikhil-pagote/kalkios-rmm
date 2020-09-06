use core::ptr;

use crate::MemoryArea;

pub mod emulate;
pub mod x86_64;

pub trait Arch {
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
