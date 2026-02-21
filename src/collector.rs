use std::collections::HashMap;

use crate::entries::{EventEntry, StreamEntry};
use crate::message::MavMsg;

type StreamKey = (u8, u8, &'static str);

pub struct Collector {
    stream: Vec<StreamEntry>,
    stream_index: HashMap<StreamKey, usize>,
    events: Vec<EventEntry>,
}

impl Collector {
    pub fn new() -> Self {
        Self {
            stream: Vec::new(),
            stream_index: HashMap::new(),
            events: Vec::new(),
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

        if msg.is_event() {
            self.events.push(EventEntry {
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

    pub fn events(&self) -> &[EventEntry] {
        &self.events
    }
}
