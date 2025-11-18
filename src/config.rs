#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceKind {
    Cpu,
    Ram,
    Io,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Text,
    Json,
}

#[derive(Debug, Clone)]
pub struct Thresholds {
    pub cpu_threshold: Option<f32>,
    pub ram_threshold: Option<f32>,
    pub io_threshold: Option<f32>,
}

impl Thresholds {
    pub fn new(cpu: Option<f32>, ram: Option<f32>, io: Option<f32>) -> Self {
        Self {
            cpu_threshold: cpu,
            ram_threshold: ram,
            io_threshold: io,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub interval_ms: u64,
    pub thresholds: Thresholds,
    pub min_spike_duration_secs: u64,
    pub output_format: OutputFormat,
    pub log_file: Option<String>,
    pub top_n_procs: usize,
}

#[derive(Debug, Clone, Copy)]
pub enum BatchLimit {
    DurationSecs(u64),
    Samples(u64),
}

#[derive(Debug, Clone)]
pub struct BatchConfig {
    pub runtime: RuntimeConfig,
    pub limit: BatchLimit,
}

#[derive(Debug, Clone)]
pub struct LogsQuery {
    pub log_file: String,
    pub resource_filter: Option<ResourceKind>,
    pub since: Option<u64>,  // seconds since epoch (optional)
    pub until: Option<u64>,  // seconds since epoch (optional)
    pub limit: Option<usize>,
    pub output_format: OutputFormat,
}
