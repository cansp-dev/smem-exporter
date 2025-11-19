use std::process::Command;

#[test]
fn version_flag_works() {
    let exe = env!("CARGO_BIN_EXE_smem_exporter");

    let output = Command::new(exe)
        .arg("--version")
        .output()
        .expect("failed to run --version");

    assert!(
        output.status.success(),
        "process exited with status {:?}",
        output.status.code()
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.to_lowercase().contains("smem-exporter"),
        "version output did not contain program name, got: {}",
        stdout
    );

    assert!(
        stdout.contains(env!("CARGO_PKG_VERSION")),
        "version output did not contain crate version {}, got: {}",
        env!("CARGO_PKG_VERSION"),
        stdout
    );
}
