
use std::env;

pub fn get_binary() -> String {
    env::var("CARGO_BIN_EXE_smem_exporter")
        .expect("smem_exporter binary not found. Run tests via `cargo test`.")
}
