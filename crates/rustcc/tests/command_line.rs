use assert_cmd::Command;

#[test]
fn command_line_no_arguments() {
    Command::cargo_bin("rustcc").unwrap().assert().failure();
}
