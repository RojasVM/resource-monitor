# resource-monitor

A small resource spike monitor for Linux written in Rust.

It samples CPU and RAM usage from /proc, detects spikes based on
user-defined thresholds, prints information to the terminal, and
optionally logs spike events as JSON lines to a file.

  Status: experimental / WIP
  IO and per-process tracking are not implemented yet.

------------------------------------------------------------------------

Features

-   Linux-only (reads directly from /proc)
-   CLI interface using clap
-   Three modes:
    -   live: continuous monitoring until interrupted
    -   batch: run for a fixed duration or number of samples
    -   logs: inspect previously recorded spike events
-   User-defined thresholds for:
    -   CPU usage (%)
    -   RAM usage (%)
    -   IO throughput (MB/s) — planned
-   Spike events include:
    -   Start and end timestamps
    -   Duration
    -   Peak value
    -   Threshold exceeded
    -   (Planned) top processes at peak
-   Colored output in text mode
-   JSON output mode for easy piping/processing
-   JSON-lines logging for spikes

------------------------------------------------------------------------

Build

Requirements:

-   Rust toolchain (stable)
-   Linux system with /proc available

Clone and build:

    git clone https://github.com/RojasVM/resource-monitor.git
    cd resource_monitor
    cargo build --release

The binary will be in:

    target/release/resource_monitor

------------------------------------------------------------------------

Usage

Global help

    resource_monitor --help

------------------------------------------------------------------------

Live mode

Continuously prints one line per interval with CPU/RAM usage and
optional spike alerts.

    resource_monitor live --interval-ms 1000 --cpu-threshold 80 --ram-threshold 90

Options:

-   --interval-ms <u64>: sampling interval in milliseconds (default:
    1000)
-   --cpu-threshold <f32>: CPU spike threshold in percent
-   --ram-threshold <f32>: RAM spike threshold in percent
-   --io-threshold <f32>: IO spike threshold in MB/s (not implemented
    yet)
-   --min-spike-duration-secs <u64>: minimum spike duration in seconds
    (default: 3)
-   --output text|json: output format (default: text)
-   --log-file <path>: append spike events to given log file
    (JSON-lines)
-   --top-n-procs <usize>: number of top processes to record (not
    implemented yet)

Example:

    resource_monitor live   --interval-ms 1000   --cpu-threshold 50   --ram-threshold 70   --output text   --log-file monitor.log

------------------------------------------------------------------------

Batch mode

Same as live, but stops after a certain number of samples or seconds.

    resource_monitor batch --samples 10 --cpu-threshold 20

or

    resource_monitor batch --duration-secs 30 --cpu-threshold 20

Options:

-   --interval-ms <u64>: sampling interval in ms (default: 1000)
-   --duration-secs <u64>: total duration in seconds (exclusive with
    --samples)
-   --samples <u64>: total number of samples (exclusive with
    --duration-secs)
-   The same threshold/output/log options as in live

If neither --duration-secs nor --samples is provided, batch will default
to 10 samples.

------------------------------------------------------------------------

Logs mode

Reads the log file (JSON-lines) and prints stored spike events.

    resource_monitor logs --log-file monitor.log

Options:

-   --log-file <path>: log file to read
-   --resource cpu|ram|io: filter events by resource type
-   --since <u64>: minimum ts_start (seconds since epoch)
-   --until <u64>: maximum ts_start (seconds since epoch)
-   --limit <usize>: maximum number of events to display
-   --output text|json: output format (default: text)

Examples:

    resource_monitor logs --log-file monitor.log
    resource_monitor logs --log-file monitor.log --resource cpu --output json
    resource_monitor logs --log-file monitor.log --resource ram --limit 5

------------------------------------------------------------------------

Log format

Each spike event is written as a single JSON line:

    {"resource":"cpu","ts_start":1731853000,"ts_end":1731853005,"duration_secs":5,"peak":92.35,"threshold":80.0,"top":[]}

------------------------------------------------------------------------

TODO

-   ☐ Real IO throughput calculation from /proc/diskstats
-   ☐ Top processes at spike peak using /proc/<pid>
-   ☐ Config file support
-   ☐ More advanced filters in logs mode
-   ☐ Unit tests for CPU/RAM/IO parsing
