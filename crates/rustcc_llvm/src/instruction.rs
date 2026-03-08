use crate::{ffi::LLVMValueRef, value::LLVMValue};

#[derive(Debug)]
#[repr(transparent)]
pub struct LLVMInstruction(LLVMValue);

impl LLVMInstruction {
    pub(crate) fn from_raw(value: LLVMValueRef) -> Self {
        let value = LLVMValue::from_raw(value);
        debug_assert!(
            value.is_instruction(),
            "Attempted to create LLVMInstruction from non-instruction value"
        );

        Self(value)
    }
}
