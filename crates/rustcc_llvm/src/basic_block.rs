use crate::{
    ffi::{LLVMBasicBlockRef, LLVMGetBasicBlockParent},
    function::LLVMFunctionValue,
};

#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct LLVMBasicBlock(LLVMBasicBlockRef);

impl LLVMBasicBlock {
    pub(crate) fn from_raw(raw: LLVMBasicBlockRef) -> Self {
        assert!(
            !raw.is_null(),
            "Attempted to create LLVMBasicBlock from null pointer"
        );
        Self(raw)
    }

    #[must_use]
    pub(crate) fn as_raw(&self) -> LLVMBasicBlockRef {
        assert!(!self.0.is_null(), "LLVMBasicBlock contains null pointer");
        self.0
    }

    #[must_use]
    pub fn get_parent(&self) -> Option<LLVMFunctionValue> {
        // Safety: LLVMGetBasicBlockParent is safe to call with a valid
        // LLVMBasicBlockRef.
        let func = unsafe { LLVMGetBasicBlockParent(self.as_raw()) };
        if func.is_null() {
            None
        } else {
            Some(LLVMFunctionValue::from_raw(func))
        }
    }
}
