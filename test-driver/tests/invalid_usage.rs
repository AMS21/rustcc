use assert_cmd::prelude::*;
use std::{path::PathBuf, process::Command};

#[test]
fn no_directory() {
    Command::cargo_bin("test-driver")
        .unwrap()
        .assert()
        .failure();
}

#[test]
fn directory_which_doesnt_exist() {
    Command::cargo_bin("test-driver")
        .unwrap()
        .arg("--directory")
        .arg("this-directory-doesnt-exist-hopefully")
        .assert()
        .failure();
}

#[test]
fn directory_with_no_tests() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let test_directory = manifest_dir.join("src");

    Command::cargo_bin("test-driver")
        .unwrap()
        .arg("--directory")
        .arg(test_directory)
        .assert()
        .failure();
}

#[test]
fn missing_output() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let test_directory = manifest_dir.join("tests/missing_output");

    Command::cargo_bin("test-driver")
        .unwrap()
        .arg("--directory")
        .arg(test_directory)
        .assert()
        .failure();
}

#[test]
fn missing_run_binary() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let test_directory = manifest_dir.join("tests/missing_binary");

    Command::cargo_bin("test-driver")
        .unwrap()
        .arg("--directory")
        .arg(test_directory)
        .assert()
        .failure();
}

#[test]
fn missing_run() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let test_directory = manifest_dir.join("tests/missing_run");

    Command::cargo_bin("test-driver")
        .unwrap()
        .arg("--directory")
        .arg(test_directory)
        .assert()
        .failure();
}

#[test]
fn output_mismatch() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let test_directory = manifest_dir.join("tests/output_mismatch");

    Command::cargo_bin("test-driver")
        .unwrap()
        .arg("--directory")
        .arg(test_directory)
        .assert()
        .failure();
}

#[test]
fn executable_not_found() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let test_directory = manifest_dir.join("tests/executable_not_found");

    Command::cargo_bin("test-driver")
        .unwrap()
        .arg("--directory")
        .arg(test_directory)
        .assert()
        .failure();
}

#[test]
fn unexpected_pass() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let test_directory = manifest_dir.join("tests/unexpected_pass");

    Command::cargo_bin("test-driver")
        .unwrap()
        .arg("--directory")
        .arg(test_directory)
        .assert()
        .failure();
}
