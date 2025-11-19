
mod helpers;

use helpers::get_binary;
use std::process::Command;

#[test]
fn health_endpoint_works() {
    let exe = get_binary();
    let output = Command::new(exe)
        .arg("--health-stdout")
        .output()
        .expect("failed to start smem-exporter");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ok"));
}
