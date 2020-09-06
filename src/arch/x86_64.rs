use core::mem;

//TODO: should these be constants?
pub const PAGE_SHIFT: usize = 12; // 4096 bytes
pub const PAGE_SIZE: usize = 1 << PAGE_SHIFT;
pub const PAGE_OFFSET_MASK: usize = PAGE_SIZE - 1;
pub const PAGE_ADDRESS_MASK: usize = !PAGE_OFFSET_MASK;
pub const PAGE_ENTRY_SIZE: usize = mem::size_of::<usize>();
pub const PAGE_ENTRIES: usize = PAGE_SIZE / PAGE_ENTRY_SIZE;
pub const PAGE_ENTRY_SHIFT: usize = 9; // 512 entries
pub const PAGE_ENTRY_MASK: usize = (1 << PAGE_ENTRY_SHIFT) - 1;
pub const PAGE_LEVELS: usize = 4; // PML4, PDP, PD, PT

pub const ENTRY_FLAG_PRESENT: usize = 1 << 0;
pub const ENTRY_FLAG_WRITABLE: usize = 1 << 1;
pub const ENTRY_FLAG_USER: usize = 1 << 2;
pub const ENTRY_FLAG_HUGE: usize = 1 << 7;
pub const ENTRY_FLAG_GLOBAL: usize = 1 << 8;
pub const ENTRY_FLAG_NO_EXEC: usize = 1 << 63;
pub const ENTRY_ADDRESS_MASK: usize = PAGE_ADDRESS_MASK;
pub const ENTRY_FLAGS_MASK: usize = !ENTRY_ADDRESS_MASK;
