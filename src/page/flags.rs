use core::{
    fmt,
    marker::PhantomData
};

use crate::Arch;

#[derive(Clone, Copy)]
pub struct PageFlags<A> {
    data: usize,
    phantom: PhantomData<A>,
}

impl<A: Arch> PageFlags<A> {
    #[inline(always)]
    pub fn new() -> Self {
        unsafe {
            Self::from_data(
                // Flags set to present, kernel space, read-only, no-execute by default
                A::ENTRY_FLAG_DEFAULT_PAGE |
                A::ENTRY_FLAG_READONLY |
                A::ENTRY_FLAG_NO_EXEC
            )
        }
    }

    #[inline(always)]
    pub fn new_table() -> Self {
        unsafe {
            Self::from_data(
                // Flags set to present, kernel space, read-only, no-execute by default
                A::ENTRY_FLAG_DEFAULT_TABLE |
                A::ENTRY_FLAG_READONLY |
                A::ENTRY_FLAG_NO_EXEC
            )
        }
    }

    #[inline(always)]
    pub unsafe fn from_data(data: usize) -> Self {
        Self { data, phantom: PhantomData }
    }

    #[inline(always)]
    pub fn data(&self) -> usize {
        self.data
    }

    #[must_use]
    #[inline(always)]
    pub fn custom_flag(mut self, flag: usize, value: bool) -> Self {
        if value {
            self.data |= flag;
        } else {
            self.data &= !flag;
        }
        self
    }

    #[inline(always)]
    pub fn has_flag(&self, flag: usize) -> bool {
        self.data & flag == flag
    }

    #[inline(always)]
    pub fn has_present(&self) -> bool {
        self.has_flag(A::ENTRY_FLAG_PRESENT)
    }

    #[must_use]
    #[inline(always)]
    pub fn user(self, value: bool) -> Self {
        self.custom_flag(A::ENTRY_FLAG_USER, value)
    }

    #[inline(always)]
    pub fn has_user(&self) -> bool {
        self.has_flag(A::ENTRY_FLAG_USER)
    }

    #[must_use]
    #[inline(always)]
    pub fn write(self, value: bool) -> Self {
        // Architecture may use readonly or readwrite, support either
        self.custom_flag(A::ENTRY_FLAG_READONLY, !value)
            .custom_flag(A::ENTRY_FLAG_READWRITE, value)
    }

    #[inline(always)]
    pub fn has_write(&self) -> bool {
        // Architecture may use readonly or readwrite, support either
        self.data & (A::ENTRY_FLAG_READONLY | A::ENTRY_FLAG_READWRITE) == A::ENTRY_FLAG_READWRITE
    }

    #[must_use]
    #[inline(always)]
    pub fn execute(self, value: bool) -> Self {
        //TODO: write xor execute?
        // Architecture may use no exec or exec, support either
        self.custom_flag(A::ENTRY_FLAG_NO_EXEC, !value)
            .custom_flag(A::ENTRY_FLAG_EXEC, value)
    }

    #[inline(always)]
    pub fn has_execute(&self) -> bool {
        // Architecture may use no exec or exec, support either
        self.data & (A::ENTRY_FLAG_NO_EXEC | A::ENTRY_FLAG_EXEC) == A::ENTRY_FLAG_EXEC
    }
}

impl<A: Arch> fmt::Debug for PageFlags<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PageFlags")
            .field("data", &self.data)
            .finish()
    }
}