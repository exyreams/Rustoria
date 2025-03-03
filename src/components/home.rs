//! Home component for Rustoria.

use crate::app::SelectedApp;
use crate::components::Component;
use crate::db;
use crate::tui::Frame;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Padding, Paragraph},
};

/// Represents the home screen UI component.
pub struct Home {
    /// The selected application index.
    selected_app_index: usize,
    /// The available applications.
    apps: Vec<SelectedApp>,
    /// The logged-in user's username.
    username: Option<String>,
    /// Selection mode: 0 for apps, 1 for back button
    selection_mode: usize,
    /// Hospital menu state
    hospital_menu_state: ListState,
    /// Pharmacy menu state
    pharmacy_menu_state: ListState,
    /// Show logout confirmation dialog
    show_logout_dialog: bool,
    /// Logout dialog selected option (0: Yes, 1: No)
    logout_dialog_selected: usize,
}

impl Home {
    /// Creates a new `Home` component.
    pub fn new() -> Self {
        let mut hospital_state = ListState::default();
        hospital_state.select(Some(0));

        let mut pharmacy_state = ListState::default();
        pharmacy_state.select(Some(0));

        Self {
            selected_app_index: 0,
            apps: vec![SelectedApp::Hospital, SelectedApp::Pharmacy],
            username: None,
            selection_mode: 0,
            hospital_menu_state: hospital_state,
            pharmacy_menu_state: pharmacy_state,
            show_logout_dialog: false,
            logout_dialog_selected: 0,
        }
    }

    /// Loads the username from the database.
    pub fn load_username(&mut self, user_id: i64) -> Result<()> {
        self.username = Some(db::get_username(user_id)?);
        Ok(())
    }

    pub fn handle_input(&mut self, key: KeyEvent) -> Result<Option<SelectedApp>> {
        if self.show_logout_dialog {
            return self.handle_logout_dialog_input(key);
        }

        match key.code {
            KeyCode::Tab => {
                // Toggle between app selection and back button
                self.selection_mode = (self.selection_mode + 1) % 2;
            }
            KeyCode::Left | KeyCode::Right => {
                if self.selection_mode == 0 {
                    // Toggle between applications
                    self.selected_app_index = (self.selected_app_index + 1) % self.apps.len();
                }
            }
            KeyCode::Up => {
                if self.selection_mode == 0 {
                    let selected_app = self.apps[self.selected_app_index];
                    match selected_app {
                        SelectedApp::Hospital => {
                            if let Some(i) = self.hospital_menu_state.selected() {
                                let i = if i == 0 { 5 } else { i - 1 };
                                self.hospital_menu_state.select(Some(i));
                            }
                        }
                        SelectedApp::Pharmacy => {
                            if let Some(i) = self.pharmacy_menu_state.selected() {
                                let i = if i == 0 { 4 } else { i - 1 };
                                self.pharmacy_menu_state.select(Some(i));
                            }
                        }
                        _ => {}
                    }
                }
            }
            KeyCode::Down => {
                if self.selection_mode == 0 {
                    let selected_app = self.apps[self.selected_app_index];
                    match selected_app {
                        SelectedApp::Hospital => {
                            if let Some(i) = self.hospital_menu_state.selected() {
                                let i = if i == 5 { 0 } else { i + 1 };
                                self.hospital_menu_state.select(Some(i));
                            }
                        }
                        SelectedApp::Pharmacy => {
                            if let Some(i) = self.pharmacy_menu_state.selected() {
                                let i = if i == 4 { 0 } else { i + 1 };
                                self.pharmacy_menu_state.select(Some(i));
                            }
                        }
                        _ => {}
                    }
                }
            }
            KeyCode::Enter => {
                if self.selection_mode == 0 {
                    // Select application
                    return Ok(Some(self.apps[self.selected_app_index]));
                } else {
                    // Logout button - show confirmation dialog
                    self.show_logout_dialog = true;
                    self.logout_dialog_selected = 1; // Default to "No"
                    return Ok(None);
                }
            }
            KeyCode::Esc => {
                // Esc also shows logout confirmation
                self.show_logout_dialog = true;
                self.logout_dialog_selected = 1; // Default to "No"
                return Ok(None);
            }
            _ => {}
        }

        Ok(None)
    }

    fn handle_logout_dialog_input(&mut self, key: KeyEvent) -> Result<Option<SelectedApp>> {
        match key.code {
            KeyCode::Left | KeyCode::Right => {
                self.logout_dialog_selected = 1 - self.logout_dialog_selected; // Toggle between Yes/No
            }
            KeyCode::Enter => {
                self.show_logout_dialog = false;
                if self.logout_dialog_selected == 0 {
                    // Yes selected - logout
                    return Ok(Some(SelectedApp::None));
                }
            }
            KeyCode::Esc => {
                self.show_logout_dialog = false; // Cancel dialog
            }
            _ => {}
        }
        Ok(None)
    }
}

impl Component for Home {
    fn handle_input(&mut self, event: KeyEvent) -> Result<Option<SelectedApp>> {
        self.handle_input(event)
    }

    fn render(&self, frame: &mut Frame) {
        let area = frame.area();

        // Main vertical layout
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4), // Welcome area (reduced height)
                Constraint::Length(1), // Spacer
                Constraint::Length(1), // Instruction text
                Constraint::Min(10),   // Apps area
                Constraint::Length(3), // Spacer for help text
                Constraint::Length(2), // Back button area
            ])
            .split(area);

        // Welcome banner
        let username = self.username.as_deref().unwrap_or("User");
        let welcome_text = Line::from(vec![
            Span::styled(
                "Welcome, ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                username,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]);

        let welcome_paragraph = Paragraph::new(welcome_text)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::NONE)
                    .padding(Padding::new(2, 2, 1, 1)),
            );

        frame.render_widget(welcome_paragraph, main_layout[0]);

        // Instruction text
        let instruction = Paragraph::new("Please select an application:")
            .style(Style::default().fg(Color::LightBlue))
            .alignment(Alignment::Center);

        frame.render_widget(instruction, main_layout[2]);

        // Apps area
        let apps_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .spacing(4)
            .margin(2)
            .split(main_layout[3]);

        // Render application options
        let app_titles = [" 🏥 Hospital Management ", " 💊 Pharmacy Management "];

        for (i, &app) in self.apps.iter().enumerate() {
            let is_selected = i == self.selected_app_index && self.selection_mode == 0;

            let app_name = match app {
                SelectedApp::Hospital => app_titles[0],
                SelectedApp::Pharmacy => app_titles[1],
                _ => "Unknown",
            };

            let border_style = if is_selected {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::Gray)
            };

            let block = Block::default()
                .title(app_name)
                .title_style(border_style)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(border_style);

            frame.render_widget(block.clone(), apps_layout[i]);

            // Render menu options for both apps, always
            let inner_area = block.inner(apps_layout[i]);

            // Add padding at the top of options
            let inner_padded = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1), // Top padding
                    Constraint::Min(1),    // Content area
                ])
                .split(inner_area);

            match app {
                SelectedApp::Hospital => {
                    let hospital_options = vec![
                        "Patient Management",
                        "Staff Scheduling",
                        "Billing & Financials",
                        "Medical Records",
                        "Inventory Management",
                        "Reports & Analytics",
                    ];

                    let selected = self.hospital_menu_state.selected();

                    let items: Vec<ListItem> = hospital_options
                        .iter()
                        .enumerate()
                        .map(|(idx, option)| {
                            let style = if is_selected && selected == Some(idx) {
                                Style::default()
                                    .fg(Color::Cyan)
                                    .add_modifier(Modifier::BOLD)
                            } else {
                                Style::default().fg(Color::Gray)
                            };

                            let prefix = if is_selected && selected == Some(idx) {
                                " > "
                            } else {
                                "  "
                            };
                            ListItem::new(format!("{}{}", prefix, option)).style(style)
                        })
                        .collect();

                    let list = List::new(items);

                    frame.render_widget(list, inner_padded[1]);
                }
                SelectedApp::Pharmacy => {
                    let pharmacy_options = vec![
                        "Inventory Management",
                        "Procurement Management",
                        "Distribution & Sales",
                        "Expiry & Wastage Management",
                        "Reporting & Analytics",
                    ];

                    let selected = self.pharmacy_menu_state.selected();

                    let items: Vec<ListItem> = pharmacy_options
                        .iter()
                        .enumerate()
                        .map(|(idx, option)| {
                            let style = if is_selected && selected == Some(idx) {
                                Style::default()
                                    .fg(Color::Cyan)
                                    .add_modifier(Modifier::BOLD)
                            } else {
                                Style::default().fg(Color::Gray)
                            };

                            let prefix = if is_selected && selected == Some(idx) {
                                " > "
                            } else {
                                "  "
                            };
                            ListItem::new(format!("{}{}", prefix, option)).style(style)
                        })
                        .collect();

                    let list = List::new(items);

                    frame.render_widget(list, inner_padded[1]);
                }
                _ => {}
            }
        }

        // Logout link
        let back_text = if self.selection_mode == 1 {
            "◀ Logout"
        } else {
            "Logout"
        };

        let back_style = if self.selection_mode == 1 {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };

        let back_paragraph = Paragraph::new(back_text)
            .style(back_style)
            .alignment(Alignment::Center);

        frame.render_widget(back_paragraph, main_layout[5]);

        // Help text at the bottom with added spacing
        let help_area = main_layout[4];

        let help_text = "Tab: Switch focus | Left/Right: Change selection | Up/Down: Navigate menu | Enter: Select";
        let help_paragraph = Paragraph::new(help_text)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);

        frame.render_widget(help_paragraph, help_area);

        // Render logout confirmation dialog if needed
        if self.show_logout_dialog {
            self.render_logout_dialog(frame, area);
        }
    }
}

impl Home {
    fn render_logout_dialog(&self, frame: &mut Frame, area: Rect) {
        let dialog_width = 40;
        let dialog_height = 8;

        let dialog_area = Rect::new(
            (area.width.saturating_sub(dialog_width)) / 2,
            (area.height.saturating_sub(dialog_height)) / 2,
            dialog_width,
            dialog_height,
        );

        // Clear the background
        frame.render_widget(Clear, dialog_area);

        // Render dialog box
        let dialog_block = Block::default()
            .title(" Confirm Logout ")
            .add_modifier(Modifier::BOLD)
            .title_style(Style::default().fg(Color::LightBlue))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::LightBlue));

        frame.render_widget(dialog_block.clone(), dialog_area);

        // Dialog content
        let inner_area = dialog_block.inner(dialog_area);

        let content_layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(2), // Message
                Constraint::Length(2), // Buttons
            ])
            .split(inner_area);

        let message = Paragraph::new("Are you sure you want to logout?")
            .style(Style::default().fg(Color::White))
            .add_modifier(Modifier::BOLD)
            .alignment(Alignment::Center);

        frame.render_widget(message, content_layout[0]);

        // Buttons
        let buttons_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(content_layout[1]);

        let yes_style = if self.logout_dialog_selected == 0 {
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };

        let no_style = if self.logout_dialog_selected == 1 {
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };

        let yes_button = Paragraph::new("Yes")
            .style(yes_style)
            .alignment(Alignment::Center);

        let no_button = Paragraph::new("No")
            .style(no_style)
            .alignment(Alignment::Center);

        frame.render_widget(yes_button, buttons_layout[0]);
        frame.render_widget(no_button, buttons_layout[1]);
    }
}
