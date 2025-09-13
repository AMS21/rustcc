pub use crate::ffi::LLVMVerifierFailureAction;
use crate::{ffi::LLVMVerifyFunction, function::LLVMFunctionValue};

#[must_use]
pub fn verify_function(function: LLVMFunctionValue, action: LLVMVerifierFailureAction) -> bool {
    // Safety: LLVMVerifyFunction is safe to call with valid parameters.
    unsafe { LLVMVerifyFunction(function.as_raw(), action) == 0 }
}
