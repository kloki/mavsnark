use colored::Colorize;
use mavlink::{MavConnection, common::MavMessage};

const COLORS: &[&str] = &["red", "green", "yellow", "blue", "magenta", "cyan"];

fn color_for(system_id: u8, component_id: u8) -> &'static str {
    let idx = (system_id as usize * 31) % COLORS.len();
    COLORS[idx]
}

fn main() {
    let mut connection = mavlink::connect::<MavMessage>("udpin:0.0.0.0:14540")
        .expect("failed to connect to udpin:0.0.0.0:1450");

    connection.set_protocol_version(mavlink::MavlinkVersion::V2);

    loop {
        match connection.recv() {
            Ok((header, msg)) => {
                let label = format!("[SYS:{} COMP:{}]", header.system_id, header.component_id);
                let line = format!("{label} {msg:?}");
                let colored = match color_for(header.system_id, header.component_id) {
                    "red" => line.red(),
                    "green" => line.green(),
                    "yellow" => line.yellow(),
                    "blue" => line.blue(),
                    "magenta" => line.magenta(),
                    "cyan" => line.cyan(),
                    _ => line.white(),
                };
                println!("{colored}");
            }
            Err(e) => {
                eprintln!("recv error: {e}");
            }
        }
    }
}
