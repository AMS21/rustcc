use crate::{
    basic_block::LLVMBasicBlock,
    ffi::{LLVMAddIncoming, LLVMBasicBlockRef, LLVMValueRef},
    value::LLVMValue,
};

#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct LLVMPhi(LLVMValue);

impl LLVMPhi {
    #[must_use]
    pub(crate) fn from_raw(raw: LLVMValueRef) -> Self {
        Self(LLVMValue::from_raw(raw))
    }

    #[must_use]
    pub(crate) fn as_raw(&self) -> LLVMValueRef {
        self.0.as_raw()
    }

    #[expect(clippy::as_ptr_cast_mut)]
    pub fn add_incoming(&self, values: &[LLVMValue], blocks: &[LLVMBasicBlock]) {
        assert!(
            values.len() == blocks.len(),
            "Values and blocks must have the same length"
        );
        #[expect(clippy::cast_possible_truncation)]
        let count = values.len() as u32;

        assert!(count > 0, "Cannot add zero incoming values to a PHI node");

        let values_ptr = values.as_ptr() as *mut LLVMValueRef;
        let blocks_ptr = blocks.as_ptr() as *mut LLVMBasicBlockRef;

        // Safety: LLVMAddIncoming is safe to call with valid parameters.
        unsafe { LLVMAddIncoming(self.as_raw(), values_ptr, blocks_ptr, count) };
    }

    #[must_use]
    pub const fn value(&self) -> LLVMValue {
        self.0
    }
}
