//! Home component for Rustoria.

use crate::app::SelectedApp;
use crate::components::Component;
use crate::tui::Frame;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph, Wrap},
};


/// Represents the home screen UI component.
pub struct Home {
    /// The state of the application list.
    app_list_state: ListState,
    /// The available applications.
    apps: Vec<SelectedApp>,
}

impl Home {
    /// Creates a new `Home` component.
    pub fn new() -> Self {
        let apps = vec![SelectedApp::Hospital, SelectedApp::Pharmacy];
        let mut app_list_state = ListState::default();
        app_list_state.select(Some(0)); // Select the first item by default
        Self { app_list_state, apps }
    }

    /// Handle input
    pub fn handle_input(&mut self, key: KeyEvent) -> Result<Option<SelectedApp>> {
        match key.code {
            KeyCode::Up => {
                let i = match self.app_list_state.selected() {
                    Some(i) => {
                        if i == 0 {
                            self.apps.len() - 1
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                self.app_list_state.select(Some(i));
            }
            KeyCode::Down => {
                let i = match self.app_list_state.selected() {
                    Some(i) => {
                        if i >= self.apps.len() - 1 {
                            0
                        } else {
                            i + 1
                        }
                    }
                    None => 0,
                };
                self.app_list_state.select(Some(i));
            }
            KeyCode::Enter => {
                // Return the selected application
                if let Some(i) = self.app_list_state.selected() {
                    return Ok(Some(self.apps[i])); // Return selected app
                }
            }
            KeyCode::Esc => {
                // Add Esc key to return to login screen or exit
                return Ok(Some(SelectedApp::Quit));
            }
            _ => {}
        }

        Ok(None) // No app selected
    }
}

impl Component for Home {
    fn handle_input(&mut self, event: KeyEvent) -> Result<Option<SelectedApp>> {
        self.handle_input(event)
    }

    fn render(&self, frame: &mut Frame) {
        let area = frame.area();

        let vertical_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .margin(1)
            .split(area);

        let welcome_block = Block::default()
            .title("Welcome to Rustoria")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);

        let text = vec![
            Line::from(vec![
                Span::raw("Welcome, "),
                Span::styled("Admin", Style::default().fg(Color::Green)),
            ]),
            Line::from("Please select an application:"),
        ];

        let paragraph = Paragraph::new(text)
            .block(welcome_block)
            .style(Style::default().fg(Color::White).bg(Color::Black))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, vertical_layout[0]);

        let items: Vec<ListItem> = self
            .apps
            .iter()
            .map(|app| {
                let display_text = match app {
                    SelectedApp::Hospital => "Hospital Management",
                    SelectedApp::Pharmacy => "Pharmaceutical Inventory",
                    SelectedApp::None => "None", // Shouldn't be displayed in the list
                    SelectedApp::Quit => "Quit", // Handle the new variant
                };
                ListItem::new(Line::from(vec![display_text.into()]))
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Applications")
                    .border_type(BorderType::Rounded),
            )
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Cyan),
            )
            .highlight_symbol("> ");

        frame.render_stateful_widget(list, vertical_layout[1], &mut self.app_list_state.clone());
    }
}