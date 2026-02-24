use std::collections::{HashMap, HashSet};

use crate::{
    entries::{MessageEntry, StreamEntry},
    message::MavMsg,
};

type StreamKey = (u8, u8, &'static str);

const DEFAULT_STREAM_TYPES: &[&str] = &[
    "HEARTBEAT",
    "SYS_STATUS",
    "SYSTEM_TIME",
    "GPS_RAW_INT",
    "GPS_STATUS",
    "GPS2_RAW",
    "RAW_IMU",
    "SCALED_IMU",
    "SCALED_IMU2",
    "SCALED_IMU3",
    "HIGHRES_IMU",
    "RAW_PRESSURE",
    "SCALED_PRESSURE",
    "SCALED_PRESSURE2",
    "SCALED_PRESSURE3",
    "ATTITUDE",
    "ATTITUDE_QUATERNION",
    "LOCAL_POSITION_NED",
    "GLOBAL_POSITION_INT",
    "POSITION_TARGET_LOCAL_NED",
    "POSITION_TARGET_GLOBAL_INT",
    "RC_CHANNELS",
    "RC_CHANNELS_RAW",
    "SERVO_OUTPUT_RAW",
    "MANUAL_CONTROL",
    "VFR_HUD",
    "NAV_CONTROLLER_OUTPUT",
    "BATTERY_STATUS",
    "POWER_STATUS",
    "ALTITUDE",
    "ESTIMATOR_STATUS",
    "VIBRATION",
    "HOME_POSITION",
    "EXTENDED_SYS_STATE",
    "WIND_COV",
    "TERRAIN_REPORT",
    "DISTANCE_SENSOR",
    "OPTICAL_FLOW",
    "ODOMETRY",
    "UTM_GLOBAL_POSITION",
    "MISSION_CURRENT",
    "AUTOPILOT_VERSION",
    "TIMESYNC",
    "PING",
    "LINK_NODE_STATUS",
    "ACTUATOR_OUTPUT_STATUS",
    "FLIGHT_INFORMATION",
];

pub struct Collector {
    stream: Vec<StreamEntry>,
    stream_index: HashMap<StreamKey, usize>,
    messages: Vec<MessageEntry>,
    stream_types: HashSet<&'static str>,
}

impl Collector {
    pub fn new() -> Self {
        Self {
            stream: Vec::new(),
            stream_index: HashMap::new(),
            messages: Vec::new(),
            stream_types: DEFAULT_STREAM_TYPES.iter().copied().collect(),
        }
    }

    pub fn push(&mut self, msg: MavMsg) {
        let sys_color = msg.sys_color();
        let comp_color = msg.comp_color();
        let msg_color = msg.msg_color();
        let sys_id = msg.header.system_id;
        let comp_id = msg.header.component_id;
        let name = msg.msg_type();
        let fields = msg.fields();
        let timestamp = msg.timestamp;

        if self.stream_types.contains(name) {
            let key = (sys_id, comp_id, name);
            if let Some(&idx) = self.stream_index.get(&key) {
                let entry = &mut self.stream[idx];
                entry.sys_color = sys_color;
                entry.comp_color = comp_color;
                entry.msg_color = msg_color;
                entry.fields = fields;
                entry.timestamp = timestamp;
            } else {
                let idx = self.stream.len();
                self.stream_index.insert(key, idx);
                self.stream.push(StreamEntry {
                    sys_color,
                    comp_color,
                    msg_color,
                    sys_id,
                    comp_id,
                    name,
                    fields,
                    timestamp,
                });
            }
        } else {
            self.messages.push(MessageEntry {
                sys_color,
                comp_color,
                msg_color,
                sys_id,
                comp_id,
                name,
                fields,
            });
        }
    }

    pub fn stream(&self) -> &[StreamEntry] {
        &self.stream
    }

    pub fn messages(&self) -> &[MessageEntry] {
        &self.messages
    }

    pub fn toggle_category(&mut self, name: &'static str, currently_stream: bool) {
        if currently_stream {
            self.stream_types.remove(name);
            self.stream.retain(|e| e.name != name);
            self.rebuild_stream_index();
        } else {
            self.stream_types.insert(name);
            self.messages.retain(|e| e.name != name);
        }
    }

    fn rebuild_stream_index(&mut self) {
        self.stream_index.clear();
        for (i, entry) in self.stream.iter().enumerate() {
            self.stream_index
                .insert((entry.sys_id, entry.comp_id, entry.name), i);
        }
    }

    pub fn clear(&mut self) {
        self.stream.clear();
        self.stream_index.clear();
        self.messages.clear();
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use mavlink::{MavHeader, common::MavMessage};

    use super::*;

    fn make_msg(msg: MavMessage, sys_id: u8, comp_id: u8) -> MavMsg {
        MavMsg {
            header: MavHeader {
                system_id: sys_id,
                component_id: comp_id,
                sequence: 0,
            },
            msg,
            timestamp: Utc::now(),
        }
    }

    #[test]
    fn new_collector_is_empty() {
        let c = Collector::new();
        assert!(c.stream().is_empty());
        assert!(c.messages().is_empty());
    }

    #[test]
    fn push_stream_message() {
        let mut c = Collector::new();
        let msg = make_msg(
            MavMessage::HEARTBEAT(mavlink::common::HEARTBEAT_DATA::default()),
            1,
            1,
        );
        c.push(msg);
        assert_eq!(c.stream().len(), 1);
        assert!(c.messages().is_empty());
    }

    #[test]
    fn push_discrete_message() {
        let mut c = Collector::new();
        let msg = make_msg(
            MavMessage::COMMAND_LONG(mavlink::common::COMMAND_LONG_DATA::default()),
            1,
            1,
        );
        c.push(msg);
        assert!(c.stream().is_empty());
        assert_eq!(c.messages().len(), 1);
    }

    #[test]
    fn stream_upsert_deduplicates() {
        let mut c = Collector::new();
        let msg1 = make_msg(
            MavMessage::HEARTBEAT(mavlink::common::HEARTBEAT_DATA::default()),
            1,
            1,
        );
        let msg2 = make_msg(
            MavMessage::HEARTBEAT(mavlink::common::HEARTBEAT_DATA::default()),
            1,
            1,
        );
        c.push(msg1);
        c.push(msg2);
        assert_eq!(c.stream().len(), 1);
    }

    #[test]
    fn stream_different_keys_preserved() {
        let mut c = Collector::new();
        c.push(make_msg(
            MavMessage::HEARTBEAT(mavlink::common::HEARTBEAT_DATA::default()),
            1,
            1,
        ));
        c.push(make_msg(
            MavMessage::ATTITUDE(mavlink::common::ATTITUDE_DATA::default()),
            1,
            1,
        ));
        assert_eq!(c.stream().len(), 2);
        assert_eq!(c.stream()[0].name, "HEARTBEAT");
        assert_eq!(c.stream()[1].name, "ATTITUDE");
    }

    #[test]
    fn mixed_stream_and_messages() {
        let mut c = Collector::new();
        c.push(make_msg(
            MavMessage::HEARTBEAT(mavlink::common::HEARTBEAT_DATA::default()),
            1,
            1,
        ));
        c.push(make_msg(
            MavMessage::COMMAND_LONG(mavlink::common::COMMAND_LONG_DATA::default()),
            1,
            1,
        ));
        c.push(make_msg(
            MavMessage::ATTITUDE(mavlink::common::ATTITUDE_DATA::default()),
            1,
            1,
        ));
        c.push(make_msg(
            MavMessage::COMMAND_ACK(mavlink::common::COMMAND_ACK_DATA::default()),
            1,
            1,
        ));
        assert_eq!(c.stream().len(), 2);
        assert_eq!(c.messages().len(), 2);
    }

    #[test]
    fn toggle_stream_to_message() {
        let mut c = Collector::new();
        c.push(make_msg(
            MavMessage::HEARTBEAT(mavlink::common::HEARTBEAT_DATA::default()),
            1,
            1,
        ));
        c.push(make_msg(
            MavMessage::ATTITUDE(mavlink::common::ATTITUDE_DATA::default()),
            1,
            1,
        ));
        assert_eq!(c.stream().len(), 2);

        c.toggle_category("HEARTBEAT", true);
        assert_eq!(c.stream().len(), 1);
        assert_eq!(c.stream()[0].name, "ATTITUDE");

        // New HEARTBEAT pushes now go to messages
        c.push(make_msg(
            MavMessage::HEARTBEAT(mavlink::common::HEARTBEAT_DATA::default()),
            1,
            1,
        ));
        assert_eq!(c.stream().len(), 1);
        assert_eq!(c.messages().len(), 1);
        assert_eq!(c.messages()[0].name, "HEARTBEAT");
    }

    #[test]
    fn toggle_message_to_stream() {
        let mut c = Collector::new();
        c.push(make_msg(
            MavMessage::COMMAND_LONG(mavlink::common::COMMAND_LONG_DATA::default()),
            1,
            1,
        ));
        c.push(make_msg(
            MavMessage::COMMAND_ACK(mavlink::common::COMMAND_ACK_DATA::default()),
            1,
            1,
        ));
        assert_eq!(c.messages().len(), 2);

        c.toggle_category("COMMAND_LONG", false);
        assert_eq!(c.messages().len(), 1);
        assert_eq!(c.messages()[0].name, "COMMAND_ACK");

        // New COMMAND_LONG pushes now go to stream
        c.push(make_msg(
            MavMessage::COMMAND_LONG(mavlink::common::COMMAND_LONG_DATA::default()),
            1,
            1,
        ));
        assert_eq!(c.stream().len(), 1);
        assert_eq!(c.stream()[0].name, "COMMAND_LONG");
    }

    #[test]
    fn double_toggle_restores_default() {
        let mut c = Collector::new();
        // HEARTBEAT starts as stream
        c.push(make_msg(
            MavMessage::HEARTBEAT(mavlink::common::HEARTBEAT_DATA::default()),
            1,
            1,
        ));
        assert_eq!(c.stream().len(), 1);

        // Toggle to message
        c.toggle_category("HEARTBEAT", true);
        assert_eq!(c.stream().len(), 0);

        // Toggle back to stream
        c.toggle_category("HEARTBEAT", false);

        // New push goes to stream again
        c.push(make_msg(
            MavMessage::HEARTBEAT(mavlink::common::HEARTBEAT_DATA::default()),
            1,
            1,
        ));
        assert_eq!(c.stream().len(), 1);
        assert!(c.messages().is_empty());
    }

    #[test]
    fn toggle_does_not_affect_other_types() {
        let mut c = Collector::new();
        c.push(make_msg(
            MavMessage::HEARTBEAT(mavlink::common::HEARTBEAT_DATA::default()),
            1,
            1,
        ));
        c.push(make_msg(
            MavMessage::ATTITUDE(mavlink::common::ATTITUDE_DATA::default()),
            1,
            1,
        ));
        c.push(make_msg(
            MavMessage::COMMAND_LONG(mavlink::common::COMMAND_LONG_DATA::default()),
            1,
            1,
        ));

        c.toggle_category("HEARTBEAT", true);
        // ATTITUDE still in stream, COMMAND_LONG still in messages
        assert_eq!(c.stream().len(), 1);
        assert_eq!(c.stream()[0].name, "ATTITUDE");
        assert_eq!(c.messages().len(), 1);
        assert_eq!(c.messages()[0].name, "COMMAND_LONG");
    }
}
