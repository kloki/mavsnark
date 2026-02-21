use clap::Parser;
use colored::Colorize;
use mavlink::{MavConnection, common::MavMessage};

#[derive(Parser)]
#[command(name = "mavsnark", about = "wireshark for mavlink")]
struct Args {
    /// MAVLink connection URI
    #[arg(short, long, default_value = "udpin:0.0.0.0:14445")]
    uri: String,
}

const COLORS: &[&str] = &["red", "green", "yellow", "blue", "magenta", "cyan"];

fn color_for(system_id: u8, component_id: u8) -> &'static str {
    let idx = (system_id as usize * 31 + component_id as usize) % COLORS.len();
    COLORS[idx]
}

fn main() {
    let args = Args::parse();

    let mut connection = mavlink::connect::<MavMessage>(&args.uri)
        .unwrap_or_else(|e| panic!("failed to connect to {}: {e}", args.uri));

    connection.set_protocol_version(mavlink::MavlinkVersion::V2);

    loop {
        match connection.recv() {
            Ok((header, msg)) => {
                if header.system_id == 1 {
                    continue;
                }
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
