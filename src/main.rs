mod analyzer;
mod batch;
mod config;
mod logging;
mod metrics;
mod output;
mod live;
mod logs_mode;

use clap::{Parser, Subcommand, CommandFactory};
use crate::batch::run_batch;
use crate::config::{
    BatchConfig, BatchLimit, LogsQuery, OutputFormat, ResourceKind, Thresholds, RuntimeConfig,
};
use crate::live::run_live;
use crate::logs_mode::run_logs;

/// CLI entry point.
#[derive(Parser, Debug)]
#[command(
    name = "resource_monitor",
    about = "Resource spike monitor for Linux (CPU/RAM/IO) with live, batch and log modes.",
    long_about = "Resource spike monitor for Linux that:\n\
                  - Samples CPU and RAM usage from /proc\n\
                  - Detects spikes based on user-defined thresholds\n\
                  - Supports live streaming, batch runs and log inspection\n\
                  - Writes spike events as JSON lines for further processing",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

/// CLI subcommands.
#[derive(Subcommand, Debug)]
enum Commands {
    /// Live monitoring mode (run until interrupted).
    Live {
        /// Sampling interval in milliseconds.
        #[arg(long, default_value_t = 1000)]
        interval_ms: u64,

        /// CPU spike threshold in percent (0-100).
        #[arg(long)]
        cpu_threshold: Option<f32>,

        /// RAM spike threshold in percent (0-100).
        #[arg(long)]
        ram_threshold: Option<f32>,

        /// IO spike threshold in MB/s (currently not implemented).
        #[arg(long)]
        io_threshold: Option<f32>,

        /// Minimum spike duration in seconds.
        #[arg(long, default_value_t = 3)]
        min_spike_duration_secs: u64,

        /// Output format: text or json.
        #[arg(long, default_value = "text")]
        output: String,

        /// Optional log file path for spike events.
        #[arg(long)]
        log_file: Option<String>,

        /// Number of top processes to record in spike events (not implemented yet).
        #[arg(long, default_value_t = 0)]
        top_n_procs: usize,
    },

    /// Batch mode: stop after N samples or N seconds.
    Batch {
        /// Sampling interval in milliseconds.
        #[arg(long, default_value_t = 1000)]
        interval_ms: u64,

        /// Total duration in seconds (exclusive with --samples).
        #[arg(long)]
        duration_secs: Option<u64>,

        /// Total number of samples (exclusive with --duration-secs).
        #[arg(long)]
        samples: Option<u64>,

        /// CPU spike threshold in percent (0-100).
        #[arg(long)]
        cpu_threshold: Option<f32>,

        /// RAM spike threshold in percent (0-100).
        #[arg(long)]
        ram_threshold: Option<f32>,

        /// IO spike threshold in MB/s (currently not implemented).
        #[arg(long)]
        io_threshold: Option<f32>,

        /// Minimum spike duration in seconds.
        #[arg(long, default_value_t = 3)]
        min_spike_duration_secs: u64,

        /// Output format: text or json.
        #[arg(long, default_value = "text")]
        output: String,

        /// Optional log file path for spike events.
        #[arg(long)]
        log_file: Option<String>,

        /// Number of top processes to record in spike events (not implemented yet).
        #[arg(long, default_value_t = 0)]
        top_n_procs: usize,
    },

    /// Show spike events stored in a log file.
    Logs {
        /// Log file path.
        #[arg(long)]
        log_file: String,

        /// Filter by resource: cpu, ram or io.
        #[arg(long)]
        resource: Option<String>,

        /// Only show events with ts_start >= this (seconds since epoch).
        #[arg(long)]
        since: Option<u64>,

        /// Only show events with ts_start <= this (seconds since epoch).
        #[arg(long)]
        until: Option<u64>,

        /// Limit number of events shown.
        #[arg(long)]
        limit: Option<usize>,

        /// Output format: text or json.
        #[arg(long, default_value = "text")]
        output: String,
    },
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        // ----------------------------
        // LIVE MODE
        // ----------------------------
        Some(Commands::Live {
            interval_ms,
            cpu_threshold,
            ram_threshold,
            io_threshold,
            min_spike_duration_secs,
            output,
            log_file,
            top_n_procs,
        }) => {
            let thresholds = Thresholds::new(cpu_threshold, ram_threshold, io_threshold);
            let output_format = parse_output_format(&output);

            let config = RuntimeConfig {
                interval_ms,
                thresholds,
                min_spike_duration_secs,
                output_format,
                log_file,
                top_n_procs,
            };

            run_live(config)
        }

        // ----------------------------
        // BATCH MODE
        // ----------------------------
        Some(Commands::Batch {
            interval_ms,
            duration_secs,
            samples,
            cpu_threshold,
            ram_threshold,
            io_threshold,
            min_spike_duration_secs,
            output,
            log_file,
            top_n_procs,
        }) => {
            let thresholds = Thresholds::new(cpu_threshold, ram_threshold, io_threshold);
            let output_format = parse_output_format(&output);

            let limit = if let Some(d) = duration_secs {
                BatchLimit::DurationSecs(d)
            } else if let Some(s) = samples {
                BatchLimit::Samples(s)
            } else {
                BatchLimit::Samples(10)
            };

            let runtime = RuntimeConfig {
                interval_ms,
                thresholds,
                min_spike_duration_secs,
                output_format,
                log_file,
                top_n_procs,
            };

            let config = BatchConfig { runtime, limit };
            run_batch(config)
        }

        // ----------------------------
        // LOGS MODE
        // ----------------------------
        Some(Commands::Logs {
            log_file,
            resource,
            since,
            until,
            limit,
            output,
        }) => {
            // Parse resource filter
            let resource_filter: Option<ResourceKind> = match resource.as_deref() {
                Some("cpu") => Some(ResourceKind::Cpu),
                Some("ram") => Some(ResourceKind::Ram),
                Some("io") => Some(ResourceKind::Io),
                Some(other) => {
                    eprintln!("Invalid resource filter '{}', ignoring filter.", other);
                    None
                }
                None => None,
            };

            let output_format = parse_output_format(&output);

            let query = LogsQuery {
                log_file,
                resource_filter,
                since,
                until,
                limit,
                output_format,
            };

            run_logs(query)
        }

        // ----------------------------
        // NO SUBCOMMAND → show help
        // ----------------------------
        None => {
            let mut cmd = Cli::command();
            cmd.print_help()?;
            println!();
            Ok(())
        }
    }
}

/// Convert string to OutputFormat.
fn parse_output_format(s: &str) -> OutputFormat {
    match s {
        "text" => OutputFormat::Text,
        "json" => OutputFormat::Json,
        other => {
            eprintln!("Invalid output '{}', using 'text'.", other);
            OutputFormat::Text
        }
    }
}
