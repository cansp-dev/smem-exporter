
mod helpers;
use helpers::find_binary;
use std::process::Command;

#[test]
fn help_output() {
    let exe = find_binary();
    let output = Command::new(exe).arg("--help").output().unwrap();
    assert!(output.status.success());
}
