# mavsnark

A CLI tool that functions as a Wireshark-like application for [MAVLink](https://mavlink.io/en/) — inspect, filter, and analyze MAVLink traffic directly from your terminal.

## Test Environment

A Docker Compose setup provides realistic MAVLink traffic from PX4 SITL for development and testing.

```
PX4 SITL (container) --> mavp2p (container) --> QGroundControl (host, :14550)
                                            --> mavsnark (host, :14540)
```

**Start:**

```sh
docker compose up
```

**Stop:**

```sh
docker compose down
```

**Connect QGroundControl:** Add a manual comm link to `localhost:14550` (UDP).

**Connect mavsnark:** Point it at `localhost:14540`.

> **Note:** The first run downloads ~8 GB of Docker images. PX4 boot takes a few minutes — wait for MAVLink heartbeats before connecting. x86_64 only (no ARM build for the PX4 Gazebo image).
