mod app;
mod collector;
mod connection;
mod entries;
mod message;
mod scroll;

use std::{io, sync::mpsc, thread};

use clap::Parser;
use crossterm::event::{self, Event};
use message::MavMsg;

pub enum AppEvent {
    Mav(Box<MavMsg>),
    Terminal(Event),
}

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

    let mav_tx = tx.clone();
    thread::spawn(move || {
        loop {
            match connection.recv() {
                Ok((header, msg)) => {
                    if mav_tx
                        .send(AppEvent::Mav(Box::new(MavMsg::new(header, msg))))
                        .is_err()
                    {
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

    thread::spawn(move || {
        while let Ok(ev) = event::read() {
            if tx.send(AppEvent::Terminal(ev)).is_err() {
                break;
            }
        }
    });

    let mut terminal = ratatui::init();
    let mut app = app::App::new();
    let result = app.run(&mut terminal, rx);
    ratatui::restore();
    result
}
