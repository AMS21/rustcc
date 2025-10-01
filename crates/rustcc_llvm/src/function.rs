use std::ptr;

use crate::{
    basic_block::LLVMBasicBlock,
    ffi::{LLVMAppendExistingBasicBlock, LLVMGetLastBasicBlock, LLVMTypeRef, LLVMValueRef},
    typ::LLVMType,
    value::LLVMValue,
};

#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct LLVMFunctionType(LLVMType);

impl LLVMFunctionType {
    #[must_use]
    pub(crate) fn from_raw(raw: LLVMTypeRef) -> Self {
        Self(LLVMType::from_raw(raw))
    }

    #[must_use]
    pub(crate) fn as_raw(&self) -> LLVMTypeRef {
        self.0.as_raw()
    }

    #[must_use]
    pub const fn typ(&self) -> LLVMType {
        self.0
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct LLVMFunctionValue(LLVMValue);

impl LLVMFunctionValue {
    #[must_use]
    pub(crate) fn from_raw(raw: LLVMValueRef) -> Self {
        Self(LLVMValue::from_raw(raw))
    }

    #[must_use]
    pub(crate) fn as_raw(&self) -> LLVMValueRef {
        self.0.as_raw()
    }

    pub fn append_existing_basic_block(&self, basic_block: LLVMBasicBlock) {
        // Safety: LLVMAppendExistingBasicBlock is safe to call with valid parameters.
        unsafe { LLVMAppendExistingBasicBlock(self.as_raw(), basic_block.as_raw()) };
    }

    #[must_use]
    pub fn last_basic_block(&self) -> Option<LLVMBasicBlock> {
        // Safety: LLVMGetLastBasicBlock is safe to call with valid parameters.
        let basic_block = unsafe { LLVMGetLastBasicBlock(self.as_raw()) };
        if basic_block.is_null() {
            None
        } else {
            Some(LLVMBasicBlock::from_raw(basic_block))
        }
    }

    #[must_use]
    pub const fn value(&self) -> LLVMValue {
        self.0
    }
}

#[must_use]
pub fn function_type(return_type: LLVMType) -> LLVMFunctionType {
    // SAFETY: The caller must ensure that `return_type` is a valid LLVM type.
    LLVMFunctionType::from_raw(unsafe {
        crate::ffi::LLVMFunctionType(return_type.as_raw(), ptr::null_mut(), 0, 0)
    })
}
