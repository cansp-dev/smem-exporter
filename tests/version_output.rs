
mod helpers;
use helpers::find_binary;
use std::process::Command;

#[test]
fn version_output() {
    let exe = find_binary();
    let output = Command::new(exe).arg("--version").output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("smem-exporter"));
}
