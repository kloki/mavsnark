use chrono::{DateTime, Utc};
use mavlink::{MavHeader, Message, common::MavMessage};
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make(msg: MavMessage, sys_id: u8, comp_id: u8) -> MavMsg {
        MavMsg {
            header: MavHeader {
                system_id: sys_id,
                component_id: comp_id,
                sequence: 0,
            },
            msg,
            timestamp: chrono::Utc::now(),
        }
    }

    #[test]
    fn color_deterministic() {
        let m1 = make(
            MavMessage::HEARTBEAT(mavlink::common::HEARTBEAT_DATA::default()),
            1,
            1,
        );
        let m2 = make(
            MavMessage::HEARTBEAT(mavlink::common::HEARTBEAT_DATA::default()),
            1,
            1,
        );
        assert_eq!(m1.color(), m2.color());
    }

    #[test]
    fn color_varies_by_id() {
        let m1 = make(
            MavMessage::HEARTBEAT(mavlink::common::HEARTBEAT_DATA::default()),
            1,
            1,
        );
        let m2 = make(
            MavMessage::HEARTBEAT(mavlink::common::HEARTBEAT_DATA::default()),
            2,
            1,
        );
        // Different sys_id should (likely) produce different colors
        // With the hash formula: (1*31+1)%6=2, (2*31+1)%6=3
        assert_ne!(m1.color(), m2.color());
    }

    #[test]
    fn msg_color_heartbeat() {
        let m = make(
            MavMessage::HEARTBEAT(mavlink::common::HEARTBEAT_DATA::default()),
            1,
            1,
        );
        assert_eq!(m.msg_color(), Some(Color::Magenta));
    }

    #[test]
    fn msg_color_attitude() {
        let m = make(
            MavMessage::ATTITUDE(mavlink::common::ATTITUDE_DATA::default()),
            1,
            1,
        );
        assert_eq!(m.msg_color(), Some(Color::Blue));
    }

    #[test]
    fn msg_color_none() {
        let m = make(
            MavMessage::SYS_STATUS(mavlink::common::SYS_STATUS_DATA::default()),
            1,
            1,
        );
        assert_eq!(m.msg_color(), None);
    }

    #[test]
    fn fields_parses_debug() {
        let m = make(
            MavMessage::HEARTBEAT(mavlink::common::HEARTBEAT_DATA::default()),
            1,
            1,
        );
        let fields = m.fields();
        assert!(fields.contains("mavtype"));
    }

    #[test]
    fn msg_type_returns_name() {
        let m = make(
            MavMessage::HEARTBEAT(mavlink::common::HEARTBEAT_DATA::default()),
            1,
            1,
        );
        assert_eq!(m.msg_type(), "HEARTBEAT");
    }
}
