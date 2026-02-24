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
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> io::Result<()> {
    let args = Args::parse();

    let (tx, rx) = tokio::sync::mpsc::channel::<MavMsg>(256);

    let connection = connection::connect(&args.uri).map_err(|e| {
        eprintln!("error: {e}");
        e
    })?;

    thread::spawn(move || {
        loop {
            match connection.recv() {
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
