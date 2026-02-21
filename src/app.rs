use std::io;
use std::sync::mpsc;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
};

use crate::collector::Collector;
use crate::message::MavMsg;

#[derive(PartialEq)]
enum Panel {
    Stream,
    Events,
}

struct ScrollState {
    offset: usize,
    auto_scroll: bool,
}

impl ScrollState {
    fn new() -> Self {
        Self {
            offset: 0,
            auto_scroll: true,
        }
    }

    fn scroll_up(&mut self, amount: usize) {
        self.auto_scroll = false;
        self.offset = self.offset.saturating_sub(amount);
    }

    fn scroll_down(&mut self, amount: usize, total: usize, visible: usize) {
        self.offset = self
            .offset
            .saturating_add(amount)
            .min(total.saturating_sub(visible));
        if self.offset >= total.saturating_sub(visible) {
            self.auto_scroll = true;
        }
    }

    fn scroll_to_top(&mut self) {
        self.auto_scroll = false;
        self.offset = 0;
    }

    fn scroll_to_bottom(&mut self, total: usize, visible: usize) {
        self.auto_scroll = true;
        self.offset = total.saturating_sub(visible);
    }

    fn auto_follow(&mut self, total: usize, visible: usize) {
        if self.auto_scroll {
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

    fn active_total(&self) -> usize {
        match self.active_panel {
            Panel::Stream => self.collector.stream().len(),
            Panel::Events => self.collector.events().len(),
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal, rx: mpsc::Receiver<MavMsg>) -> io::Result<()> {
        loop {
            while let Ok(msg) = rx.try_recv() {
                self.push(msg);
            }

            terminal.draw(|frame| draw(frame, self))?;

            if event::poll(std::time::Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        let h = terminal.get_frame().area().height.saturating_sub(2) as usize;
                        let total = self.active_total();
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                            KeyCode::Tab | KeyCode::Left | KeyCode::Right | KeyCode::Char('h') | KeyCode::Char('l') => {
                                self.toggle_panel()
                            }
                            KeyCode::Up | KeyCode::Char('k') => self.active_scroll().scroll_up(1),
                            KeyCode::Down | KeyCode::Char('j') => {
                                self.active_scroll().scroll_down(1, total, h)
                            }
                            KeyCode::PageUp => self.active_scroll().scroll_up(h),
                            KeyCode::PageDown => self.active_scroll().scroll_down(h, total, h),
                            KeyCode::Char('g') => self.active_scroll().scroll_to_top(),
                            KeyCode::Char('G') => self.active_scroll().scroll_to_bottom(total, h),
                            _ => {}
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
        Line::from(Span::styled(r" _____ _____ _ _ ___ ___ ___ ___ ___ ", header_style)),
        Line::from(Span::styled(r"|     |  _  | | |_ -|   | .'|  _| '_|", header_style)),
        Line::from(Span::styled(r"|_|_|_|__|__|\_/|___|_|_|__,|_| |_,_|", header_style)),
    ]);
    frame.render_widget(header, rows[0]);

    let chunks = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[1]);

    draw_stream(
        frame,
        &app.collector,
        &mut app.stream_scroll,
        chunks[0],
        app.active_panel == Panel::Stream,
    );
    draw_events(
        frame,
        &app.collector,
        &mut app.events_scroll,
        chunks[1],
        app.active_panel == Panel::Events,
    );

    let footer = Line::from(vec![
        Span::styled(" q", Style::default().fg(Color::Cyan).bold()),
        Span::raw(" Quit  "),
        Span::styled("Tab/\u{2190}\u{2192}/h/l", Style::default().fg(Color::Cyan).bold()),
        Span::raw(" Switch Panel  "),
        Span::styled("\u{2191}\u{2193}/j/k", Style::default().fg(Color::Cyan).bold()),
        Span::raw(" Scroll  "),
        Span::styled("PgUp/PgDn", Style::default().fg(Color::Cyan).bold()),
        Span::raw(" Page  "),
        Span::styled("g/G", Style::default().fg(Color::Cyan).bold()),
        Span::raw(" Top/Bottom "),
    ]);
    frame.render_widget(Paragraph::new(footer), rows[2]);
}

fn draw_stream(
    frame: &mut Frame,
    collector: &Collector,
    scroll: &mut ScrollState,
    area: Rect,
    active: bool,
) {
    let vh = area.height.saturating_sub(2) as usize;
    let stream = collector.stream();
    let total = stream.len();

    scroll.auto_follow(total, vh);

    let lines: Vec<Line> = stream
        .iter()
        .skip(scroll.offset)
        .take(vh)
        .map(|entry| entry.to_line())
        .collect();

    let title = format!(
        " Stream [{} types] {} ",
        total,
        if scroll.auto_scroll { "[AUTO]" } else { "" }
    );

    let border_style = if active {
        Style::default().fg(Color::Cyan).bold()
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style);

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);

    let mut scrollbar_state =
        ScrollbarState::new(total.saturating_sub(vh)).position(scroll.offset);
    frame.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::VerticalRight),
        area,
        &mut scrollbar_state,
    );
}

fn draw_events(
    frame: &mut Frame,
    collector: &Collector,
    scroll: &mut ScrollState,
    area: Rect,
    active: bool,
) {
    let vh = area.height.saturating_sub(2) as usize;
    let events = collector.events();
    let total = events.len();

    scroll.auto_follow(total, vh);

    let lines: Vec<Line> = events
        .iter()
        .skip(scroll.offset)
        .take(vh)
        .map(|entry| entry.to_line())
        .collect();

    let title = format!(
        " Events [{}/{}] {} ",
        scroll.offset + 1,
        total,
        if scroll.auto_scroll { "[AUTO]" } else { "" }
    );

    let border_style = if active {
        Style::default().fg(Color::Cyan).bold()
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style);

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);

    let mut scrollbar_state =
        ScrollbarState::new(total.saturating_sub(vh)).position(scroll.offset);
    frame.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::VerticalRight),
        area,
        &mut scrollbar_state,
    );
}
