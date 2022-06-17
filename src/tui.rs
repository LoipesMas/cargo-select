use crate::select::{score_targets, Target};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use fuzzy_matcher::skim::SkimMatcherV2;
use std::{
    error::Error,
    io::{self, Stdout},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};

pub struct Tui;

impl Tui {
    pub fn launch(targets: &[Target]) -> Result<String, Box<dyn Error>> {
        // setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let res = Tui::main_loop(&mut terminal, targets);

        // restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;
        res
    }

    fn main_loop<B: Backend>(
        terminal: &mut Terminal<B>,
        targets: &[Target],
    ) -> Result<String, Box<dyn Error>> {
        let mut pattern = String::new();
        loop {
            terminal.draw(|f| Tui::ui(f, targets, &pattern))?;

            match crossterm::event::read()? {
                Event::Key(key) => {
                    if matches!(key.code, KeyCode::Char('c'))
                        && key.modifiers.contains(KeyModifiers::CONTROL)
                    {
                        std::process::exit(1);
                    }
                    match key.code {
                        KeyCode::Char(c) => pattern.push(c),
                        KeyCode::Backspace => {
                            pattern.pop();
                        }
                        KeyCode::Enter => return Ok(pattern),
                        _ => {}
                    };
                }
                Event::Mouse(_) => todo!(),
                Event::Resize(_, _) => todo!(),
            }
        }
    }

    fn ui<B: Backend>(frame: &mut Frame<B>, targets: &[Target], pattern: &str) {
        let skim = SkimMatcherV2::default();

        let items = if !pattern.is_empty() {
            (score_targets(targets, pattern, &skim)
                .iter()
                .map(|t| ListItem::new(t.to_string())))
            .collect::<Vec<_>>()
        } else {
            targets
                .iter()
                .map(|t| ListItem::new(t.to_string()))
                .collect::<Vec<_>>()
        };

        let padding = (frame.size().height - 1).saturating_sub(items.len() as u16);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(padding),
                    Constraint::Min(1),
                    Constraint::Length(1),
                ]
                .as_ref(),
            )
            .split(frame.size());

        let items = List::new(items);
        frame.render_widget(items, chunks[1]);
        let input =
            Paragraph::new(pattern).block(Block::default().style(Style::default().bg(Color::DarkGray)));
        frame.set_cursor(chunks[2].x + pattern.len() as u16, chunks[2].y);
        frame.render_widget(input, chunks[2]);
    }
}
