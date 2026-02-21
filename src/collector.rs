use std::collections::HashMap;
use std::time::Instant;

use ratatui::style::Color;

use crate::message::MavMsg;

pub struct TelemetryEntry {
    pub color: Color,
    pub text: String,
    pub count: usize,
    pub first_seen: Instant,
}

pub struct CommandEntry {
    pub color: Color,
    pub text: String,
    pub timestamp: Instant,
}

pub struct Collector {
    telemetry: HashMap<String, TelemetryEntry>,
    commands: Vec<CommandEntry>,
}

impl Collector {
    pub fn new() -> Self {
        Self {
            telemetry: HashMap::new(),
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
            let key = format!(
                "{}:{}:{}",
                msg.header.system_id, msg.header.component_id, msg.msg_type()
            );
            let entry = self.telemetry.entry(key).or_insert(TelemetryEntry {
                color,
                text: String::new(),
                count: 0,
                first_seen: timestamp,
            });
            entry.color = color;
            entry.text = text;
            entry.count += 1;
        }
    }

    pub fn telemetry_sorted(&self) -> Vec<(&str, &TelemetryEntry)> {
        let mut entries: Vec<_> = self
            .telemetry
            .iter()
            .map(|(k, v)| (k.as_str(), v))
            .collect();
        entries.sort_by(|a, b| a.1.first_seen.cmp(&b.1.first_seen));
        entries
    }

    pub fn telemetry_count(&self) -> usize {
        self.telemetry.len()
    }

    pub fn commands(&self) -> &[CommandEntry] {
        &self.commands
    }
}
