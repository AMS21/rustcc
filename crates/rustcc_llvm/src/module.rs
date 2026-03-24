use std::ffi::CStr;

use crate::{
    context::LLVMContext,
    ffi::{
        LLVMAddFunction, LLVMDisposeMessage, LLVMDisposeModule, LLVMModuleCreateWithNameInContext,
        LLVMModuleRef, LLVMPrintModuleToString,
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
        // Safety: LLVMPrintModuleToString is safe to call with a valid LLVMModuleRef.
        let dump_string = unsafe { LLVMPrintModuleToString(self.0) };

        // Safety: The returned pointer from LLVMPrintModuleToString is valid for
        // reading until LLVMDisposeMessage is called.
        let rust_string = unsafe { CStr::from_ptr(dump_string.cast_const()) }.to_string_lossy();
        print!("{rust_string}");

        // Safety: We must dispose of the message returned by LLVM to avoid memory
        // leaks.
        unsafe { LLVMDisposeMessage(dump_string) };
    }
}

impl Drop for LLVMModule {
    fn drop(&mut self) {
        // Safety: LLVMDisposeModule is safe to call with a valid LLVMModuleRef.
        unsafe { LLVMDisposeModule(self.0) };
    }
}
