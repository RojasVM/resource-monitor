# resource-monitor
A small resource spike monitor for Linux written in Rust.  It samples CPU and RAM usage from /proc, detects spikes based on user-defined thresholds, prints information to the terminal, and optionally logs spike events as JSON lines to a file. Status: experimental / WIP IO and per-process tracking are not implemented yet.
