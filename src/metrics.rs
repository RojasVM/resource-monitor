use std::error::Error;
use std::fs;
use std::sync::{Mutex, OnceLock};
use std::time::SystemTime;

/// Simple process info (placeholder for future use).
#[derive(Debug, Clone)]
pub struct ProcessSample {
    pub pid: u32,
    pub name: String,
    pub cpu_percent: f32,
    pub ram_bytes: u64,
}

/// System metrics snapshot for one tick.
#[derive(Debug, Clone)]
pub struct SystemSnapshot {
    pub timestamp: SystemTime,
    pub cpu_usage_percent: f32,
    pub ram_usage_percent: f32,
    pub io_read_bytes_per_s: f64,    // 0.0 for now
    pub io_write_bytes_per_s: f64,   // 0.0 for now
    pub top_processes: Vec<ProcessSample>,
}

/// Raw CPU times from /proc/stat.
#[derive(Debug, Clone, Copy)]
struct CpuTimes {
    idle_all: u64,
    total: u64,
}

/// Global state for last CPU times.
static LAST_CPU_TIMES: OnceLock<Mutex<Option<CpuTimes>>> = OnceLock::new();

fn cpu_state() -> &'static Mutex<Option<CpuTimes>> {
    LAST_CPU_TIMES.get_or_init(|| Mutex::new(None))
}

/// Build a SystemSnapshot using /proc data.
pub fn read_system_snapshot(
    _top_n_procs: usize,
) -> Result<SystemSnapshot, Box<dyn Error>> {
    let timestamp = SystemTime::now();

    let cpu_usage_percent = read_cpu_usage_percent_delta()?;
    let ram_usage_percent = read_ram_usage_percent()?;

    // IO not implemented yet.
    let io_read_bytes_per_s = 0.0;
    let io_write_bytes_per_s = 0.0;
    let top_processes = Vec::new();

    Ok(SystemSnapshot {
        timestamp,
        cpu_usage_percent,
        ram_usage_percent,
        io_read_bytes_per_s,
        io_write_bytes_per_s,
        top_processes,
    })
}

/// Read aggregated CPU times from /proc/stat.
fn read_raw_cpu_times() -> Result<CpuTimes, Box<dyn Error>> {
    let contents = fs::read_to_string("/proc/stat")?;
    let mut lines = contents.lines();

    let first_line = lines
        .next()
        .ok_or("Empty /proc/stat or unexpected format")?;

    let mut parts = first_line.split_whitespace();

    let tag = parts.next().ok_or("Malformed 'cpu' line in /proc/stat")?;
    if tag != "cpu" {
        return Err("First line in /proc/stat does not start with 'cpu'".into());
    }

    let mut values: Vec<u64> = Vec::new();
    for p in parts {
        if let Ok(v) = p.parse::<u64>() {
            values.push(v);
        }
    }

    if values.len() < 4 {
        return Err("Not enough CPU fields in /proc/stat".into());
    }

    let user = values[0];
    let nice = values[1];
    let system = values[2];
    let idle = values[3];
    let iowait = values.get(4).copied().unwrap_or(0);
    let irq = values.get(5).copied().unwrap_or(0);
    let softirq = values.get(6).copied().unwrap_or(0);
    let steal = values.get(7).copied().unwrap_or(0);

    let idle_all = idle + iowait;
    let non_idle = user + nice + system + irq + softirq + steal;
    let total = idle_all + non_idle;

    Ok(CpuTimes { idle_all, total })
}

/// CPU usage (%) based on delta between calls.
fn read_cpu_usage_percent_delta() -> Result<f32, Box<dyn Error>> {
    let current = read_raw_cpu_times()?;

    let state_mutex = cpu_state();
    let mut guard = state_mutex
        .lock()
        .map_err(|_| "Failed to lock CPU state mutex")?;

    if let Some(prev) = *guard {
        let delta_total = current.total.saturating_sub(prev.total);
        let delta_idle = current.idle_all.saturating_sub(prev.idle_all);

        *guard = Some(current);

        if delta_total == 0 {
            return Ok(0.0);
        }

        let non_idle = delta_total.saturating_sub(delta_idle);
        let usage = (non_idle as f32 / delta_total as f32) * 100.0;
        Ok(usage)
    } else {
        *guard = Some(current);
        Ok(0.0)
    }
}

/// RAM usage (%) from /proc/meminfo.
fn read_ram_usage_percent() -> Result<f32, Box<dyn Error>> {
    let contents = fs::read_to_string("/proc/meminfo")?;

    let mut mem_total_kb: Option<u64> = None;
    let mut mem_available_kb: Option<u64> = None;

    for line in contents.lines() {
        if line.starts_with("MemTotal:") {
            if let Some(value_str) = line.split_whitespace().nth(1) {
                if let Ok(v) = value_str.parse::<u64>() {
                    mem_total_kb = Some(v);
                }
            }
        } else if line.starts_with("MemAvailable:") {
            if let Some(value_str) = line.split_whitespace().nth(1) {
                if let Ok(v) = value_str.parse::<u64>() {
                    mem_available_kb = Some(v);
                }
            }
        }
    }

    let mem_total = mem_total_kb.ok_or("Missing MemTotal in /proc/meminfo")?;
    let mem_available =
        mem_available_kb.ok_or("Missing MemAvailable in /proc/meminfo")?;

    if mem_total == 0 {
        return Ok(0.0);
    }

    let used = mem_total.saturating_sub(mem_available);
    let usage_percent = (used as f32 / mem_total as f32) * 100.0;

    Ok(usage_percent)
}
