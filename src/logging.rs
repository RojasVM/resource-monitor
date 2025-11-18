use std::error::Error;
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::analyzer::SpikeEvent;
use crate::config::ResourceKind;

/// Simple JSON-lines logger for spike events.
pub struct EventLogger {
    writer: BufWriter<std::fs::File>,
}

impl EventLogger {
    /// Open (or create) the log file in append mode.
    pub fn new(log_path: &str) -> Result<Self, Box<dyn Error>> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)?;

        Ok(Self {
            writer: BufWriter::new(file),
        })
    }

    /// Append one spike event as a JSON line.
    pub fn log_event(&mut self, event: &SpikeEvent) -> Result<(), Box<dyn Error>> {
        let resource_str = match event.resource {
            ResourceKind::Cpu => "cpu",
            ResourceKind::Ram => "ram",
            ResourceKind::Io => "io",
        };

        let ts_start = format_time_secs(event.timestamp_start);
        let ts_end = format_time_secs(event.timestamp_end);
        let duration_secs = match event.timestamp_end.duration_since(event.timestamp_start) {
            Ok(d) => d.as_secs(),
            Err(_) => 0,
        };

        // Start JSON object
        write!(
            self.writer,
            "{{\"resource\":\"{}\",\"ts_start\":{},\"ts_end\":{},\"duration_secs\":{},\"peak\":{:.4},\"threshold\":{:.4},\"top\":[",
            resource_str,
            ts_start,
            ts_end,
            duration_secs,
            event.peak_value,
            event.threshold,
        )?;

        // Top processes array
        for (i, p) in event.top_processes.iter().enumerate() {
            if i > 0 {
                write!(self.writer, ",")?;
            }
            let name_escaped = escape_string(&p.name);
            write!(
                self.writer,
                "{{\"pid\":{},\"name\":\"{}\",\"cpu\":{:.4},\"ram_bytes\":{}}}",
                p.pid, name_escaped, p.cpu_percent, p.ram_bytes
            )?;
        }

        // Close JSON object and write newline
        writeln!(self.writer, "]}}")?;

        // Flush to ensure data hits disk
        self.writer.flush()?;

        Ok(())
    }
}

/// Convert SystemTime to seconds since Unix epoch.
fn format_time_secs(t: SystemTime) -> u64 {
    match t.duration_since(UNIX_EPOCH) {
        Ok(dur) => dur.as_secs(),
        Err(_) => 0,
    }
}

/// Very simple JSON string escaper.
fn escape_string(s: &str) -> String {
    // For now, only escape backslash and double quote.
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            _ => out.push(ch),
        }
    }
    out
}
