use crate::analyzer::SpikeEvent;
use crate::config::{OutputFormat, ResourceKind};
use crate::metrics::SystemSnapshot;
use colored::*;
use std::time::{SystemTime, UNIX_EPOCH};

/// Format SystemTime as seconds since Unix epoch.
fn format_time_secs(t: SystemTime) -> String {
    match t.duration_since(UNIX_EPOCH) {
        Ok(dur) => format!("{}", dur.as_secs()),
        Err(_) => "0".to_string(),
    }
}

/// Human unit label for resource values.
fn resource_unit(kind: ResourceKind) -> &'static str {
    match kind {
        ResourceKind::Cpu => "%",
        ResourceKind::Ram => "%",
        ResourceKind::Io => "MB/s",
    }
}

/// Print one line with current system metrics.
pub fn print_snapshot(snapshot: &SystemSnapshot, format: OutputFormat) {
    match format {
        OutputFormat::Text => {
            let ts = format_time_secs(snapshot.timestamp);
            let ts_str = format!("[{}]", ts).dimmed();

            let cpu_label = "CPU".cyan().bold();
            let ram_label = "RAM".green().bold();
            let io_label = "IO".magenta().bold();

            println!(
                "{} {}: {:.1}% | {}: {:.1}% | {}: {:.2} B/s r, {:.2} B/s w",
                ts_str,
                cpu_label,
                snapshot.cpu_usage_percent,
                ram_label,
                snapshot.ram_usage_percent,
                io_label,
                snapshot.io_read_bytes_per_s,
                snapshot.io_write_bytes_per_s,
            );
        }
        OutputFormat::Json => {
            let ts = format_time_secs(snapshot.timestamp);
            println!(
                "{{\"ts\":{},\"cpu\":{:.1},\"ram\":{:.1},\"io_read\":{:.2},\"io_write\":{:.2}}}",
                ts,
                snapshot.cpu_usage_percent,
                snapshot.ram_usage_percent,
                snapshot.io_read_bytes_per_s,
                snapshot.io_write_bytes_per_s,
            );
        }
    }
}

/// Print a spike event (alert) in text or JSON format.
pub fn print_event(event: &SpikeEvent, format: OutputFormat) {
    match format {
        OutputFormat::Text => {
            let ts_start = format_time_secs(event.timestamp_start);
            let ts_end = format_time_secs(event.timestamp_end);
            let duration_secs = match event.timestamp_end.duration_since(event.timestamp_start) {
                Ok(d) => d.as_secs(),
                Err(_) => 0,
            };

            let resource = match event.resource {
                ResourceKind::Cpu => "CPU",
                ResourceKind::Ram => "RAM",
                ResourceKind::Io => "IO",
            };

            let unit = resource_unit(event.resource);

            let header = format!(
                ">>> {} spike: start={} end={} duration={}s peak={:.2}{} (threshold={:.2}{})",
                resource,
                ts_start,
                ts_end,
                duration_secs,
                event.peak_value,
                unit,
                event.threshold,
                unit,
            )
            .red()
            .bold();

            println!("{}", header);

            if !event.top_processes.is_empty() {
                println!("{}", "    Top processes at peak:".yellow());
                for p in &event.top_processes {
                    println!(
                        "      PID {} ({}) CPU={:.1}% RAM={} bytes",
                        p.pid.to_string().cyan(),
                        p.name,
                        p.cpu_percent,
                        p.ram_bytes
                    );
                }
            }
        }
        OutputFormat::Json => {
            let ts_start = format_time_secs(event.timestamp_start);
            let ts_end = format_time_secs(event.timestamp_end);
            let duration_secs = match event.timestamp_end.duration_since(event.timestamp_start) {
                Ok(d) => d.as_secs(),
                Err(_) => 0,
            };

            let resource_str = match event.resource {
                ResourceKind::Cpu => "cpu",
                ResourceKind::Ram => "ram",
                ResourceKind::Io => "io",
            };

            print!(
                "{{\"resource\":\"{}\",\"ts_start\":{},\"ts_end\":{},\"duration_secs\":{},\"peak\":{:.2},\"threshold\":{:.2},\"top\":[",
                resource_str,
                ts_start,
                ts_end,
                duration_secs,
                event.peak_value,
                event.threshold,
            );

            for (i, p) in event.top_processes.iter().enumerate() {
                if i > 0 {
                    print!(",");
                }
                print!(
                    "{{\"pid\":{},\"name\":\"{}\",\"cpu\":{:.1},\"ram_bytes\":{}}}",
                    p.pid, p.name, p.cpu_percent, p.ram_bytes
                );
            }

            println!("]}}");
        }
    }
}
