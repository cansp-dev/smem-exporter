# smem-exporter

Prometheus exporter for detailed process memory metrics (RSS/PSS/USS) with intelligent grouping and business context.

## 🚀 Features

- **Detailed Memory Metrics**: RSS, PSS, USS per process
- **Smart Grouping**: Automatic process classification with regex patterns
- **Business Context**: Team, environment, criticality labels for aggregation
- **High Performance**: Parallel scanning with Rayon (~100ms for 600+ processes)
- **Flexible Configuration**: YAML-based configuration with comprehensive filtering
- **Production Ready**: Stable operation with minimal resource footprint

## 📊 Performance Highlights

Tested on workstation with 600+ processes:
- **Scan Time**: 85-105ms consistently
- **Memory Usage**: ~52MB constant
- **CPU Overhead**: ~0.05% average
- **Stability**: 30+ minutes proven operation

## 🛠 Installation

```bash
git clone https://github.com/yourusername/smem-exporter
cd smem-exporter
cargo build --release

## 📖 Usage

See `configs/smem-exporter.example.yaml` for detailed configuration options.

Basic Usage
./target/release/smem-exporter

With Configuration
./target/release/smem-exporter -c smem-exporter.yaml

Configuration Validation
./target/release/smem-exporter --testconfig -c config.yaml

Show complete Config with Defaults
./target/release/smem-exporter --overallconfig -c config.yaml

⚙️ Configuration
See smem-exporter.example.yaml for detailed configuration options.

Key settings:

scan_interval_seconds: Background scan interval (default: 300)

min_uss_kb: Minimum USS threshold for process inclusion

top_n_processes: Number of ungrouped processes to export

groups: Process classification rules with business metadata

## 📈 Metrics
Access metrics at: http://localhost:9215/metrics

Key Metrics:
smem_rss_bytes, smem_pss_bytes, smem_uss_bytes - Memory per process

smem_group_*_bytes - Aggregated by group/subgroup

smem_team_uss_bytes - Business context aggregates

smem_scan_* - Scan performance statistics

## 🧪 Development

# Build
cargo build

# Run tests  
cargo test

# Format code
cargo fmt

# Linting
cargo clippy

## 📄 License
MIT

