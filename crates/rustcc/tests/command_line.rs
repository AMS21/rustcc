use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn command_line_no_arguments() {
    cargo_bin_cmd!("rustcc").assert().failure();
}

#[test]
fn command_line_help() {
    cargo_bin_cmd!("rustcc").arg("--help").assert().success();
    cargo_bin_cmd!("rustcc").arg("-h").assert().success();
}

#[test]
fn command_line_version() {
    cargo_bin_cmd!("rustcc").arg("--version").assert().success();
    cargo_bin_cmd!("rustcc").arg("-V").assert().success();
}
