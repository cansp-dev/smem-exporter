
mod helpers;

use helpers::get_binary;
use std::process::Command;

#[test]
fn version_output() {
    let exe = get_binary();
    let output = Command::new(exe)
        .arg("--version")
        .output()
        .expect("failed to start smem-exporter");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.starts_with("smem-exporter "));
    assert!(stdout.contains(env!("CARGO_PKG_VERSION")));
}
