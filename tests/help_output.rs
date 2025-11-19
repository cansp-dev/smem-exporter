mod helpers;
use helpers::find_binary;
use std::process::Command;

#[test]
fn help_output() {
    let exe = find_binary();
    let output = Command::new(exe)
        .arg("--help")
        .output()
        .expect("failed to run smem_exporter --help");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("smem_exporter"));
    assert!(stdout.contains("--listen") || stdout.contains("-l"));
}
