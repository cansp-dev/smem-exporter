mod helpers;
use helpers::find_binary;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

fn wait_for_port(addr: &str, retries: u32) -> bool {
    for _ in 0..retries {
        if TcpStream::connect(addr).is_ok() {
            return true;
        }
        thread::sleep(Duration::from_millis(100));
    }
    false
}

#[test]
fn health_endpoint_works() {
    let exe = find_binary();
    let addr = "127.0.0.1:19215";

    let mut child = Command::new(exe)
        .arg("--listen")
        .arg(addr)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to start smem_exporter");

    if !wait_for_port(addr, 50) {
        let _ = child.kill();
        panic!("smem_exporter did not start listening on {}", addr);
    }

    let mut stream = TcpStream::connect(addr).expect("failed to connect to smem_exporter");
    stream
        .write_all(b"GET /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
        .expect("failed to write request");

    let mut buf = String::new();
    stream
        .read_to_string(&mut buf)
        .expect("failed to read response");

    let _ = child.kill();
    let _ = child.wait();

    assert!(
        buf.contains("200 OK"),
        "health response missing 200 OK: {}",
        buf
    );
    assert!(buf.contains("OK"), "health body missing OK: {}", buf);
}
