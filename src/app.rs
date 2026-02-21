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
    Telemetry,
    Commands,
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
    telemetry_scroll: ScrollState,
    commands_scroll: ScrollState,
    active_panel: Panel,
}

impl App {
    pub fn new() -> Self {
        Self {
            collector: Collector::new(),
            telemetry_scroll: ScrollState::new(),
            commands_scroll: ScrollState::new(),
            active_panel: Panel::Telemetry,
        }
    }

    pub fn push(&mut self, msg: MavMsg) {
        self.collector.push(msg);
    }

    fn toggle_panel(&mut self) {
        self.active_panel = match self.active_panel {
            Panel::Telemetry => Panel::Commands,
            Panel::Commands => Panel::Telemetry,
        };
    }

    fn active_scroll(&mut self) -> &mut ScrollState {
        match self.active_panel {
            Panel::Telemetry => &mut self.telemetry_scroll,
            Panel::Commands => &mut self.commands_scroll,
        }
    }

    fn active_total(&self) -> usize {
        match self.active_panel {
            Panel::Telemetry => self.collector.telemetry().len(),
            Panel::Commands => self.collector.commands().len(),
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
    let rows = Layout::vertical([Constraint::Min(0), Constraint::Length(1)])
        .split(frame.area());

    let chunks = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[0]);

    draw_telemetry(
        frame,
        &app.collector,
        &mut app.telemetry_scroll,
        chunks[0],
        app.active_panel == Panel::Telemetry,
    );
    draw_commands(
        frame,
        &app.collector,
        &mut app.commands_scroll,
        chunks[1],
        app.active_panel == Panel::Commands,
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
    frame.render_widget(Paragraph::new(footer), rows[1]);
}

fn draw_telemetry(
    frame: &mut Frame,
    collector: &Collector,
    scroll: &mut ScrollState,
    area: Rect,
    active: bool,
) {
    let vh = area.height.saturating_sub(2) as usize;
    let telemetry = collector.telemetry();
    let total = telemetry.len();

    scroll.auto_follow(total, vh);

    let lines: Vec<Line> = telemetry
        .iter()
        .skip(scroll.offset)
        .take(vh)
        .map(|entry| {
            Line::from(Span::styled(
                entry.text.as_str(),
                Style::default().fg(entry.color),
            ))
        })
        .collect();

    let title = format!(
        " Telemetry [{} types] {} ",
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

fn draw_commands(
    frame: &mut Frame,
    collector: &Collector,
    scroll: &mut ScrollState,
    area: Rect,
    active: bool,
) {
    let vh = area.height.saturating_sub(2) as usize;
    let commands = collector.commands();
    let total = commands.len();

    scroll.auto_follow(total, vh);

    let lines: Vec<Line> = commands
        .iter()
        .skip(scroll.offset)
        .take(vh)
        .map(|entry| {
            Line::from(Span::styled(
                entry.text.as_str(),
                Style::default().fg(entry.color),
            ))
        })
        .collect();

    let title = format!(
        " Commands [{}/{}] {} ",
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
