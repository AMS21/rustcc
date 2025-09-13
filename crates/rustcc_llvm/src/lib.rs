mod ffi;

pub mod analysis;
pub mod basic_block;
pub mod builder;
pub mod context;
pub mod function;
pub mod module;
pub mod phi;
pub mod typ;
pub mod value;

use ffi::{LLVM_VERSION_MAJOR, LLVM_VERSION_MINOR, LLVM_VERSION_PATCH};
use semver::Version;

pub const LLVM_VERSION: Version = Version::new(
    LLVM_VERSION_MAJOR as u64,
    LLVM_VERSION_MINOR as u64,
    LLVM_VERSION_PATCH as u64,
);

#[must_use]
#[expect(clippy::expect_used)]
fn to_cstring<'a, S: Into<&'a str>>(string: S) -> std::ffi::CString {
    std::ffi::CString::new(string.into()).expect("Invalid string")
}

#[cfg(test)]
mod tests {

    use semver::Version;

    use super::LLVM_VERSION;
    use crate::ffi::LLVMGetVersion;

    #[test]
    fn test_llvm_version() {
        // Just a basic test to ensure the LLVM version is as expected.
        assert!(LLVM_VERSION >= semver::Version::new(5, 0, 0));

        // Get the version from LLVM
        let mut llvm_major = 0;
        let mut llvm_minor = 0;
        let mut llvm_patch = 0;
        // Safety: LLVMGetVersion is safe to call with valid pointers.
        unsafe {
            LLVMGetVersion(
                &raw mut llvm_major,
                &raw mut llvm_minor,
                &raw mut llvm_patch,
            );
        }
        let llvm_version = Version::new(
            u64::from(llvm_major),
            u64::from(llvm_minor),
            u64::from(llvm_patch),
        );

        assert_eq!(LLVM_VERSION, llvm_version);
    }
}
