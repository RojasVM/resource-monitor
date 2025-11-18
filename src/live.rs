use std::error::Error;
use std::thread;
use std::time::Duration;

use crate::analyzer::{analyze_snapshot, AnalyzerState};
use crate::config::RuntimeConfig;
use crate::logging::EventLogger;
use crate::metrics::read_system_snapshot;
use crate::output::{print_event, print_snapshot};

/// Live mode: monitor until interrupted.
pub fn run_live(config: RuntimeConfig) -> Result<(), Box<dyn Error>> {
    let mut analyzer_state = AnalyzerState::new();

    let mut logger = match &config.log_file {
        Some(path) => Some(EventLogger::new(path)?),
        None => None,
    };

    loop {
        thread::sleep(Duration::from_millis(config.interval_ms));

        let snapshot = match read_system_snapshot(config.top_n_procs) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[monitor] Error reading snapshot: {e}");
                continue;
            }
        };

        print_snapshot(&snapshot, config.output_format);

        let events = analyze_snapshot(
            &snapshot,
            &config.thresholds,
            config.min_spike_duration_secs,
            &mut analyzer_state,
        );

        for event in events {
            print_event(&event, config.output_format);

            if let Some(logger) = &mut logger {
                if let Err(e) = logger.log_event(&event) {
                    eprintln!("[monitor] Error logging event: {e}");
                }
            }
        }
    }
}
