use assert_cmd::{assert::OutputAssertExt, cargo::CommandCargoExt};
use std::{path::PathBuf, process::Command};

#[test]
fn test_driver() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let test_directory = manifest_dir.join("tests");

    Command::cargo_bin("test-driver")
        .unwrap()
        .arg("--directory")
        .arg(test_directory)
        .assert()
        .success();
}
