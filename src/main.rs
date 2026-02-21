mod app;
mod collector;
mod connection;
mod message;

use std::{io, sync::mpsc, thread};

use clap::Parser;

use message::MavMsg;

#[derive(Parser)]
#[command(name = "mavsnark", about = "wireshark for mavlink")]
struct Args {
    /// MAVLink connection URI
    #[arg(short, long, default_value = "udpin:0.0.0.0:14445")]
    uri: String,
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let (tx, rx) = mpsc::channel();

    let connection = connection::connect(&args.uri).map_err(|e| {
        eprintln!("error: {e}");
        e
    })?;

    thread::spawn(move || {
        loop {
            match connection.recv() {
                Ok((header, msg)) => {
                    if tx.send(MavMsg::new(header, msg)).is_err() {
                        break;
                    }
                }
                Err(_) => {}
            }
        }
    });

    let mut terminal = ratatui::init();
    let mut app = app::App::new();
    let result = app.run(&mut terminal, rx);
    ratatui::restore();
    result
}
