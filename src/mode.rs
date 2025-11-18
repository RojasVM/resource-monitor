use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

use serde::Deserialize;

use crate::config::{LogsQuery, OutputFormat, ResourceKind};

/// Log record as stored in the JSON-lines file.
#[derive(Debug, Deserialize)]
struct LogRecord {
    resource: String,
    ts_start: u64,
    ts_end: u64,
    duration_secs: u64,
    peak: f64,
    threshold: f64,
    #[allow(dead_code)]
    top: Vec<LogProc>,
}

#[derive(Debug, Deserialize)]
struct LogProc {
    pid: u32,
    name: String,
    cpu: f64,
    ram_bytes: u64,
}

/// Read log file and print events with optional filters.
pub fn run_logs(query: LogsQuery) -> Result<(), Box<dyn Error>> {
    let file = File::open(&query.log_file)?;
    let reader = BufReader::new(file);

    let mut printed: usize = 0;

    for line in reader.lines() {
        let line = line?;

        // Parse JSON line into LogRecord
        let record: LogRecord = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("[monitor-logs] Failed to parse log line: {e}");
                continue;
            }
        };

        // Filter by resource type
        if let Some(kind) = query.resource_filter {
            if !resource_matches(&record, kind) {
                continue;
            }
        }

        // Filter by time range (if provided)
        if let Some(since) = query.since {
            if record.ts_start < since {
                continue;
            }
        }
        if let Some(until) = query.until {
            if record.ts_start > until {
                continue;
            }
        }

        // Limit number of entries
        if let Some(max) = query.limit {
            if printed >= max {
                break;
            }
        }

        match query.output_format {
            OutputFormat::Json => {
                // Print original JSON line
                println!("{}", line);
            }
            OutputFormat::Text => {
                print_record_text(&record);
            }
        }

        printed += 1;
    }

    Ok(())
}

fn resource_matches(record: &LogRecord, kind: ResourceKind) -> bool {
    match kind {
        ResourceKind::Cpu => record.resource == "cpu",
        ResourceKind::Ram => record.resource == "ram",
        ResourceKind::Io => record.resource == "io",
    }
}

fn print_record_text(r: &LogRecord) {
    let resource = match r.resource.as_str() {
        "cpu" => "CPU",
        "ram" => "RAM",
        "io" => "IO",
        _ => "UNKNOWN",
    };

    let unit = match r.resource.as_str() {
        "cpu" => "%",
        "ram" => "%",
        "io" => "MB/s",
        _ => "",
    };

    println!(
        "[LOG] {} spike: start={} end={} duration={}s peak={:.2}{} (threshold={:.2}{})",
        resource,
        r.ts_start,
        r.ts_end,
        r.duration_secs,
        r.peak,
        unit,
        r.threshold,
        unit,
    );
}
