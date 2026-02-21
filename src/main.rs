mod app;
mod connection;

use std::io;
use std::sync::mpsc;
use std::thread;

use clap::Parser;
use mavlink::Message;
use mavlink::common::MavMessage;

#[derive(Parser)]
#[command(name = "mavsnark", about = "wireshark for mavlink")]
struct Args {
    /// MAVLink connection URI
    #[arg(short, long, default_value = "udpin:0.0.0.0:14445")]
    uri: String,
}

#[allow(deprecated)]
fn is_command(msg: &MavMessage) -> bool {
    matches!(
        msg,
        // Command protocol
        MavMessage::COMMAND_INT(..)
            | MavMessage::COMMAND_LONG(..)
            | MavMessage::COMMAND_ACK(..)
            | MavMessage::COMMAND_CANCEL(..)
            // Mission protocol
            | MavMessage::MISSION_ITEM(..)
            | MavMessage::MISSION_ITEM_INT(..)
            | MavMessage::MISSION_REQUEST(..)
            | MavMessage::MISSION_REQUEST_INT(..)
            | MavMessage::MISSION_REQUEST_LIST(..)
            | MavMessage::MISSION_REQUEST_PARTIAL_LIST(..)
            | MavMessage::MISSION_SET_CURRENT(..)
            | MavMessage::MISSION_WRITE_PARTIAL_LIST(..)
            | MavMessage::MISSION_COUNT(..)
            | MavMessage::MISSION_CLEAR_ALL(..)
            | MavMessage::MISSION_ACK(..)
            // SET_* messages
            | MavMessage::SET_MODE(..)
            | MavMessage::SET_ATTITUDE_TARGET(..)
            | MavMessage::SET_POSITION_TARGET_LOCAL_NED(..)
            | MavMessage::SET_POSITION_TARGET_GLOBAL_INT(..)
            | MavMessage::SET_ACTUATOR_CONTROL_TARGET(..)
            | MavMessage::SET_GPS_GLOBAL_ORIGIN(..)
            | MavMessage::SET_HOME_POSITION(..)
            // Manual control
            | MavMessage::MANUAL_CONTROL(..)
            | MavMessage::MANUAL_SETPOINT(..)
            | MavMessage::RC_CHANNELS_OVERRIDE(..)
            // Param set
            | MavMessage::PARAM_SET(..)
            | MavMessage::PARAM_EXT_SET(..)
            // Safety
            | MavMessage::SAFETY_SET_ALLOWED_AREA(..)
            // Gimbal set
            | MavMessage::GIMBAL_DEVICE_SET_ATTITUDE(..)
            | MavMessage::GIMBAL_MANAGER_SET_ATTITUDE(..)
            | MavMessage::GIMBAL_MANAGER_SET_MANUAL_CONTROL(..)
            | MavMessage::GIMBAL_MANAGER_SET_PITCHYAW(..)
    )
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let connection = connection::connect(&args.uri);
        loop {
            match connection.recv() {
                Ok((header, msg)) => {
                    let color = app::color_for(header.system_id, header.component_id);
                    let text = format!(
                        "[SYS:{} COMP:{}] {:?}",
                        header.system_id, header.component_id, msg
                    );
                    let message = app::Message {
                        color,
                        msg_type: msg.message_name().to_string(),
                        is_command: is_command(&msg),
                        text,
                    };
                    if tx.send(message).is_err() {
                        break;
                    }
                }
                Err(_) => {}
            }
        }
    });

    let mut terminal = ratatui::init();
    let result = app::run(&mut terminal, rx);
    ratatui::restore();
    result
}
