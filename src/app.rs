use std::{io, sync::mpsc};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
};

use crate::{collector::Collector, message::MavMsg};

#[derive(Debug, PartialEq)]
enum Panel {
    Stream,
    Events,
}

struct ScrollState {
    offset: usize,
    selected: usize,
    auto_scroll: bool,
}

impl ScrollState {
    fn new() -> Self {
        Self {
            offset: 0,
            selected: 0,
            auto_scroll: true,
        }
    }

    fn select_up(&mut self, amount: usize) {
        self.auto_scroll = false;
        self.selected = self.selected.saturating_sub(amount);
        if self.selected < self.offset {
            self.offset = self.selected;
        }
    }

    fn select_down(&mut self, amount: usize, total: usize, visible: usize) {
        if total == 0 {
            return;
        }
        self.selected = self.selected.saturating_add(amount).min(total - 1);
        if self.selected >= self.offset + visible {
            self.offset = self.selected.saturating_sub(visible - 1);
        }
        self.auto_scroll = self.selected >= total.saturating_sub(1);
    }

    fn select_top(&mut self) {
        self.auto_scroll = false;
        self.selected = 0;
        self.offset = 0;
    }

    fn select_bottom(&mut self, total: usize, visible: usize) {
        if total == 0 {
            return;
        }
        self.auto_scroll = true;
        self.selected = total - 1;
        self.offset = total.saturating_sub(visible);
    }

    fn auto_follow(&mut self, total: usize, visible: usize) {
        if self.auto_scroll && total > 0 {
            self.selected = total - 1;
            self.offset = total.saturating_sub(visible);
        }
    }
}

pub struct App {
    collector: Collector,
    stream_scroll: ScrollState,
    events_scroll: ScrollState,
    active_panel: Panel,
}

impl App {
    pub fn new() -> Self {
        Self {
            collector: Collector::new(),
            stream_scroll: ScrollState::new(),
            events_scroll: ScrollState::new(),
            active_panel: Panel::Stream,
        }
    }

    pub fn push(&mut self, msg: MavMsg) {
        self.collector.push(msg);
    }

    fn toggle_panel(&mut self) {
        self.active_panel = match self.active_panel {
            Panel::Stream => Panel::Events,
            Panel::Events => Panel::Stream,
        };
    }

    fn active_scroll(&mut self) -> &mut ScrollState {
        match self.active_panel {
            Panel::Stream => &mut self.stream_scroll,
            Panel::Events => &mut self.events_scroll,
        }
    }

    fn selected_name(&self) -> Option<&'static str> {
        match self.active_panel {
            Panel::Stream => {
                let stream = self.collector.stream();
                stream.get(self.stream_scroll.selected).map(|e| e.name)
            }
            Panel::Events => {
                let events = self.collector.events();
                events.get(self.events_scroll.selected).map(|e| e.name)
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
            Panel::Events => self.collector.events().len(),
        }
    }

    /// Handle a key press. Returns `true` if the app should quit.
    pub fn handle_key(
        &mut self,
        code: KeyCode,
        modifiers: KeyModifiers,
        visible_height: usize,
    ) -> bool {
        if code == KeyCode::Char('o') && modifiers.contains(KeyModifiers::CONTROL) {
            self.open_docs();
        }
        let total = self.active_total();
        match code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Tab
            | KeyCode::Left
            | KeyCode::Right
            | KeyCode::Char('h')
            | KeyCode::Char('l') => self.toggle_panel(),
            KeyCode::Up | KeyCode::Char('k') => self.active_scroll().select_up(1),
            KeyCode::Down | KeyCode::Char('j') => {
                self.active_scroll().select_down(1, total, visible_height)
            }
            KeyCode::PageUp => self.active_scroll().select_up(visible_height),
            KeyCode::PageDown => {
                self.active_scroll()
                    .select_down(visible_height, total, visible_height)
            }
            KeyCode::Char('g') => self.active_scroll().select_top(),
            KeyCode::Char('G') => self.active_scroll().select_bottom(total, visible_height),
            _ => {}
        }
        false
    }

    pub fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
        rx: mpsc::Receiver<MavMsg>,
    ) -> io::Result<()> {
        loop {
            while let Ok(msg) = rx.try_recv() {
                self.push(msg);
            }

            terminal.draw(|frame| draw(frame, self))?;

            if event::poll(std::time::Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        let frame_h = terminal.get_frame().area().height.saturating_sub(4);
                        let h = match self.active_panel {
                            Panel::Events => frame_h.saturating_sub(2) as usize,
                            Panel::Stream => {
                                ((frame_h as u32 * 60 / 100) as u16).saturating_sub(2) as usize
                            }
                        };
                        if self.handle_key(key.code, key.modifiers, h) {
                            return Ok(());
                        }
                    }
                }
            }
        }
    }
}

fn draw(frame: &mut Frame, app: &mut App) {
    let rows = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .split(frame.area());

    let header_style = Style::default().fg(Color::Cyan).bold();
    let header = Paragraph::new(vec![
        Line::from(Span::styled(
            r" _____ _____ _ _ ___ ___ ___ ___ ___ ",
            header_style,
        )),
        Line::from(Span::styled(
            r"|     |  _  | | |_ -|   | .'|  _| '_|",
            header_style,
        )),
        Line::from(Span::styled(
            r"|_|_|_|__|__|\_/|___|_|_|__,|_| |_,_|",
            header_style,
        )),
    ]);
    frame.render_widget(header, rows[0]);

    let columns =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).split(rows[1]);

    let right_rows = Layout::vertical([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(columns[1]);

    let events_vh = columns[0].height.saturating_sub(2) as usize;
    let stream_vh = right_rows[0].height.saturating_sub(2) as usize;

    // Auto-follow before drawing
    let stream_total = app.collector.stream().len();
    app.stream_scroll.auto_follow(stream_total, stream_vh);
    let events_total = app.collector.events().len();
    app.events_scroll.auto_follow(events_total, events_vh);

    draw_events(
        frame,
        &app.collector,
        &app.events_scroll,
        columns[0],
        app.active_panel == Panel::Events,
    );
    draw_stream(
        frame,
        &app.collector,
        &app.stream_scroll,
        right_rows[0],
        app.active_panel == Panel::Stream,
    );
    draw_message(
        frame,
        &app.collector,
        &app.active_panel,
        &app.stream_scroll,
        &app.events_scroll,
        right_rows[1],
    );

    let footer = Line::from(vec![
        Span::styled(" q", Style::default().fg(Color::Cyan).bold()),
        Span::raw(" Quit  "),
        Span::styled(
            "Tab/\u{2190}\u{2192}/h/l",
            Style::default().fg(Color::Cyan).bold(),
        ),
        Span::raw(" Switch Panel  "),
        Span::styled(
            "\u{2191}\u{2193}/j/k",
            Style::default().fg(Color::Cyan).bold(),
        ),
        Span::raw(" Select  "),
        Span::styled("PgUp/PgDn", Style::default().fg(Color::Cyan).bold()),
        Span::raw(" Page  "),
        Span::styled("g/G", Style::default().fg(Color::Cyan).bold()),
        Span::raw(" Top/Bottom  "),
        Span::styled("Ctrl+o", Style::default().fg(Color::Cyan).bold()),
        Span::raw(" Docs "),
    ]);
    frame.render_widget(Paragraph::new(footer), rows[2]);
}

fn draw_stream(
    frame: &mut Frame,
    collector: &Collector,
    scroll: &ScrollState,
    area: Rect,
    active: bool,
) {
    let vh = area.height.saturating_sub(2) as usize;
    let stream = collector.stream();
    let total = stream.len();

    let selected_style = Style::default().bg(Color::DarkGray);

    let lines: Vec<Line> = stream
        .iter()
        .enumerate()
        .skip(scroll.offset)
        .take(vh)
        .map(|(i, entry)| {
            let line = entry.to_line();
            if active && i == scroll.selected {
                line.style(selected_style)
            } else {
                line
            }
        })
        .collect();

    let title = format!(
        " Stream [{} types] {} ",
        total,
        if scroll.auto_scroll { "[AUTO]" } else { "" }
    );

    let border_style = if active {
        Style::default().fg(Color::Cyan).bold()
    } else {
        Style::default().fg(Color::Gray)
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style);

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);

    let mut scrollbar_state = ScrollbarState::new(total.saturating_sub(vh)).position(scroll.offset);
    frame.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::VerticalRight),
        area,
        &mut scrollbar_state,
    );
}

fn draw_events(
    frame: &mut Frame,
    collector: &Collector,
    scroll: &ScrollState,
    area: Rect,
    active: bool,
) {
    let vh = area.height.saturating_sub(2) as usize;
    let events = collector.events();
    let total = events.len();

    let selected_style = Style::default().bg(Color::DarkGray);

    let lines: Vec<Line> = events
        .iter()
        .enumerate()
        .skip(scroll.offset)
        .take(vh)
        .map(|(i, entry)| {
            let line = entry.to_line();
            if active && i == scroll.selected {
                line.style(selected_style)
            } else {
                line
            }
        })
        .collect();

    let title = format!(
        " Events [{}] {} ",
        total,
        if scroll.auto_scroll { "[AUTO]" } else { "" }
    );

    let border_style = if active {
        Style::default().fg(Color::Cyan).bold()
    } else {
        Style::default().fg(Color::Gray)
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style);

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);

    let mut scrollbar_state = ScrollbarState::new(total.saturating_sub(vh)).position(scroll.offset);
    frame.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::VerticalRight),
        area,
        &mut scrollbar_state,
    );
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

fn draw_message(
    frame: &mut Frame,
    collector: &Collector,
    active_panel: &Panel,
    stream_scroll: &ScrollState,
    events_scroll: &ScrollState,
    area: Rect,
) {
    let block = Block::default()
        .title(" Message ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Gray));

    let lines: Vec<Line> = match active_panel {
        Panel::Stream => {
            let stream = collector.stream();
            if stream.is_empty() {
                vec![Line::from(Span::styled(
                    "No messages",
                    Style::default().fg(Color::DarkGray),
                ))]
            } else {
                let entry = &stream[stream_scroll.selected.min(stream.len() - 1)];
                message_lines(
                    entry.name,
                    entry.sys_id,
                    entry.comp_id,
                    entry.color,
                    entry.parsed_fields(),
                )
            }
        }
        Panel::Events => {
            let events = collector.events();
            if events.is_empty() {
                vec![Line::from(Span::styled(
                    "No events",
                    Style::default().fg(Color::DarkGray),
                ))]
            } else {
                let entry = &events[events_scroll.selected.min(events.len() - 1)];
                message_lines(
                    entry.name,
                    entry.sys_id,
                    entry.comp_id,
                    entry.color,
                    entry.parsed_fields(),
                )
            }
        }
    };

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(ratatui::widgets::Wrap { trim: false });
    frame.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use mavlink::{MavHeader, common::MavMessage};

    use super::*;
    use crate::message::MavMsg;

    fn make_app_with_stream_entries(n: usize) -> App {
        let mut app = App::new();
        for i in 0..n {
            let header = MavHeader {
                system_id: i as u8,
                component_id: 1,
                sequence: 0,
            };
            let msg = MavMessage::HEARTBEAT(mavlink::common::HEARTBEAT_DATA::default());
            app.push(MavMsg::new(header, msg));
        }
        app
    }

    // --- ScrollState tests ---

    #[test]
    fn scroll_up_from_zero() {
        let mut s = ScrollState::new();
        s.select_up(1);
        assert_eq!(s.selected, 0);
        assert_eq!(s.offset, 0);
    }

    #[test]
    fn scroll_down_clamps_to_total() {
        let mut s = ScrollState::new();
        s.select_down(100, 5, 10);
        assert_eq!(s.selected, 4);
    }

    #[test]
    fn scroll_down_adjusts_offset() {
        let mut s = ScrollState::new();
        // visible=3, total=10, go down 5 times
        for _ in 0..5 {
            s.select_down(1, 10, 3);
        }
        assert_eq!(s.selected, 5);
        // offset should have adjusted so selected is visible
        assert!(s.offset + 3 > s.selected);
    }

    #[test]
    fn scroll_up_adjusts_offset() {
        let mut s = ScrollState::new();
        // Go to bottom first
        s.select_bottom(10, 3);
        // Now scroll up
        s.select_up(5);
        assert_eq!(s.selected, 4);
        assert!(s.offset <= s.selected);
    }

    #[test]
    fn select_top_resets() {
        let mut s = ScrollState::new();
        s.select_down(5, 10, 3);
        s.select_top();
        assert_eq!(s.selected, 0);
        assert_eq!(s.offset, 0);
    }

    #[test]
    fn select_bottom_jumps_to_end() {
        let mut s = ScrollState::new();
        s.select_bottom(10, 3);
        assert_eq!(s.selected, 9);
        assert_eq!(s.offset, 7);
    }

    #[test]
    fn auto_follow_when_enabled() {
        let mut s = ScrollState::new();
        s.auto_scroll = true;
        s.auto_follow(10, 5);
        assert_eq!(s.selected, 9);
        assert_eq!(s.offset, 5);
    }

    #[test]
    fn auto_follow_noop_when_disabled() {
        let mut s = ScrollState::new();

        s.auto_scroll = false;
        s.auto_follow(10, 5);
        assert_eq!(s.selected, 0);
        assert_eq!(s.offset, 0);
    }

    // --- handle_key tests ---

    #[test]
    fn quit_on_q() {
        let mut app = App::new();
        assert!(app.handle_key(KeyCode::Char('q'), KeyModifiers::NONE, 10));
    }

    #[test]
    fn quit_on_esc() {
        let mut app = App::new();
        assert!(app.handle_key(KeyCode::Esc, KeyModifiers::NONE, 10));
    }

    #[test]
    fn j_moves_down() {
        let mut app = make_app_with_stream_entries(5);
        app.handle_key(KeyCode::Char('j'), KeyModifiers::NONE, 10);
        assert_eq!(app.stream_scroll.selected, 1);
    }

    #[test]
    fn k_moves_up() {
        let mut app = make_app_with_stream_entries(5);
        app.handle_key(KeyCode::Char('j'), KeyModifiers::NONE, 10);
        app.handle_key(KeyCode::Char('j'), KeyModifiers::NONE, 10);
        app.handle_key(KeyCode::Char('k'), KeyModifiers::NONE, 10);
        assert_eq!(app.stream_scroll.selected, 1);
    }

    #[test]
    fn tab_toggles_panel() {
        let mut app = App::new();
        assert_eq!(app.active_panel, Panel::Stream);
        app.handle_key(KeyCode::Tab, KeyModifiers::NONE, 10);
        assert_eq!(app.active_panel, Panel::Events);
        app.handle_key(KeyCode::Tab, KeyModifiers::NONE, 10);
        assert_eq!(app.active_panel, Panel::Stream);
    }

    #[test]
    fn g_selects_top() {
        let mut app = make_app_with_stream_entries(5);
        app.handle_key(KeyCode::Char('j'), KeyModifiers::NONE, 10);
        app.handle_key(KeyCode::Char('j'), KeyModifiers::NONE, 10);
        app.handle_key(KeyCode::Char('g'), KeyModifiers::NONE, 10);
        assert_eq!(app.stream_scroll.selected, 0);
    }

    #[test]
    fn big_g_selects_bottom() {
        let mut app = make_app_with_stream_entries(5);
        app.handle_key(KeyCode::Char('G'), KeyModifiers::NONE, 10);
        assert_eq!(app.stream_scroll.selected, 4);
    }
}
