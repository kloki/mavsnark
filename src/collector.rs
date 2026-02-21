use std::{collections::HashMap, time::Instant};

use ratatui::style::Color;

use crate::message::MavMsg;

pub struct TelemetryEntry {
    pub color: Color,
    pub text: String,
    pub timestamp: Instant,
}

pub struct CommandEntry {
    pub color: Color,
    pub text: String,
    pub timestamp: Instant,
}

type TelemetryKey = (u8, u8, &'static str);

pub struct Collector {
    telemetry: Vec<TelemetryEntry>,
    telemetry_index: HashMap<TelemetryKey, usize>,
    commands: Vec<CommandEntry>,
}

impl Collector {
    pub fn new() -> Self {
        Self {
            telemetry: Vec::new(),
            telemetry_index: HashMap::new(),
            commands: Vec::new(),
        }
    }

    pub fn push(&mut self, msg: MavMsg) {
        let color = msg.color();
        let text = msg.text();
        let timestamp = msg.timestamp;

        if msg.is_command() {
            self.commands.push(CommandEntry {
                color,
                text,
                timestamp,
            });
        } else {
            let key = (
                msg.header.system_id,
                msg.header.component_id,
                msg.msg_type(),
            );
            if let Some(&idx) = self.telemetry_index.get(&key) {
                let entry = &mut self.telemetry[idx];
                entry.color = color;
                entry.text = text;
            } else {
                let idx = self.telemetry.len();
                self.telemetry_index.insert(key, idx);
                self.telemetry.push(TelemetryEntry {
                    color,
                    text,
                    timestamp: timestamp,
                });
            }
        }
    }

    pub fn telemetry(&self) -> &[TelemetryEntry] {
        &self.telemetry
    }

    pub fn commands(&self) -> &[CommandEntry] {
        &self.commands
    }
}
