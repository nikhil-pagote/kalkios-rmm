use core::marker::PhantomData;

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
    pub unsafe fn from_data(data: usize) -> Self {
        Self { data, phantom: PhantomData }
    }

    #[inline(always)]
    pub fn data(&self) -> usize {
        self.data
    }

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
    pub fn user(self, value: bool) -> Self {
        self.custom_flag(A::ENTRY_FLAG_USER, value)
    }

    #[inline(always)]
    pub fn write(self, value: bool) -> Self {
        // Architecture may use readonly or readwrite, support either
        self.custom_flag(A::ENTRY_FLAG_READONLY, !value)
            .custom_flag(A::ENTRY_FLAG_READWRITE, value)
    }

    #[inline(always)]
    pub fn execute(self, value: bool) -> Self {
        //TODO: write xor execute?
        // Architecture may use no exec or exec, support either
        self.custom_flag(A::ENTRY_FLAG_NO_EXEC, !value)
            .custom_flag(A::ENTRY_FLAG_EXEC, value)
    }
}
