
mod helpers;
use helpers::find_binary;
use std::process::Command;

#[test]
fn health_endpoint_works() {
    let exe = find_binary();
    let output = Command::new(exe).arg("--health-check").output().unwrap();
    assert!(output.status.success());
}
