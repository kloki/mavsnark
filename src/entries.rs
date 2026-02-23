use chrono::{DateTime, Utc};
use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

pub(crate) fn parse_fields(s: &str) -> Vec<(&str, &str)> {
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
    pub sys_color: Color,
    pub comp_color: Color,
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
        let sys_style = Style::default().fg(self.sys_color);
        let comp_style = Style::default().fg(self.comp_color);
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
            Span::styled(format!("{:>3}", self.sys_id), sys_style),
            Span::raw(":"),
            Span::styled(format!("{:>3}", self.comp_id), comp_style),
            Span::raw("] "),
            Span::styled(format!("{ago:>6.1}s "), gray),
            Span::styled(format!("{}: {}", self.name, self.fields), msg_style),
        ])
    }
}

pub struct MessageEntry {
    pub sys_color: Color,
    pub comp_color: Color,
    pub msg_color: Option<Color>,
    pub sys_id: u8,
    pub comp_id: u8,
    pub name: &'static str,
    pub fields: String,
}

impl MessageEntry {
    pub fn parsed_fields(&self) -> Vec<(&str, &str)> {
        parse_fields(&self.fields)
    }

    pub fn to_line(&self) -> Line<'_> {
        let sys_style = Style::default().fg(self.sys_color);
        let comp_style = Style::default().fg(self.comp_color);
        let msg_style = match self.msg_color {
            Some(c) => Style::default().fg(c),
            None => Style::default(),
        };
        Line::from(vec![
            Span::raw("["),
            Span::styled(format!("{:>3}", self.sys_id), sys_style),
            Span::raw(":"),
            Span::styled(format!("{:>3}", self.comp_id), comp_style),
            Span::raw("] "),
            Span::styled(format!("{}: {}", self.name, self.fields), msg_style),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_basic_fields() {
        let result = parse_fields("throttle: 500, yaw: 0.0");
        assert_eq!(result, vec![("throttle", "500"), ("yaw", "0.0")]);
    }

    #[test]
    fn parse_empty_string() {
        let result = parse_fields("");
        assert!(result.is_empty());
    }

    #[test]
    fn parse_trailing_comma() {
        let result = parse_fields("a: 1,");
        assert_eq!(result, vec![("a", "1")]);
    }

    #[test]
    fn parse_no_colon() {
        let result = parse_fields("garbage");
        assert!(result.is_empty());
    }

    #[test]
    fn parsed_fields_on_stream_entry() {
        let entry = StreamEntry {
            sys_color: Color::Red,
            comp_color: Color::Cyan,
            msg_color: None,
            sys_id: 1,
            comp_id: 1,
            name: "TEST",
            fields: "x: 10, y: 20".to_string(),
            timestamp: Utc::now(),
        };
        let fields = entry.parsed_fields();
        assert_eq!(fields, vec![("x", "10"), ("y", "20")]);
    }

    #[test]
    fn parsed_fields_on_message_entry() {
        let entry = MessageEntry {
            sys_color: Color::Red,
            comp_color: Color::Cyan,
            msg_color: None,
            sys_id: 1,
            comp_id: 1,
            name: "TEST",
            fields: "cmd: 42".to_string(),
        };
        let fields = entry.parsed_fields();
        assert_eq!(fields, vec![("cmd", "42")]);
    }
}
