use crate::{
    context::LLVMContext,
    ffi::{
        LLVMAddFunction, LLVMDisposeModule, LLVMDumpModule, LLVMModuleCreateWithNameInContext,
        LLVMModuleRef,
    },
    function::{LLVMFunctionType, LLVMFunctionValue},
    to_cstring,
};

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct LLVMModule(LLVMModuleRef);

impl LLVMModule {
    pub fn new_in_context<'a, S: Into<&'a str>>(name: S, context: &LLVMContext) -> Option<Self> {
        let name = to_cstring(name.into());
        // Safety: LLVMModuleCreateWithNameInContext is safe to call with valid
        // parameters.
        let module = unsafe { LLVMModuleCreateWithNameInContext(name.as_ptr(), context.as_raw()) };
        if module.is_null() {
            None
        } else {
            Some(Self(module))
        }
    }

    #[must_use]
    pub fn add_function<'a, S: Into<&'a str>>(
        &self,
        name: S,
        function_type: LLVMFunctionType,
    ) -> LLVMFunctionValue {
        let name = to_cstring(name.into());
        // Safety: LLVMAddFunction is safe to call with valid parameters.
        LLVMFunctionValue::from_raw(unsafe {
            LLVMAddFunction(self.0, name.as_ptr(), function_type.as_raw())
        })
    }

    pub fn dump(&self) {
        // Safety: LLVMDumpModule is safe to call with a valid LLVMModuleRef.
        unsafe { LLVMDumpModule(self.0) };
    }
}

impl Drop for LLVMModule {
    fn drop(&mut self) {
        // Safety: LLVMDisposeModule is safe to call with a valid LLVMModuleRef.
        unsafe { LLVMDisposeModule(self.0) };
    }
}
