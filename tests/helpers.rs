
use std::path::PathBuf;

pub fn find_binary() -> PathBuf {
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_smem_exporter") {
        return PathBuf::from(path);
    }

    let debug_bin = PathBuf::from("target/debug/smem-exporter");
    if debug_bin.exists() {
        return debug_bin;
    }

    panic!("smem_exporter binary not found. Run tests via `cargo test`.");
}
