mod app;
mod collector;
mod connection;
mod entries;
mod message;
mod scroll;

use std::{io, thread};

use clap::Parser;
use message::MavMsg;

#[derive(Parser)]
#[command(name = "mavsnark", about = "wireshark for mavlink")]
struct Args {
    /// MAVLink connection URI
    #[arg(short, long, default_value = "udpin:0.0.0.0:14445")]
    uri: String,

    /// Send heartbeat with this system ID to enable mavlink-routerd sniffer mode
    #[arg(long)]
    heartbeat: Option<u8>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> io::Result<()> {
    let args = Args::parse();

    let (tx, rx) = tokio::sync::mpsc::channel::<MavMsg>(256);

    let connection = connection::connect(&args.uri).map_err(|e| {
        eprintln!("error: {e}");
        e
    })?;

    if let Some(system_id) = args.heartbeat {
        connection::spawn_heartbeat(&connection, system_id);
    }

    let conn = connection.clone();
    thread::spawn(move || {
        loop {
            match conn.recv() {
                Ok((header, msg)) => {
                    if tx.blocking_send(MavMsg::new(header, msg)).is_err() {
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("mavlink recv error: {e}");
                    break;
                }
            }
        }
    });

    let mut terminal = ratatui::init();
    let mut app = app::App::new();
    let result = app.run(&mut terminal, rx).await;
    ratatui::restore();
    result
}
