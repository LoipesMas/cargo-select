use crate::select::{score_targets, Target};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use fuzzy_matcher::skim::SkimMatcherV2;
use std::{error::Error, io};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};

pub struct Tui;

impl Tui {
    pub fn launch(targets: &[Target]) -> Result<&Target, Box<dyn Error>> {
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

    fn main_loop<'a, B: Backend>(
        terminal: &mut Terminal<B>,
        targets: &'a [Target],
    ) -> Result<&'a Target, Box<dyn Error>> {
        let mut pattern = String::new();
        let mut list_state = ListState::default();
        let mut selected_idx = 0;
        let skim = SkimMatcherV2::default();
        loop {
            let terminal_height: usize = terminal.size().unwrap().height.into();

            let targets = if !pattern.is_empty() {
                score_targets(targets, &pattern, &skim)
            } else {
                targets.iter().collect()
            };

            let targets = targets
                .windows(terminal_height.saturating_sub(1))
                .last()
                .unwrap_or(&targets)
                .to_vec();

            selected_idx = selected_idx.min(targets.len()).max(1);
            let transformed_idx = targets.len().saturating_sub(selected_idx);
            list_state.select(Some(transformed_idx));
            terminal.draw(|f| Tui::ui(f, &targets, &pattern, &mut list_state))?;

            if let Event::Key(key) = crossterm::event::read()? {
                if (matches!(key.code, KeyCode::Char('c'))
                    && key.modifiers.contains(KeyModifiers::CONTROL))
                    || matches!(key.code, KeyCode::Esc)
                {
                    return Err("User interrupt.".into());
                }
                if matches!(key.code, KeyCode::Char('w'))
                    && key.modifiers.contains(KeyModifiers::CONTROL)
                {
                    // delete last word
                    let mut p = pattern.pop();
                    while !matches!(p, Some(' ') | None) {
                        p = pattern.pop();
                    }
                    continue;
                }
                match key.code {
                    KeyCode::Char(c) => pattern.push(c),
                    KeyCode::Backspace => {
                        pattern.pop();
                    }
                    KeyCode::Up => selected_idx += 1,
                    KeyCode::Down => selected_idx = selected_idx.saturating_sub(1),
                    KeyCode::Enter => {
                        if targets.is_empty() {
                            return Err("No targets matched!".into());
                        } else {
                            return Ok(targets[transformed_idx.min(targets.len() - 1)]);
                        }
                    }
                    _ => {}
                };
            }
        }
    }

    fn ui<B: Backend>(
        frame: &mut Frame<B>,
        targets: &[&Target],
        pattern: &str,
        list_state: &mut ListState,
    ) {
        let items = targets
            .windows(frame.size().height.saturating_sub(1).into())
            .last()
            .unwrap_or(targets)
            .iter()
            .map(|t| ListItem::new(t.to_string()))
            .collect::<Vec<_>>();

        let padding = (frame.size().height - 1).saturating_sub(items.len() as u16);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(padding),
                    Constraint::Min(0),
                    Constraint::Length(1),
                ]
                .as_ref(),
            )
            .split(frame.size());

        let items = List::new(items).highlight_style(Style::default().bg(Color::DarkGray));
        frame.render_stateful_widget(items, chunks[1], list_state);
        let input = Paragraph::new(pattern)
            .block(Block::default().style(Style::default().bg(Color::DarkGray)));
        frame.set_cursor(chunks[2].x + pattern.len() as u16, chunks[2].y);
        frame.render_widget(input, chunks[2]);
    }
}
