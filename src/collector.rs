use std::collections::{HashMap, HashSet};

use crate::{
    entries::{MessageEntry, StreamEntry},
    message::MavMsg,
};

type StreamKey = (u8, u8, &'static str);

const DEFAULT_MESSAGE_TYPES: &[&str] = &[
    "COMMAND_INT",
    "COMMAND_LONG",
    "COMMAND_ACK",
    "COMMAND_CANCEL",
    "MISSION_ITEM",
    "MISSION_ITEM_INT",
    "MISSION_REQUEST",
    "MISSION_REQUEST_INT",
    "MISSION_REQUEST_LIST",
    "MISSION_REQUEST_PARTIAL_LIST",
    "MISSION_SET_CURRENT",
    "MISSION_WRITE_PARTIAL_LIST",
    "MISSION_COUNT",
    "MISSION_CLEAR_ALL",
    "MISSION_ACK",
    "SET_MODE",
    "SET_GPS_GLOBAL_ORIGIN",
    "SET_HOME_POSITION",
    "PARAM_SET",
    "PARAM_EXT_SET",
    "SAFETY_SET_ALLOWED_AREA",
];

pub struct Collector {
    stream: Vec<StreamEntry>,
    stream_index: HashMap<StreamKey, usize>,
    messages: Vec<MessageEntry>,
    message_types: HashSet<&'static str>,
}

impl Collector {
    pub fn new() -> Self {
        Self {
            stream: Vec::new(),
            stream_index: HashMap::new(),
            messages: Vec::new(),
            message_types: DEFAULT_MESSAGE_TYPES.iter().copied().collect(),
        }
    }

    pub fn push(&mut self, msg: MavMsg) {
        let color = msg.color();
        let msg_color = msg.msg_color();
        let sys_id = msg.header.system_id;
        let comp_id = msg.header.component_id;
        let name = msg.msg_type();
        let fields = msg.fields();
        let timestamp = msg.timestamp;

        if self.message_types.contains(name) {
            self.messages.push(MessageEntry {
                color,
                msg_color,
                sys_id,
                comp_id,
                name,
                fields,
            });
        } else {
            let key = (sys_id, comp_id, name);
            if let Some(&idx) = self.stream_index.get(&key) {
                let entry = &mut self.stream[idx];
                entry.color = color;
                entry.msg_color = msg_color;
                entry.fields = fields;
                entry.timestamp = timestamp;
            } else {
                let idx = self.stream.len();
                self.stream_index.insert(key, idx);
                self.stream.push(StreamEntry {
                    color,
                    msg_color,
                    sys_id,
                    comp_id,
                    name,
                    fields,
                    timestamp,
                });
            }
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
            self.message_types.insert(name);
            self.stream.retain(|e| e.name != name);
            self.rebuild_stream_index();
        } else {
            self.message_types.remove(name);
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
