# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

mavsnark is a terminal UI for inspecting MAVLink drone protocol traffic in real time. It connects to any MAVLink source (UDP, TCP, serial) and displays a split-panel TUI with stream (telemetry) and events (commands/missions).

## Build & Run

```bash
cargo build
cargo run -- --uri udpin:0.0.0.0:14445   # default URI if omitted
cargo clippy                               # lint
cargo fmt                                  # format
```

No tests exist yet.

### Test setup (requires QGroundControl with telemetry forwarding enabled)

```bash
docker run --rm -it jonasvautherin/px4-gazebo-headless:1.16.1
```

## Architecture

**Threading model:** `main.rs` spawns a background thread that reads MAVLink packets from the connection and sends `MavMsg` values over an `mpsc::channel` to the UI thread, which polls at 50ms intervals.

**Message classification** (`message.rs`): Every incoming MAVLink message is wrapped in `MavMsg` (adding a timestamp and source color). `is_event()` classifies command/mission/param-set messages as discrete events; everything else is telemetry stream data.

**Collection** (`collector.rs`): `Collector` maintains two data structures:
- **Stream:** insertion-ordered `Vec<StreamEntry>` with a `HashMap<(sys_id, comp_id, msg_name), index>` for O(1) upsert. Only the latest value per key is kept.
- **Events:** append-only `Vec<EventEntry>`.

**UI** (`app.rs`): ratatui-based TUI with a 50/50 horizontal split. Each panel has independent `ScrollState` with auto-scroll that disables on manual scroll and re-enables when scrolled to bottom. Vim-style keybindings (`j/k/g/G/PgUp/PgDn`, `Tab`/`h`/`l` to switch panels).

**Connection** (`connection.rs`): Thin wrapper around `mavlink::connect()`, sets protocol to V2, returns a trait object.

## Conventions

- MAVLink crate uses the `common` feature set (not `ardupilotmega` or other dialects)
- `MavMsg::fields()` extracts message fields by parsing `Debug` output — this is intentionally simple but brittle
- `#[allow(deprecated)]` on `is_event()` because some MAVLink message variants are deprecated upstream
- Colors are deterministic per (system_id, component_id) pair via simple hash into a 6-color palette
