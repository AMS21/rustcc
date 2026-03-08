use std::{path::PathBuf, process::Command};

use assert_cmd::{assert::OutputAssertExt, cargo::CommandCargoExt};

#[test]
fn test_driver() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_dir = manifest_dir.parent().unwrap();

    #[expect(deprecated)]
    Command::cargo_bin("test-driver")
        .unwrap()
        .current_dir(workspace_dir)
        .arg("--directory")
        .arg("rustcc/tests")
        .assert()
        .success();
}
