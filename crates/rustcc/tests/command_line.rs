use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn command_line_no_arguments() {
    cargo_bin_cmd!("rustcc").assert().failure();
}
