use crate::{
    basic_block::LLVMBasicBlock,
    ffi::{
        LLVMAppendBasicBlockInContext, LLVMContextCreate, LLVMContextDispose, LLVMContextRef,
        LLVMCreateBasicBlockInContext, LLVMInt1TypeInContext, LLVMInt8TypeInContext,
        LLVMInt16TypeInContext, LLVMInt32TypeInContext, LLVMInt64TypeInContext,
        LLVMInt128TypeInContext,
    },
    function::LLVMFunctionValue,
    to_cstring,
    typ::LLVMType,
};

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct LLVMContext(LLVMContextRef);

impl LLVMContext {
    #[must_use]
    pub fn new() -> Option<Self> {
        // Safety: LLVMContextCreate is safe to call and returns a valid LLVMContextRef
        // or null.
        let context = unsafe { LLVMContextCreate() };
        if context.is_null() {
            None
        } else {
            Some(Self(context))
        }
    }

    pub(crate) fn as_raw(&self) -> LLVMContextRef {
        assert!(!self.0.is_null(), "LLVMContext contains null pointer");
        self.0
    }

    #[must_use]
    pub fn bool_type(&self) -> LLVMType {
        // Safety: LLVMInt1TypeInContext is safe to call with a valid LLVMContextRef.
        LLVMType::from_raw(unsafe { LLVMInt1TypeInContext(self.0) })
    }

    #[must_use]
    pub fn int8_type(&self) -> LLVMType {
        // Safety: LLVMInt8TypeInContext is safe to call with a valid LLVMContextRef.
        LLVMType::from_raw(unsafe { LLVMInt8TypeInContext(self.0) })
    }

    #[must_use]
    pub fn int16_type(&self) -> LLVMType {
        // Safety: LLVMInt16TypeInContext is safe to call with a valid LLVMContextRef.
        LLVMType::from_raw(unsafe { LLVMInt16TypeInContext(self.0) })
    }

    #[must_use]
    pub fn int32_type(&self) -> LLVMType {
        // Safety: LLVMInt32TypeInContext is safe to call with a valid LLVMContextRef.
        LLVMType::from_raw(unsafe { LLVMInt32TypeInContext(self.0) })
    }

    #[must_use]
    pub fn int64_type(&self) -> LLVMType {
        // Safety: LLVMInt64TypeInContext is safe to call with a valid LLVMContextRef.
        LLVMType::from_raw(unsafe { LLVMInt64TypeInContext(self.0) })
    }

    #[must_use]
    pub fn int128_type(&self) -> LLVMType {
        // Safety: LLVMInt128TypeInContext is safe to call with a valid LLVMContextRef.
        LLVMType::from_raw(unsafe { LLVMInt128TypeInContext(self.0) })
    }

    /// Create a basic block for an existing function.
    ///
    /// The function must have been created in (or otherwise belong to) this
    /// context.
    #[must_use]
    pub fn create_basic_block_for_function<'a, S: Into<&'a str>>(
        &self,
        function: &LLVMFunctionValue,
        name: S,
    ) -> LLVMBasicBlock {
        let name = to_cstring(name.into());
        // Safety: LLVMAppendBasicBlockInContext is safe to call with valid parameters.
        LLVMBasicBlock::from_raw(unsafe {
            LLVMAppendBasicBlockInContext(self.0, function.as_raw(), name.as_ptr())
        })
    }

    pub fn create_basic_block<'a, S: Into<&'a str>>(&self, name: S) -> LLVMBasicBlock {
        let name = to_cstring(name.into());
        // Safety: LLVMCreateBasicBlockInContext is safe to call with valid parameters.
        LLVMBasicBlock::from_raw(unsafe { LLVMCreateBasicBlockInContext(self.0, name.as_ptr()) })
    }
}

impl Drop for LLVMContext {
    fn drop(&mut self) {
        // Safety: LLVMContextDispose is safe to call with a valid LLVMContextRef.
        unsafe { LLVMContextDispose(self.0) };
    }
}
