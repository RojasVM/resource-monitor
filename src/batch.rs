use std::error::Error;
use std::thread;
use std::time::{Duration, Instant};

use crate::analyzer::{analyze_snapshot, AnalyzerState};
use crate::config::{BatchConfig, BatchLimit};
use crate::logging::EventLogger;
use crate::metrics::read_system_snapshot;
use crate::output::{print_event, print_snapshot};

/// Batch mode: run for a fixed time or number of samples, then exit.
pub fn run_batch(config: BatchConfig) -> Result<(), Box<dyn Error>> {
    let mut analyzer_state = AnalyzerState::new();

    let mut logger = match &config.runtime.log_file {
        Some(path) => Some(EventLogger::new(path)?),
        None => None,
    };

    let start = Instant::now();
    let mut samples: u64 = 0;

    loop {
        // Check stop conditions
        match config.limit {
            BatchLimit::DurationSecs(max_secs) => {
                if start.elapsed().as_secs() >= max_secs {
                    break;
                }
            }
            BatchLimit::Samples(max_samples) => {
                if samples >= max_samples {
                    break;
                }
            }
        }

        thread::sleep(Duration::from_millis(config.runtime.interval_ms));

        let snapshot = match read_system_snapshot(config.runtime.top_n_procs) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[monitor-batch] Error reading snapshot: {e}");
                continue;
            }
        };

        print_snapshot(&snapshot, config.runtime.output_format);

        let events = analyze_snapshot(
            &snapshot,
            &config.runtime.thresholds,
            config.runtime.min_spike_duration_secs,
            &mut analyzer_state,
        );

        for event in events {
            print_event(&event, config.runtime.output_format);

            if let Some(logger) = &mut logger {
                if let Err(e) = logger.log_event(&event) {
                    eprintln!("[monitor-batch] Error logging event: {e}");
                }
            }
        }

        samples += 1;
    }

    Ok(())
}
