use crate::{
    ffi::{LLVMBool, LLVMConstInt, LLVMTypeRef},
    value::LLVMValue,
};

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct LLVMType(LLVMTypeRef);

impl LLVMType {
    pub(crate) fn from_raw(raw: LLVMTypeRef) -> Self {
        assert!(
            !raw.is_null(),
            "Attempted to create LLVMType from null pointer"
        );
        Self(raw)
    }

    pub(crate) fn as_raw(&self) -> LLVMTypeRef {
        assert!(!self.0.is_null(), "LLVMType contains null pointer");
        self.0
    }

    #[must_use]
    pub fn constant_integer(&self, value: u64, sign_extend: bool) -> LLVMValue {
        // SAFETY: LLVMConstInt creates a constant integer value of the specified type.
        // The caller must ensure that `typ` is a valid integer type.
        LLVMValue::from_raw(unsafe {
            LLVMConstInt(self.as_raw(), value, LLVMBool::from(sign_extend))
        })
    }
}
