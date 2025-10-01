use std::path::PathBuf;

use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn no_directory() {
    cargo_bin_cmd!("test-driver").assert().failure();
}

#[test]
fn directory_which_doesnt_exist() {
    cargo_bin_cmd!("test-driver")
        .arg("--directory")
        .arg("this-directory-doesnt-exist-hopefully")
        .assert()
        .failure();
}

#[test]
fn directory_with_no_tests() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let test_directory = manifest_dir.join("src");

    cargo_bin_cmd!("test-driver")
        .arg("--directory")
        .arg(test_directory)
        .assert()
        .failure();
}

#[test]
fn missing_output() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let test_directory = manifest_dir.join("tests/missing_output");

    cargo_bin_cmd!("test-driver")
        .arg("--directory")
        .arg(test_directory)
        .assert()
        .failure();
}

#[test]
fn missing_run_binary() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let test_directory = manifest_dir.join("tests/missing_binary");

    cargo_bin_cmd!("test-driver")
        .arg("--directory")
        .arg(test_directory)
        .assert()
        .failure();
}

#[test]
fn missing_run() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let test_directory = manifest_dir.join("tests/missing_run");

    cargo_bin_cmd!("test-driver")
        .arg("--directory")
        .arg(test_directory)
        .assert()
        .failure();
}

#[test]
fn output_mismatch() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let test_directory = manifest_dir.join("tests/output_mismatch");

    cargo_bin_cmd!("test-driver")
        .arg("--directory")
        .arg(test_directory)
        .assert()
        .failure();
}

#[test]
fn executable_not_found() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let test_directory = manifest_dir.join("tests/executable_not_found");

    cargo_bin_cmd!("test-driver")
        .arg("--directory")
        .arg(test_directory)
        .assert()
        .failure();
}

#[test]
fn unexpected_pass() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let test_directory = manifest_dir.join("tests/unexpected_pass");

    cargo_bin_cmd!("test-driver")
        .arg("--directory")
        .arg(test_directory)
        .assert()
        .failure();
}
