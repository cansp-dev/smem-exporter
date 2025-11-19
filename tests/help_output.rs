use std::process::Command;

#[test]
fn help_flag_works() {
    let exe = env!("CARGO_BIN_EXE_smem_exporter");

    let output = Command::new(exe)
        .arg("--help")
        .output()
        .expect("failed to run --help");

    assert!(
        output.status.success(),
        "process exited with status {:?}",
        output.status.code()
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("smem-exporter"),
        "help output did not contain program name, got: {}",
        stdout
    );

    assert!(
        stdout.to_lowercase().contains("usage"),
        "help output did not contain 'usage', got: {}",
        stdout
    );
}
