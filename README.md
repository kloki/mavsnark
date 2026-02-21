```
 _____ _____ _ _ ___ ___ ___ ___ ___
|     |  _  | | |_ -|   | .'|  _| '_|
|_|_|_|__|__|\_/|___|_|_|__,|_| |_,_|
```

A terminal UI for inspecting [MAVLink](https://mavlink.io/en/) traffic in real time.

## Features

- **Stream panel** — one row per unique message type (keyed by system/component/name), updated in place with seconds since last message
- **Events panel** — chronological log of command, mission, param, and other one-shot messages
- **Message detail panel** — shows all fields of the currently selected message
- Color-coded by system/component ID and message category
- Selection cursor with Vim-style navigation (`j`/`k`, `g`/`G`, `PgUp`/`PgDn`) and `Tab`/arrow keys to switch panels
- `Ctrl+O` opens the MAVLink docs for the selected message
- MAVLink v2, any connection URI supported by the `mavlink` crate (`udpin:`, `tcpin:`, `serial:`, etc.)

# Install

Check [Releases](https://github.com/kloki/mavsnark/releases) for binaries and installers

## Usage

```
mavsnark --uri udpin:0.0.0.0:14445
```

The default URI is `udpin:0.0.0.0:14445`.

## Test setup

Enable telemetry forwarding in QGroundControl, then:

```
docker run --rm -it jonasvautherin/px4-gazebo-headless:1.16.1
```
