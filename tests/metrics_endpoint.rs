
mod helpers;
use helpers::find_binary;
use std::process::Command;

#[test]
fn metrics_endpoint_works() {
    let exe = find_binary();
    let output = Command::new(exe).arg("--print-metrics").output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("smem_"));
}
