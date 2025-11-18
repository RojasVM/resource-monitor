use std::time::{Duration, SystemTime};

use crate::config::{ResourceKind, Thresholds};
use crate::metrics::{ProcessSample, SystemSnapshot};

/// Spike event info for logs and alerts.
#[derive(Debug, Clone)]
pub struct SpikeEvent {
    pub resource: ResourceKind,
    pub timestamp_start: SystemTime,
    pub timestamp_end: SystemTime,
    pub peak_value: f32,
    pub threshold: f32,
    pub top_processes: Vec<ProcessSample>,
}

/// Per-resource state for spike detection.
#[derive(Debug, Clone)]
pub struct SpikeState {
    pub in_spike: bool,
    pub spike_start: Option<SystemTime>,
    pub spike_max_value: f32,
    pub spike_max_snapshot: Option<SystemSnapshot>,
}

impl SpikeState {
    pub fn new() -> Self {
        Self {
            in_spike: false,
            spike_start: None,
            spike_max_value: 0.0,
            spike_max_snapshot: None,
        }
    }

    pub fn reset(&mut self) {
        self.in_spike = false;
        self.spike_start = None;
        self.spike_max_value = 0.0;
        self.spike_max_snapshot = None;
    }
}

/// Global analyzer state for CPU, RAM and IO.
#[derive(Debug, Clone)]
pub struct AnalyzerState {
    pub cpu: SpikeState,
    pub ram: SpikeState,
    pub io: SpikeState,
}

impl AnalyzerState {
    pub fn new() -> Self {
        Self {
            cpu: SpikeState::new(),
            ram: SpikeState::new(),
            io: SpikeState::new(),
        }
    }
}

/// Analyze one snapshot and return spike events closed on this tick.
pub fn analyze_snapshot(
    snapshot: &SystemSnapshot,
    thresholds: &Thresholds,
    min_spike_duration_secs: u64,
    state: &mut AnalyzerState,
) -> Vec<SpikeEvent> {
    let mut events = Vec::new();

    // CPU
    if let Some(th) = thresholds.cpu_threshold {
        if let Some(ev) = update_spike_for_resource(
            ResourceKind::Cpu,
            snapshot.cpu_usage_percent,
            th,
            snapshot,
            min_spike_duration_secs,
            &mut state.cpu,
        ) {
            events.push(ev);
        }
    } else {
        state.cpu.reset();
    }

    // RAM
    if let Some(th) = thresholds.ram_threshold {
        if let Some(ev) = update_spike_for_resource(
            ResourceKind::Ram,
            snapshot.ram_usage_percent,
            th,
            snapshot,
            min_spike_duration_secs,
            &mut state.ram,
        ) {
            events.push(ev);
        }
    } else {
        state.ram.reset();
    }

    // IO (currently always 0.0)
    if let Some(th) = thresholds.io_threshold {
        let total_io_bytes =
            snapshot.io_read_bytes_per_s + snapshot.io_write_bytes_per_s;
        let io_mb_per_s = (total_io_bytes / 1_000_000.0) as f32;

        if let Some(ev) = update_spike_for_resource(
            ResourceKind::Io,
            io_mb_per_s,
            th,
            snapshot,
            min_spike_duration_secs,
            &mut state.io,
        ) {
            events.push(ev);
        }
    } else {
        state.io.reset();
    }

    events
}

/// Core spike state machine for one resource.
fn update_spike_for_resource(
    resource: ResourceKind,
    value: f32,
    threshold: f32,
    snapshot: &SystemSnapshot,
    min_spike_duration_secs: u64,
    state: &mut SpikeState,
) -> Option<SpikeEvent> {
    let now = snapshot.timestamp;

    // Not in spike yet
    if !state.in_spike {
        if value >= threshold {
            state.in_spike = true;
            state.spike_start = Some(now);
            state.spike_max_value = value;
            state.spike_max_snapshot = Some(snapshot.clone());
        }
        return None;
    }

    // Already in spike
    if value >= threshold {
        if value > state.spike_max_value {
            state.spike_max_value = value;
            state.spike_max_snapshot = Some(snapshot.clone());
        }
        return None;
    }

    // Spike ended (value dropped below threshold)
    let start = match state.spike_start {
        Some(ts) => ts,
        None => {
            state.reset();
            return None;
        }
    };

    let duration = match now.duration_since(start) {
        Ok(d) => d,
        Err(_) => Duration::from_secs(0),
    };

    let mut event: Option<SpikeEvent> = None;

    if duration.as_secs() >= min_spike_duration_secs {
        let top_processes = state
            .spike_max_snapshot
            .as_ref()
            .map(|snap| snap.top_processes.clone())
            .unwrap_or_else(Vec::new);

        event = Some(SpikeEvent {
            resource,
            timestamp_start: start,
            timestamp_end: now,
            peak_value: state.spike_max_value,
            threshold,
            top_processes,
        });
    }

    state.reset();
    event
}
