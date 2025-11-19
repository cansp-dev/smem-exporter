#[test]
fn version_output() {
    let output = std::process::Command::new("target/debug/smem-exporter")
        .arg("--version")
        .output()
        .expect("Failed to execute smem_exporter");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("smem_exporter"));
    assert!(stdout.contains('v'));
}
