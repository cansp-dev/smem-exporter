use std::process::{Command, Stdio};
use std::{thread, time::Duration};

#[test]
fn health_endpoint_works() {
    let exe = env!("CARGO_BIN_EXE_smem_exporter");

    let mut child = Command::new(exe)
        .arg("--listen")
        .arg("127.0.0.1:9921")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("could not start smem-exporter");

    // give the server a moment to start
    thread::sleep(Duration::from_millis(400));

    let resp = reqwest::blocking::get("http://127.0.0.1:9921/health")
        .expect("request to /health failed");

    assert!(
        resp.status().is_success(),
        "/health did not return success, got: {}",
        resp.status()
    );

    let body = resp.text().unwrap();
    assert_eq!(body.trim(), "OK", "unexpected /health body: {}", body);

    let _ = child.kill();
}
