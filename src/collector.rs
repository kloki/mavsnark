use std::collections::HashMap;

use chrono::{DateTime, Utc};
use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

use crate::message::MavMsg;

pub struct StreamEntry {
    pub color: Color,
    pub msg_color: Option<Color>,
    pub sys_id: u8,
    pub comp_id: u8,
    pub name: &'static str,
    pub fields: String,
    pub timestamp: DateTime<Utc>,
}

impl StreamEntry {
    pub fn to_line(&self) -> Line<'_> {
        let colored = Style::default().fg(self.color);
        let ago = Utc::now()
            .signed_duration_since(self.timestamp)
            .num_milliseconds() as f64
            / 1000.0;
        let gray = Style::default().fg(Color::DarkGray);
        let msg_style = match self.msg_color {
            Some(c) => Style::default().fg(c),
            None => Style::default(),
        };
        Line::from(vec![
            Span::raw("["),
            Span::styled(format!("{:>3}", self.sys_id), colored),
            Span::raw(":"),
            Span::styled(format!("{:>3}", self.comp_id), colored),
            Span::raw("] "),
            Span::styled(format!("{ago:>6.1}s "), gray),
            Span::styled(format!("{}: {}", self.name, self.fields), msg_style),
        ])
    }
}

pub struct EventEntry {
    pub color: Color,
    pub msg_color: Option<Color>,
    pub sys_id: u8,
    pub comp_id: u8,
    pub name: &'static str,
    pub fields: String,
}

impl EventEntry {
    pub fn to_line(&self) -> Line<'_> {
        let colored = Style::default().fg(self.color);
        let msg_style = match self.msg_color {
            Some(c) => Style::default().fg(c),
            None => Style::default(),
        };
        Line::from(vec![
            Span::raw("["),
            Span::styled(format!("{:>3}", self.sys_id), colored),
            Span::raw(":"),
            Span::styled(format!("{:>3}", self.comp_id), colored),
            Span::raw("] "),
            Span::styled(format!("{}: {}", self.name, self.fields), msg_style),
        ])
    }
}

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
