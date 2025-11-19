// smem_exporter - final version 1.0.0
// Fully production-ready, top-N support, sorting, cache, metrics

use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Router};
use clap::Parser;
use prometheus::{Encoder, Gauge, GaugeVec, Opts, Registry, TextEncoder};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    io::{BufRead, BufReader},
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
    time::Instant,
};
use tokio::{
    net::TcpListener,
    signal,
    sync::RwLock,
    time::{interval, Duration},
};

type SharedState = Arc<AppState>;

const DEFAULT_LISTEN_ADDR: &str = "0.0.0.0:9215";
const DEFAULT_CACHE_TTL: u64 = 30;
const BUFFER_CAP: usize = 512 * 1024;

/// CLI Arguments
#[derive(Parser, Debug)]
#[command(
    name = "smem_exporter",
    version,
    about = "Prometheus exporter for per-process RSS/PSS/USS"
)]
struct Args {
    #[arg(short, long)]
    config: Option<String>,

    #[arg(short, long)]
    listen: Option<String>,

    #[arg(long)]
    print_config: bool,

    #[arg(short, long, default_value_t = DEFAULT_CACHE_TTL)]
    cache_ttl: u64,
}

/// YAML configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Config {
    listen_addr: Option<String>,
    min_uss_kb: Option<u64>,

    /// Sorting: "uss", "rss", "pss"
    top_sort: Option<String>,

    /// Limit output to top N processes
    top_n_processes: Option<usize>,

    include_names: Option<Vec<String>>,
    exclude_names: Option<Vec<String>>,
    parallelism: Option<usize>,
    max_processes: Option<usize>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            listen_addr: Some(DEFAULT_LISTEN_ADDR.to_string()),
            min_uss_kb: Some(0),
            top_sort: Some("uss".into()),
            top_n_processes: Some(20),
            include_names: None,
            exclude_names: None,
            parallelism: None,
            max_processes: None,
        }
    }
}

/// Process entry from /proc
#[derive(Debug, Clone)]
struct ProcEntry {
    pid: String,
    proc_path: PathBuf,
}

/// Process memory metrics
#[derive(Debug, Clone)]
struct ProcMem {
    pid: String,
    name: String,
    rss: u64,
    pss: u64,
    uss: u64,
}

/// Prometheus metric set
#[derive(Clone)]
struct MemoryMetrics {
    rss: GaugeVec,
    pss: GaugeVec,
    uss: GaugeVec,
}

impl MemoryMetrics {
    fn new(registry: &Registry) -> Result<Self, Box<dyn std::error::Error>> {
        let labels = &["pid", "name"];

        let rss = GaugeVec::new(
            Opts::new("smem_rss_bytes", "Resident Set Size per process in bytes"),
            labels,
        )?;
        let pss = GaugeVec::new(
            Opts::new("smem_pss_bytes", "Proportional Set Size per process in bytes"),
            labels,
        )?;
        let uss = GaugeVec::new(
            Opts::new("smem_uss_bytes", "Unique Set Size per process in bytes"),
            labels,
        )?;

        registry.register(Box::new(rss.clone()))?;
        registry.register(Box::new(pss.clone()))?;
        registry.register(Box::new(uss.clone()))?;

        Ok(Self { rss, pss, uss })
    }

    fn reset(&self) {
        self.rss.reset();
        self.pss.reset();
        self.uss.reset();
    }

    fn set_for_process(&self, pid: &str, name: &str, rss: u64, pss: u64, uss: u64) {
        let labels = &[pid, name];
        self.rss.with_label_values(labels).set(rss as f64);
        self.pss.with_label_values(labels).set(pss as f64);
        self.uss.with_label_values(labels).set(uss as f64);
    }
}

/// Cache state
#[derive(Clone, Default)]
struct MetricsCache {
    processes: HashMap<String, ProcMem>,
    last_updated: Option<Instant>,
    update_duration_seconds: f64,
    update_success: bool,
}

/// Global application state
struct AppState {
    registry: Registry,
    metrics: MemoryMetrics,
    scrape_duration: Gauge,
    processes_total: Gauge,
    cache_update_duration: Gauge,
    cache_update_success: Gauge,
    cache: Arc<RwLock<MetricsCache>>,
    config: Arc<Config>,
}

/// Error for /metrics
#[derive(Debug)]
enum MetricsError {
    EncodingFailed,
}
impl IntoResponse for MetricsError {
    fn into_response(self) -> axum::response::Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to encode metrics",
        )
        .into_response()
    }
}

/// --------------------------------------------------------
/// MAIN PROGRAM
/// --------------------------------------------------------
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Load config
    let config = Arc::new(load_config(args.config.as_deref()));

    if args.print_config {
        println!("{}", serde_yaml::to_string(&*config)?);
        return Ok(());
    }

    let listen_addr = args
        .listen
        .as_deref()
        .or(config.listen_addr.as_deref())
        .unwrap_or(DEFAULT_LISTEN_ADDR);

    if let Some(threads) = config.parallelism {
        if threads > 0 {
            rayon::ThreadPoolBuilder::new()
                .num_threads(threads)
                .build_global()
                .unwrap_or_else(|e| eprintln!("WARN: Failed to set rayon thread pool: {}", e));
        }
    }

    // Metrics registry
    let registry = Registry::new();

    // Metric sets
    let metrics = MemoryMetrics::new(&registry)?;
    let scrape_duration = Gauge::new(
        "smem_scrape_duration_seconds", 
        "Time spent serving /metrics request (reading from cache)"
    )?;
    let processes_total = Gauge::new(
        "smem_processes_total", 
        "Number of processes currently exported by smem_exporter"
    )?;
    let cache_update_duration = Gauge::new(
        "smem_cache_update_duration_seconds",
        "Time spent updating the process metrics cache in background",
    )?;
    let cache_update_success = Gauge::new(
        "smem_cache_update_success", 
        "Whether the last cache update was successful (1) or failed (0)"
    )?;

    registry.register(Box::new(scrape_duration.clone()))?;
    registry.register(Box::new(processes_total.clone()))?;
    registry.register(Box::new(cache_update_duration.clone()))?;
    registry.register(Box::new(cache_update_success.clone()))?;

    // Shared State
    let state = Arc::new(AppState {
        registry,
        metrics,
        scrape_duration,
        processes_total,
        cache_update_duration,
        cache_update_success,
        cache: Arc::new(RwLock::new(MetricsCache::default())),
        config: config.clone(),
    });

    // FIRST CACHE UPDATE
    if let Err(e) = update_cache(&state).await {
        eprintln!("Initial cache failed: {}", e);
    }

    // Background refresh
    let bg_state = state.clone();
    let ttl = Duration::from_secs(args.cache_ttl);

    let background_task = tokio::spawn(async move {
        let mut int = interval(ttl);
        loop {
            int.tick().await;
            if let Err(e) = update_cache(&bg_state).await {
                eprintln!("Cache update failed: {}", e);
            }
        }
    });

    // Graceful shutdown handler
    let shutdown_signal = async {
        let ctrl_c = async {
            signal::ctrl_c()
                .await
                .expect("Failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("Failed to install signal handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {
                eprintln!("Received SIGINT (Ctrl+C), shutting down gracefully...");
            }
            _ = terminate => {
                eprintln!("Received SIGTERM, shutting down gracefully...");
            }
        }
    };

    // Prepare server
    let addr: SocketAddr = listen_addr.parse()?;
    let app = Router::new()
        .route("/metrics", get(metrics_handler))
        .route("/health", get(|| async { (StatusCode::OK, "OK") }))
        .with_state(state.clone());

    let listener = TcpListener::bind(addr).await?;
    println!("smem_exporter listening on http://{}", listen_addr);

    // Start server with graceful shutdown
    let server = axum::serve(listener, app);

    tokio::select! {
        result = server => {
            if let Err(e) = result {
                eprintln!("Server error: {}", e);
                return Err(e.into());
            }
        }
        _ = shutdown_signal => {
            println!("Shutdown signal received, exiting...");
        }
    }

    // Cleanup: cancel background task
    background_task.abort();
    let _ = background_task.await;

    println!("smem_exporter stopped gracefully");
    Ok(())
}

/// --------------------------------------------------------
/// /metrics HANDLER
/// --------------------------------------------------------
async fn metrics_handler(State(state): State<SharedState>) -> Result<String, MetricsError> {
    let start = Instant::now();

    let processes_vec: Vec<ProcMem>;
    let meta: (f64, bool);

    {
        let cache = state.cache.read().await;
        processes_vec = cache.processes.values().cloned().collect();
        meta = (cache.update_duration_seconds, cache.update_success);
    }

    state.cache_update_duration.set(meta.0);
    state.cache_update_success.set(if meta.1 { 1.0 } else { 0.0 });

    state.metrics.reset();

    for p in &processes_vec {
        state
            .metrics
            .set_for_process(&p.pid, &p.name, p.rss, p.pss, p.uss);
    }

    state.processes_total.set(processes_vec.len() as f64);

    state.scrape_duration.set(start.elapsed().as_secs_f64());

    let families = state.registry.gather();
    let mut buffer = Vec::with_capacity(BUFFER_CAP);
    let encoder = TextEncoder::new();

    if encoder.encode(&families, &mut buffer).is_err() {
        return Err(MetricsError::EncodingFailed);
    }

    String::from_utf8(buffer).map_err(|_| MetricsError::EncodingFailed)
}

/// --------------------------------------------------------
/// CACHE UPDATE
/// --------------------------------------------------------
async fn update_cache(state: &SharedState) -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();

    let entries = collect_proc_entries("/proc", state.config.max_processes);
    let min_uss_bytes = state.config.min_uss_kb.unwrap_or(0) * 1024;
    let top_n = state.config.top_n_processes.unwrap_or(20);

    let mut results: Vec<ProcMem> = entries
        .par_iter()
        .filter_map(|entry| {
            let name = read_process_name(&entry.proc_path)?;
            if !should_include_process(&name, &state.config) {
                return None;
            }

            let smaps_path = entry.proc_path.join("smaps");
            if let Ok((rss, pss, uss)) = parse_smaps(&smaps_path) {
                if uss < min_uss_bytes {
                    return None;
                }
                return Some(ProcMem {
                    pid: entry.pid.clone(),
                    name,
                    rss,
                    pss,
                    uss,
                });
            }
            None
        })
        .collect();

    // Validate and apply sorting
    let mut sort_key = state.config.top_sort.clone().unwrap_or("uss".into());
    if !["uss", "rss", "pss"].contains(&sort_key.as_str()) {
        eprintln!("WARN: Invalid top_sort '{}', using 'uss'", sort_key);
        sort_key = "uss".into();
    }

    match sort_key.as_str() {
        "rss" => results.sort_by_key(|p| std::cmp::Reverse(p.rss)),
        "pss" => results.sort_by_key(|p| std::cmp::Reverse(p.pss)),
        _ => results.sort_by_key(|p| std::cmp::Reverse(p.uss)),
    }

    // Limit to top-N
    if results.len() > top_n {
        results.truncate(top_n);
    }

    // Log if no processes found
    if results.is_empty() {
        eprintln!("WARN: No processes matched filters after sorting");
    }

    // Write final data into cache
    let mut cache = state.cache.write().await;
    cache.processes.clear();
    for p in results {
        cache.processes.insert(p.pid.clone(), p);
    }

    cache.update_duration_seconds = start.elapsed().as_secs_f64();
    cache.update_success = true;
    cache.last_updated = Some(start);

    eprintln!(
        "Cache updated with {} processes (top {} by {}) in {:.2}ms",
        cache.processes.len(),
        top_n,
        sort_key,
        cache.update_duration_seconds * 1000.0
    );

    Ok(())
}

//
// Helper functions (unchanged)
//

fn collect_proc_entries(root: &str, max: Option<usize>) -> Vec<ProcEntry> {
    let mut out = Vec::new();
    if let Ok(entries) = fs::read_dir(root) {
        for entry in entries.flatten() {
            let p = entry.path();
            let name = match p.file_name().and_then(|s| s.to_str()) {
                Some(v) => v,
                None => continue,
            };
            if !name.chars().all(|c| c.is_ascii_digit()) {
                continue;
            }
            if !p.join("smaps").exists() {
                continue;
            }
            out.push(ProcEntry {
                pid: name.to_string(),
                proc_path: p,
            });
            if let Some(maxp) = max {
                if out.len() >= maxp {
                    break;
                }
            }
        }
    }
    out
}

fn read_process_name(proc_path: &Path) -> Option<String> {
    let comm = proc_path.join("comm");
    if let Ok(s) = fs::read_to_string(&comm) {
        let t = s.trim();
        if !t.is_empty() {
            return Some(t.into());
        }
    }

    let cmd = proc_path.join("cmdline");
    if let Ok(content) = fs::read(&cmd) {
        if !content.is_empty() {
            let parts: Vec<&str> = content
                .split(|&b| b == 0u8)
                .filter_map(|s| std::str::from_utf8(s).ok())
                .collect();
            if !parts.is_empty() {
                return Some(parts[0].to_string());
            }
        }
    }
    None
}

fn parse_smaps(path: &Path) -> Result<(u64, u64, u64), std::io::Error> {
    let file = fs::File::open(path)?;
    let reader = BufReader::new(file);

    let mut rss = 0;
    let mut pss = 0;
    let mut pc = 0;
    let mut pd = 0;

    for line in reader.lines() {
        let l = line?;
        if let Some(kb) = l.strip_prefix("Rss:") {
            rss += parse_kb_value(kb).unwrap_or(0);
        } else if let Some(kb) = l.strip_prefix("Pss:") {
            pss += parse_kb_value(kb).unwrap_or(0);
        } else if let Some(kb) = l.strip_prefix("Private_Clean:") {
            pc += parse_kb_value(kb).unwrap_or(0);
        } else if let Some(kb) = l.strip_prefix("Private_Dirty:") {
            pd += parse_kb_value(kb).unwrap_or(0);
        }
    }

    Ok((rss * 1024, pss * 1024, (pc + pd) * 1024))
}

fn parse_kb_value(v: &str) -> Option<u64> {
    v.split_whitespace().next()?.parse().ok()
}

fn should_include_process(name: &str, cfg: &Config) -> bool {
    if let Some(ex) = &cfg.exclude_names {
        if ex.iter().any(|s| name.contains(s)) {
            return false;
        }
    }
    if let Some(inc) = &cfg.include_names {
        if !inc.is_empty() {
            return inc.iter().any(|s| name.contains(s));
        }
    }
    true
}

fn load_config(path: Option<&str>) -> Config {
    if let Some(p) = path {
        if Path::new(p).exists() {
            if let Ok(txt) = fs::read_to_string(p) {
                if let Ok(cfg) = serde_yaml::from_str(&txt) {
                    eprintln!("Loaded configuration from: {}", p);
                    return cfg;
                } else {
                    eprintln!("ERROR: Failed to parse configuration from: {}", p);
                }
            } else {
                eprintln!("ERROR: Failed to read configuration from: {}", p);
            }
        } else {
            eprintln!("WARN: Configuration file not found: {}", p);
        }
    }

    let defaults = [
        "/etc/smem_exporter.yaml",
        "/etc/smem_exporter.yml",
        "./smem_exporter.yaml",
        "./smem_exporter.yml",
    ];

    for p in &defaults {
        if Path::new(p).exists() {
            if let Ok(txt) = fs::read_to_string(p) {
                if let Ok(cfg) = serde_yaml::from_str(&txt) {
                    eprintln!("Loaded configuration from: {}", p);
                    return cfg;
                } else {
                    eprintln!("ERROR: Failed to parse configuration from: {}", p);
                }
            } else {
                eprintln!("ERROR: Failed to read configuration from: {}", p);
            }
        }
    }

    eprintln!("Using default configuration");
    Config::default()
}