use std::process::{Command, Stdio};
use std::{thread, time::Duration};

#[test]
fn metrics_endpoint_exposes_expected_metric_names() {
    let exe = env!("CARGO_BIN_EXE_smem_exporter");

    let mut child = Command::new(exe)
        .arg("--listen")
        .arg("127.0.0.1:9922")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("could not start smem-exporter");

    // give the server a moment to start
    thread::sleep(Duration::from_millis(500));

    let resp = reqwest::blocking::get("http://127.0.0.1:9922/metrics")
        .expect("request to /metrics failed");

    assert!(
        resp.status().is_success(),
        "/metrics did not return success, got: {}",
        resp.status()
    );

    let body = resp.text().unwrap();

    assert!(
        body.contains("smem_rss_bytes")
            && body.contains("smem_pss_bytes")
            && body.contains("smem_uss_bytes"),
        "metrics output did not contain expected metric names; got:
{}",
        body
    );

    let _ = child.kill();
}
