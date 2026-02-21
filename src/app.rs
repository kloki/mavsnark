use std::{collections::HashMap, io, sync::mpsc};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
};

use crate::message::MavMsg;

struct TelemetryEntry {
    color: Color,
    text: String,
    count: usize,
}

struct TelemetryPanel {
    entries: HashMap<String, TelemetryEntry>,
    scroll: usize,
    auto_scroll: bool,
}

impl TelemetryPanel {
    fn new() -> Self {
        Self {
            entries: HashMap::new(),
            scroll: 0,
            auto_scroll: true,
        }
    }

    fn push(&mut self, msg_type: String, color: Color, text: String) {
        let entry = self.entries.entry(msg_type).or_insert(TelemetryEntry {
            color,
            text: String::new(),
            count: 0,
        });
        entry.color = color;
        entry.text = text;
        entry.count += 1;
    }

    fn sorted_entries(&self) -> Vec<(&str, &TelemetryEntry)> {
        let mut entries: Vec<_> = self.entries.iter().map(|(k, v)| (k.as_str(), v)).collect();
        entries.sort_by(|a, b| b.1.count.cmp(&a.1.count));
        entries
    }

    fn len(&self) -> usize {
        self.entries.len()
    }

    fn scroll_up(&mut self, amount: usize) {
        self.auto_scroll = false;
        self.scroll = self.scroll.saturating_sub(amount);
    }

    fn scroll_down(&mut self, amount: usize, visible_height: usize) {
        self.scroll = self
            .scroll
            .saturating_add(amount)
            .min(self.len().saturating_sub(visible_height));
        if self.scroll >= self.len().saturating_sub(visible_height) {
            self.auto_scroll = true;
        }
    }

    fn scroll_to_top(&mut self) {
        self.auto_scroll = false;
        self.scroll = 0;
    }

    fn scroll_to_bottom(&mut self, visible_height: usize) {
        self.auto_scroll = true;
        self.scroll = self.len().saturating_sub(visible_height);
    }
}

struct CommandPanel {
    messages: Vec<(Color, String)>,
    scroll: usize,
    auto_scroll: bool,
}

impl CommandPanel {
    fn new() -> Self {
        Self {
            messages: Vec::new(),
            scroll: 0,
            auto_scroll: true,
        }
    }

    fn push(&mut self, color: Color, text: String) {
        self.messages.push((color, text));
        if self.auto_scroll {
            self.scroll = self.messages.len().saturating_sub(1);
        }
    }

    fn scroll_up(&mut self, amount: usize) {
        self.auto_scroll = false;
        self.scroll = self.scroll.saturating_sub(amount);
    }

    fn scroll_down(&mut self, amount: usize, visible_height: usize) {
        self.scroll = self
            .scroll
            .saturating_add(amount)
            .min(self.messages.len().saturating_sub(visible_height));
        if self.scroll >= self.messages.len().saturating_sub(visible_height) {
            self.auto_scroll = true;
        }
    }

    fn scroll_to_top(&mut self) {
        self.auto_scroll = false;
        self.scroll = 0;
    }

    fn scroll_to_bottom(&mut self, visible_height: usize) {
        self.auto_scroll = true;
        self.scroll = self.messages.len().saturating_sub(visible_height);
    }
}

#[derive(PartialEq)]
enum Panel {
    Telemetry,
    Commands,
}

pub struct App {
    telemetry: TelemetryPanel,
    commands: CommandPanel,
    active_panel: Panel,
}

impl App {
    pub fn new() -> Self {
        Self {
            telemetry: TelemetryPanel::new(),
            commands: CommandPanel::new(),
            active_panel: Panel::Telemetry,
        }
    }

    pub fn push(&mut self, msg: MavMsg) {
        let color = msg.color();
        let text = msg.text();
        if msg.is_command() {
            self.commands.push(color, text);
        } else {
            self.telemetry
                .push(msg.msg_type().to_string(), color, text);
        }
    }

    fn toggle_panel(&mut self) {
        self.active_panel = match self.active_panel {
            Panel::Telemetry => Panel::Commands,
            Panel::Commands => Panel::Telemetry,
        };
    }
}

pub fn run(terminal: &mut DefaultTerminal, rx: mpsc::Receiver<MavMsg>) -> io::Result<()> {
    let mut app = App::new();

    loop {
        while let Ok(msg) = rx.try_recv() {
            app.push(msg);
        }

        terminal.draw(|frame| draw(frame, &app))?;

        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    let h = terminal.get_frame().area().height.saturating_sub(2) as usize;
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                        KeyCode::Tab | KeyCode::Char('h') | KeyCode::Char('l') => {
                            app.toggle_panel()
                        }
                        KeyCode::Up | KeyCode::Char('k') => match app.active_panel {
                            Panel::Telemetry => app.telemetry.scroll_up(1),
                            Panel::Commands => app.commands.scroll_up(1),
                        },
                        KeyCode::Down | KeyCode::Char('j') => match app.active_panel {
                            Panel::Telemetry => app.telemetry.scroll_down(1, h),
                            Panel::Commands => app.commands.scroll_down(1, h),
                        },
                        KeyCode::PageUp => match app.active_panel {
                            Panel::Telemetry => app.telemetry.scroll_up(h),
                            Panel::Commands => app.commands.scroll_up(h),
                        },
                        KeyCode::PageDown => match app.active_panel {
                            Panel::Telemetry => app.telemetry.scroll_down(h, h),
                            Panel::Commands => app.commands.scroll_down(h, h),
                        },
                        KeyCode::Char('g') => match app.active_panel {
                            Panel::Telemetry => app.telemetry.scroll_to_top(),
                            Panel::Commands => app.commands.scroll_to_top(),
                        },
                        KeyCode::Char('G') => match app.active_panel {
                            Panel::Telemetry => app.telemetry.scroll_to_bottom(h),
                            Panel::Commands => app.commands.scroll_to_bottom(h),
                        },
                        _ => {}
                    }
                }
            }
        }
    }
}

fn draw(frame: &mut Frame, app: &App) {
    let chunks = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(frame.area());

    draw_telemetry(
        frame,
        &app.telemetry,
        chunks[0],
        app.active_panel == Panel::Telemetry,
    );
    draw_commands(
        frame,
        &app.commands,
        chunks[1],
        app.active_panel == Panel::Commands,
    );
}

fn draw_telemetry(frame: &mut Frame, panel: &TelemetryPanel, area: Rect, active: bool) {
    let vh = area.height.saturating_sub(2) as usize;
    let sorted = panel.sorted_entries();

    let lines: Vec<Line> = sorted
        .iter()
        .skip(panel.scroll)
        .take(vh)
        .map(|(_, entry)| {
            Line::from(Span::styled(
                entry.text.as_str(),
                Style::default().fg(entry.color),
            ))
        })
        .collect();

    let title = format!(
        " Telemetry [{} types] {} ",
        panel.len(),
        if panel.auto_scroll { "[AUTO]" } else { "" }
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
        ScrollbarState::new(panel.len().saturating_sub(vh)).position(panel.scroll);
    frame.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::VerticalRight),
        area,
        &mut scrollbar_state,
    );
}

fn draw_commands(frame: &mut Frame, panel: &CommandPanel, area: Rect, active: bool) {
    let vh = area.height.saturating_sub(2) as usize;

    let lines: Vec<Line> = panel
        .messages
        .iter()
        .skip(panel.scroll)
        .take(vh)
        .map(|(color, text)| Line::from(Span::styled(text.as_str(), Style::default().fg(*color))))
        .collect();

    let title = format!(
        " Commands [{}/{}] {} ",
        panel.scroll + 1,
        panel.messages.len(),
        if panel.auto_scroll { "[AUTO]" } else { "" }
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
        ScrollbarState::new(panel.messages.len().saturating_sub(vh)).position(panel.scroll);
    frame.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::VerticalRight),
        area,
        &mut scrollbar_state,
    );
}
