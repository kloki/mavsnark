use std::{
    io,
    sync::{LazyLock, mpsc},
};

use crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
};

use crate::{AppEvent, collector::Collector, scroll::ScrollState};

#[derive(Debug, PartialEq)]
enum Panel {
    Stream,
    Messages,
}

static HEADER: LazyLock<Paragraph<'static>> = LazyLock::new(|| {
    let style = Style::default().fg(Color::Cyan).bold();
    Paragraph::new(vec![
        Line::from(Span::styled(
            r" _____ _____ _ _ ___ ___ ___ ___ ___ ",
            style,
        )),
        Line::from(Span::styled(
            r"|     |  _  | | |_ -|   | .'|  _| '_|",
            style,
        )),
        Line::from(Span::styled(
            r"|_|_|_|__|__|\_/|___|_|_|__,|_| |_,_|",
            style,
        )),
    ])
});

static FOOTER: LazyLock<Paragraph<'static>> = LazyLock::new(|| {
    let key = Style::default().fg(Color::Cyan).bold();
    Paragraph::new(Line::from(vec![
        Span::styled(" q", key),
        Span::raw(" Quit  "),
        Span::styled("Tab/\u{2190}\u{2192}/h/l", key),
        Span::raw(" Switch Panel  "),
        Span::styled("\u{2191}\u{2193}/j/k", key),
        Span::raw(" Select  "),
        Span::styled("PgUp/PgDn", key),
        Span::raw(" Page  "),
        Span::styled("g/G", key),
        Span::raw(" Top/Bottom  "),
        Span::styled("Ctrl+o", key),
        Span::raw(" Docs "),
    ]))
});

pub struct App {
    collector: Collector,
    stream_scroll: ScrollState,
    messages_scroll: ScrollState,
    active_panel: Panel,
    stream_vh: usize,
    messages_vh: usize,
}

impl App {
    pub fn new() -> Self {
        Self {
            collector: Collector::new(),
            stream_scroll: ScrollState::new(),
            messages_scroll: ScrollState::new(),
            active_panel: Panel::Messages,
            stream_vh: 0,
            messages_vh: 0,
        }
    }

    fn toggle_panel(&mut self) {
        self.active_panel = match self.active_panel {
            Panel::Stream => Panel::Messages,
            Panel::Messages => Panel::Stream,
        };
    }

    fn active_scroll(&mut self) -> &mut ScrollState {
        match self.active_panel {
            Panel::Stream => &mut self.stream_scroll,
            Panel::Messages => &mut self.messages_scroll,
        }
    }

    fn selected_name(&self) -> Option<&'static str> {
        match self.active_panel {
            Panel::Stream => {
                let stream = self.collector.stream();
                stream.get(self.stream_scroll.selected).map(|e| e.name)
            }
            Panel::Messages => {
                let messages = self.collector.messages();
                messages.get(self.messages_scroll.selected).map(|e| e.name)
            }
        }
    }

    fn open_docs(&self) {
        if let Some(name) = self.selected_name() {
            let url = format!("https://mavlink.io/en/messages/common.html#{name}");
            let _ = open::that(url);
        }
    }

    fn active_total(&self) -> usize {
        match self.active_panel {
            Panel::Stream => self.collector.stream().len(),
            Panel::Messages => self.collector.messages().len(),
        }
    }

    fn active_vh(&self) -> usize {
        match self.active_panel {
            Panel::Stream => self.stream_vh,
            Panel::Messages => self.messages_vh,
        }
    }

    /// Handle a key press. Returns `true` if the app should quit.
    pub fn handle_key(&mut self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        let total = self.active_total();
        let vh = self.active_vh();
        match (code, modifiers) {
            (KeyCode::Char('q'), _) | (KeyCode::Esc, _) => return true,
            (KeyCode::Char('o'), m) if m.contains(KeyModifiers::CONTROL) => self.open_docs(),
            (KeyCode::Tab, _)
            | (KeyCode::Left, _)
            | (KeyCode::Right, _)
            | (KeyCode::Char('h'), _)
            | (KeyCode::Char('l'), _) => self.toggle_panel(),
            (KeyCode::Up, _) | (KeyCode::Char('k'), _) => self.active_scroll().select_up(1),
            (KeyCode::Down, _) | (KeyCode::Char('j'), _) => {
                self.active_scroll().select_down(1, total, vh)
            }
            (KeyCode::PageUp, _) => self.active_scroll().select_up(vh),
            (KeyCode::PageDown, _) => self.active_scroll().select_down(vh, total, vh),
            (KeyCode::Char('g'), _) => self.active_scroll().select_top(),
            (KeyCode::Char('G'), _) => self.active_scroll().select_bottom(total, vh),
            _ => {}
        }
        false
    }

    pub fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
        rx: mpsc::Receiver<AppEvent>,
    ) -> io::Result<()> {
        let timeout = std::time::Duration::from_millis(200);
        loop {
            let first = rx.recv_timeout(timeout);
            match first {
                Ok(AppEvent::Mav(msg)) => self.collector.push(*msg),
                Ok(AppEvent::Terminal(crossterm::event::Event::Key(key))) => {
                    if key.kind == KeyEventKind::Press && self.handle_key(key.code, key.modifiers) {
                        return Ok(());
                    }
                }
                Ok(_) | Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => return Ok(()),
            }

            // Drain remaining queued events
            while let Ok(event) = rx.try_recv() {
                match event {
                    AppEvent::Mav(msg) => self.collector.push(*msg),
                    AppEvent::Terminal(crossterm::event::Event::Key(key)) => {
                        if key.kind == KeyEventKind::Press
                            && self.handle_key(key.code, key.modifiers)
                        {
                            return Ok(());
                        }
                    }
                    _ => {}
                }
            }

            terminal.draw(|frame| self.draw(frame))?;
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        let rows = Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(frame.area());

        frame.render_widget(&*HEADER, rows[0]);

        let columns = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(rows[1]);

        let right_rows = Layout::vertical([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(columns[1]);

        self.messages_vh = columns[0].height.saturating_sub(2) as usize;
        self.stream_vh = right_rows[0].height.saturating_sub(2) as usize;

        // Auto-follow before drawing
        let stream_total = self.collector.stream().len();
        self.stream_scroll.auto_follow(stream_total, self.stream_vh);
        let messages_total = self.collector.messages().len();
        self.messages_scroll.auto_follow(messages_total, self.messages_vh);

        let (messages_widget, mut messages_sb) = self.build_messages();
        frame.render_widget(messages_widget, columns[0]);
        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight),
            columns[0],
            &mut messages_sb,
        );

        let (stream_widget, mut stream_sb) = self.build_stream();
        frame.render_widget(stream_widget, right_rows[0]);
        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight),
            right_rows[0],
            &mut stream_sb,
        );

        frame.render_widget(self.build_message(), right_rows[1]);

        frame.render_widget(&*FOOTER, rows[2]);
    }

    fn build_stream(&self) -> (Paragraph<'_>, ScrollbarState) {
        let active = self.active_panel == Panel::Stream;
        let vh = self.stream_vh;
        let stream = self.collector.stream();
        let total = stream.len();

        let selected_style = Style::default().bg(Color::DarkGray);

        let lines: Vec<Line> = stream
            .iter()
            .enumerate()
            .skip(self.stream_scroll.offset)
            .take(vh)
            .map(|(i, entry)| {
                let line = entry.to_line();
                if active && i == self.stream_scroll.selected {
                    line.style(selected_style)
                } else {
                    line
                }
            })
            .collect();

        let block = panel_block(
            "Stream",
            total,
            "types",
            self.stream_scroll.auto_scroll,
            active,
        );

        let paragraph = Paragraph::new(lines).block(block);
        let scrollbar_state =
            ScrollbarState::new(total.saturating_sub(vh)).position(self.stream_scroll.offset);

        (paragraph, scrollbar_state)
    }

    fn build_messages(&self) -> (Paragraph<'_>, ScrollbarState) {
        let active = self.active_panel == Panel::Messages;
        let vh = self.messages_vh;
        let messages = self.collector.messages();
        let total = messages.len();

        let selected_style = Style::default().bg(Color::DarkGray);

        let lines: Vec<Line> = messages
            .iter()
            .enumerate()
            .skip(self.messages_scroll.offset)
            .take(vh)
            .map(|(i, entry)| {
                let line = entry.to_line();
                if active && i == self.messages_scroll.selected {
                    line.style(selected_style)
                } else {
                    line
                }
            })
            .collect();

        let block = panel_block("Messages", total, "", self.messages_scroll.auto_scroll, active);

        let paragraph = Paragraph::new(lines).block(block);
        let scrollbar_state =
            ScrollbarState::new(total.saturating_sub(vh)).position(self.messages_scroll.offset);

        (paragraph, scrollbar_state)
    }

    fn build_message(&self) -> Paragraph<'_> {
        let block = Block::default()
            .title(" Message ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Gray));

        let selected = match self.active_panel {
            Panel::Stream => {
                let s = self.collector.stream();
                s.get(self.stream_scroll.selected.min(s.len().saturating_sub(1)))
                    .map(|e| (e.name, e.sys_id, e.comp_id, e.color, e.parsed_fields()))
            }
            Panel::Messages => {
                let m = self.collector.messages();
                m.get(self.messages_scroll.selected.min(m.len().saturating_sub(1)))
                    .map(|e| (e.name, e.sys_id, e.comp_id, e.color, e.parsed_fields()))
            }
        };

        let lines: Vec<Line> = match selected {
            Some((name, sys_id, comp_id, color, fields)) => {
                message_lines(name, sys_id, comp_id, color, fields)
            }
            None => vec![Line::from(Span::styled(
                "No messages",
                Style::default().fg(Color::DarkGray),
            ))],
        };

        Paragraph::new(lines)
            .block(block)
            .wrap(ratatui::widgets::Wrap { trim: false })
    }
}

fn panel_block(
    label: &str,
    count: usize,
    count_suffix: &str,
    auto_scroll: bool,
    active: bool,
) -> Block<'static> {
    let count_label = if count_suffix.is_empty() {
        format!("{count}")
    } else {
        format!("{count} {count_suffix}")
    };
    let title = format!(
        " {label} [{count_label}] {} ",
        if auto_scroll { "[AUTO]" } else { "" }
    );
    let border_style = if active {
        Style::default().fg(Color::Cyan).bold()
    } else {
        Style::default().fg(Color::Gray)
    };
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style)
}

fn message_lines(
    name: &'static str,
    sys_id: u8,
    comp_id: u8,
    color: Color,
    fields: Vec<(&str, &str)>,
) -> Vec<Line<'static>> {
    let colored = Style::default().fg(color);
    let label = Style::default().fg(Color::Gray);
    let mut lines = vec![
        Line::from(Span::styled(name, Style::default().fg(Color::Cyan).bold())),
        Line::from(""),
        Line::from(vec![
            Span::styled("sys_id  ", label),
            Span::styled(format!("{}", sys_id), colored),
        ]),
        Line::from(vec![
            Span::styled("comp_id ", label),
            Span::styled(format!("{}", comp_id), colored),
        ]),
        Line::from(""),
    ];
    for (key, value) in fields {
        lines.push(Line::from(vec![
            Span::styled(format!("{key}: "), label),
            Span::raw(value.to_string()),
        ]));
    }
    lines
}

#[cfg(test)]
mod tests {
    use mavlink::{MavHeader, common::MavMessage};

    use super::*;
    use crate::message::MavMsg;

    fn make_app_with_stream_entries(n: usize) -> App {
        let mut app = App::new();
        app.active_panel = Panel::Stream;
        app.stream_vh = 10;
        app.messages_vh = 10;
        for i in 0..n {
            let header = MavHeader {
                system_id: i as u8,
                component_id: 1,
                sequence: 0,
            };
            let msg = MavMessage::HEARTBEAT(mavlink::common::HEARTBEAT_DATA::default());
            app.collector.push(MavMsg::new(header, msg));
        }
        app
    }

    // --- handle_key tests ---

    #[test]
    fn quit_on_q() {
        let mut app = App::new();
        assert!(app.handle_key(KeyCode::Char('q'), KeyModifiers::NONE));
    }

    #[test]
    fn quit_on_esc() {
        let mut app = App::new();
        assert!(app.handle_key(KeyCode::Esc, KeyModifiers::NONE));
    }

    #[test]
    fn j_moves_down() {
        let mut app = make_app_with_stream_entries(5);
        app.handle_key(KeyCode::Char('j'), KeyModifiers::NONE);
        assert_eq!(app.stream_scroll.selected, 1);
    }

    #[test]
    fn k_moves_up() {
        let mut app = make_app_with_stream_entries(5);
        app.handle_key(KeyCode::Char('j'), KeyModifiers::NONE);
        app.handle_key(KeyCode::Char('j'), KeyModifiers::NONE);
        app.handle_key(KeyCode::Char('k'), KeyModifiers::NONE);
        assert_eq!(app.stream_scroll.selected, 1);
    }

    #[test]
    fn tab_toggles_panel() {
        let mut app = App::new();
        assert_eq!(app.active_panel, Panel::Messages);
        app.handle_key(KeyCode::Tab, KeyModifiers::NONE);
        assert_eq!(app.active_panel, Panel::Stream);
        app.handle_key(KeyCode::Tab, KeyModifiers::NONE);
        assert_eq!(app.active_panel, Panel::Messages);
    }

    #[test]
    fn g_selects_top() {
        let mut app = make_app_with_stream_entries(5);
        app.handle_key(KeyCode::Char('j'), KeyModifiers::NONE);
        app.handle_key(KeyCode::Char('j'), KeyModifiers::NONE);
        app.handle_key(KeyCode::Char('g'), KeyModifiers::NONE);
        assert_eq!(app.stream_scroll.selected, 0);
    }

    #[test]
    fn big_g_selects_bottom() {
        let mut app = make_app_with_stream_entries(5);
        app.handle_key(KeyCode::Char('G'), KeyModifiers::NONE);
        assert_eq!(app.stream_scroll.selected, 4);
    }
}
