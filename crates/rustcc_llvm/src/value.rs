use crate::ffi::{LLVMIsAInstruction, LLVMValueRef};

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct LLVMValue(LLVMValueRef);

impl LLVMValue {
    pub(crate) fn from_raw(raw: LLVMValueRef) -> Self {
        assert!(
            !raw.is_null(),
            "Attempted to create LLVMValue from null pointer"
        );
        Self(raw)
    }

    pub(crate) fn as_raw(&self) -> LLVMValueRef {
        assert!(!self.0.is_null(), "LLVMValue contains null pointer");
        self.0
    }

    #[must_use]
    pub fn is_instruction(&self) -> bool {
        // Safety: LLVMIsAInstruction is safe to call with a valid LLVMValueRef.
        unsafe { !LLVMIsAInstruction(self.as_raw()).is_null() }
    }
}
