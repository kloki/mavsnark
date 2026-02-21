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
}

impl MavMsg {
    pub fn new(header: MavHeader, msg: MavMessage) -> Self {
        Self { header, msg }
    }

    pub fn color(&self) -> Color {
        let idx = (self.header.system_id as usize * 31 + self.header.component_id as usize)
            % COLORS.len();
        COLORS[idx]
    }

    pub fn msg_type(&self) -> &'static str {
        self.msg.message_name()
    }

    pub fn text(&self) -> String {
        format!(
            "[SYS:{} COMP:{}] {:?}",
            self.header.system_id, self.header.component_id, self.msg
        )
    }

    #[allow(deprecated)]
    pub fn is_command(&self) -> bool {
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
}
