# smem-exporter

A Prometheus exporter providing per-process memory metrics (RSS, PSS, USS) with configurable top-N filtering and efficient caching.

## Features
- Top-N processes by RSS/PSS/USS
- Smart caching
- Parallel /proc scanning
- Robust error handling
- Graceful shutdown

## Build
```
cargo build --release
```

## Run
```
./smem-exporter
```

## License
MIT
