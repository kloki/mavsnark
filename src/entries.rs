use chrono::{DateTime, Utc};
use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

fn parse_fields(s: &str) -> Vec<(&str, &str)> {
    s.split(',')
        .filter_map(|part| {
            let part = part.trim();
            if part.is_empty() {
                return None;
            }
            let (key, value) = part.split_once(':')?;
            Some((key.trim(), value.trim()))
        })
        .collect()
}

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
    pub fn parsed_fields(&self) -> Vec<(&str, &str)> {
        parse_fields(&self.fields)
    }

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
    pub fn parsed_fields(&self) -> Vec<(&str, &str)> {
        parse_fields(&self.fields)
    }

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
