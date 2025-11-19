mod version;
use version::VERSION;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "smem_exporter",
    version = VERSION,
    about = "Prometheus exporter for per-process RSS/PSS/USS"
)]
struct Args {}

fn main() {
    println!("Version: {}", VERSION);
}
