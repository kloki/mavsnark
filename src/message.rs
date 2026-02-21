use chrono::{DateTime, Utc};
use mavlink::common::MavMessage;
use mavlink::{MavHeader, Message};
use ratatui::style::Color;

const COLORS: &[Color] = &[
    Color::Red,
    Color::Green,
    Color::Yellow,
    Color::Blue,
    Color::Magenta,
    Color::Cyan,
];

pub struct MavMsg {
    pub header: MavHeader,
    pub msg: MavMessage,
    pub timestamp: DateTime<Utc>,
}

impl MavMsg {
    pub fn new(header: MavHeader, msg: MavMessage) -> Self {
        Self {
            header,
            msg,
            timestamp: Utc::now(),
        }
    }

    pub fn color(&self) -> Color {
        let idx = (self.header.system_id as usize * 31 + self.header.component_id as usize)
            % COLORS.len();
        COLORS[idx]
    }

    pub fn msg_color(&self) -> Option<Color> {
        match self.msg {
            MavMessage::HEARTBEAT(..) => Some(Color::Magenta),
            MavMessage::MANUAL_CONTROL(..) => Some(Color::Green),
            MavMessage::ATTITUDE(..) | MavMessage::GLOBAL_POSITION_INT(..) => Some(Color::Blue),
            _ => None,
        }
    }

    pub fn msg_type(&self) -> &'static str {
        self.msg.message_name()
    }

    pub fn fields(&self) -> String {
        let debug = format!("{:?}", self.msg);
        let start = debug.find('{').map(|i| i + 1).unwrap_or(0);
        let end = debug.rfind('}').unwrap_or(debug.len());
        debug[start..end].trim().to_string()
    }

    #[allow(deprecated)]
    pub fn is_event(&self) -> bool {
        matches!(
            self.msg,
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
                // Discrete SET_* messages
                | MavMessage::SET_MODE(..)
                | MavMessage::SET_GPS_GLOBAL_ORIGIN(..)
                | MavMessage::SET_HOME_POSITION(..)
                // Param set
                | MavMessage::PARAM_SET(..)
                | MavMessage::PARAM_EXT_SET(..)
                // Safety
                | MavMessage::SAFETY_SET_ALLOWED_AREA(..)
        )
    }
}
