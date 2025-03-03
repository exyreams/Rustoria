//! Login component for Rustoria.

use crate::components::Component;
use crate::tui::Frame;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, Clear, Paragraph}, // Removed Wrap import
};
use std::time::{Duration, Instant};

/// Represents the login UI component.
#[derive(Debug, Default)]
pub struct Login {
    /// The username input field.
    pub username: String,
    /// The password input field.
    pub password: String,
    /// Flag that indicates whether the username input is active
    pub focus_username: bool,
    /// Optional error message to display.
    pub error_message: Option<String>,
    /// Current selection in the login screen (0: Username, 1: Password, 2: Exit)
    pub selected_index: usize,
    /// Flag to indicate if the exit confirmation dialog is open
    pub show_exit_dialog: bool,
    /// Selected option in the exit dialog (0: Yes, 1: No)
    pub exit_dialog_selected: usize,
    /// Time when the error message was last shown.
    error_message_time: Option<Instant>,
}

impl Login {
    /// Creates a new `Login` component.
    pub fn new() -> Self {
        Self {
            focus_username: true, // Start with focus on the username field.
            selected_index: 0,
            show_exit_dialog: false,
            exit_dialog_selected: 0,
            error_message_time: None,
            ..Default::default()
        }
    }

    /// Handles exit dialog input separately
    fn handle_exit_dialog_input(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Left | KeyCode::Right => {
                self.exit_dialog_selected = 1 - self.exit_dialog_selected; // Toggle
            }
            KeyCode::Enter => {
                if self.exit_dialog_selected == 0 {
                    return Ok(true); // Yes: Exit
                } else {
                    self.show_exit_dialog = false; // No: Close dialog
                }
            }
            KeyCode::Esc => {
                self.show_exit_dialog = false; // Close dialog on Esc
            }
            _ => {}
        }
        Ok(false)
    }

    /// Handles login input separately
    fn handle_login_input(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Char(c) => {
                if self.selected_index == 0 {
                    self.username.push(c);
                } else if self.selected_index == 1 {
                    self.password.push(c);
                }
                self.clear_error_message(); // Clear error on input
            }
            KeyCode::Backspace => {
                if self.selected_index == 0 {
                    self.username.pop();
                } else if self.selected_index == 1 {
                    self.password.pop();
                }
                self.clear_error_message();
            }
            KeyCode::Tab | KeyCode::Down => {
                self.selected_index = (self.selected_index + 1) % 3;
            }
            KeyCode::Up => {
                self.selected_index = (self.selected_index + 2) % 3;
            }
            KeyCode::Enter => {
                if self.selected_index == 2 {
                    self.show_exit_dialog = true; // Show exit dialog
                } else {
                    // Check for empty fields before signaling login attempt
                    if self.selected_index == 0 || self.selected_index == 1 {
                        if self.username.is_empty() {
                            self.set_error_message("Username cannot be empty.".to_string());
                            return Ok(false);
                        }
                        
                        if self.password.is_empty() {
                            self.set_error_message("Password cannot be empty.".to_string());
                            return Ok(false);
                        }
                        
                        return Ok(true); // Signal login attempt
                    }
                }
            }
            KeyCode::Esc => {
                self.show_exit_dialog = !self.show_exit_dialog; // Toggle dialog
            }
            _ => {}
        }
        Ok(false)
    }

    /// Clears the error message and resets the timer.
    fn clear_error_message(&mut self) {
        self.error_message = None;
        self.error_message_time = None;
    }

    /// Updates the error message and sets the timer.
    fn set_error_message(&mut self, message: String) {
        self.error_message = Some(message);
        self.error_message_time = Some(Instant::now());
    }
    
    /// Checks if the error message should be hidden (timeout).
    pub fn check_error_timeout(&mut self) {
        if let Some(time) = self.error_message_time {
            if time.elapsed() >= Duration::from_secs(5) {
                self.clear_error_message();
            }
        }
    }
}

impl Component for Login {
    fn handle_input(&mut self, event: KeyEvent) -> Result<Option<crate::app::SelectedApp>> {
        self.check_error_timeout();

        if self.show_exit_dialog {
            // Handle exit dialog input
            match event.code {
                KeyCode::Left | KeyCode::Right => {
                    self.exit_dialog_selected = 1 - self.exit_dialog_selected; // Toggle
                }
                KeyCode::Enter => {
                    if self.exit_dialog_selected == 0 {
                        // User selected "Yes" to exit - signal application to quit gracefully
                        return Ok(Some(crate::app::SelectedApp::Quit));
                    } else {
                        self.show_exit_dialog = false; // Close dialog on "No"
                    }
                }
                KeyCode::Esc => {
                    self.show_exit_dialog = false; // Close dialog on Esc
                }
                _ => {}
            }
        } else {
            // Handle regular login form input
            match event.code {
                KeyCode::Char(c) => {
                    if self.selected_index == 0 {
                        self.username.push(c);
                    } else if self.selected_index == 1 {
                        self.password.push(c);
                    }
                    self.clear_error_message();
                }
                KeyCode::Backspace => {
                    if self.selected_index == 0 {
                        self.username.pop();
                    } else if self.selected_index == 1 {
                        self.password.pop();
                    }
                    self.clear_error_message();
                }
                KeyCode::Tab | KeyCode::Down => {
                    self.selected_index = (self.selected_index + 1) % 3;
                }
                KeyCode::Up => {
                    self.selected_index = (self.selected_index + 2) % 3;
                }
                KeyCode::Enter => {
                    if self.selected_index == 2 {
                        self.show_exit_dialog = true; // Show exit dialog
                    } else {
                        // Check for empty fields before signaling login attempt
                        if self.username.is_empty() {
                            self.set_error_message("Username cannot be empty.".to_string());
                            return Ok(None);
                        }
                        
                        if self.password.is_empty() {
                            self.set_error_message("Password cannot be empty.".to_string());
                            return Ok(None);
                        }
                        
                        // Signal login attempt
                        return Ok(Some(crate::app::SelectedApp::None));
                    }
                }
                KeyCode::Esc => {
                    self.show_exit_dialog = true; // Show exit dialog on Esc
                }
                _ => {}
            }
        }
        Ok(None)
    }

    fn render(&self, frame: &mut Frame) {
        let vertical_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(7),     // Title
                    Constraint::Length(1),     // Slogan
                    Constraint::Length(2),     // Spacing
                    Constraint::Length(1),     // "Login to Rustoria"
                    Constraint::Length(1),     // spacing
                    Constraint::Length(3),     // Username
                    Constraint::Length(3),     // Password
                    Constraint::Length(3),     // Centered error message
                    Constraint::Length(3),     // Spacing above Exit text
                    Constraint::Length(1),     // Exit text
                    Constraint::Min(0),
                ]
                .as_ref(),
            )
            .margin(1)
            .split(frame.area()); // Correct: Use frame.area()

        // --- Title: Rustoria (using box-drawing characters) ---
        let title = Paragraph::new(Text::from(vec![
            Line::from(
                "██████╗░██╗░░░██╗░██████╗████████╗░█████╗░██████╗░██╗░█████╗░".to_string(),
            ),
            Line::from(
                "██╔══██╗██║░░░██║██╔════╝╚══██╔══╝██╔══██╗██╔══██╗██║██╔══██╗".to_string(),
            ),
            Line::from(
                "██████╔╝██║░░░██║╚█████╗░░░░██║░░░██║░░██║██████╔╝██║███████║".to_string(),
            ),
            Line::from(
                "██╔══██╗██║░░░██║░╚═══██╗░░░██║░░░██║░░██║██╔══██╗██║██╔══██║".to_string(),
            ),
            Line::from(
                "██║░░██║╚██████╔╝██████╔╝░░░██║░░░╚█████╔╝██║░░██║██║██║░░██║".to_string(),
            ),
            Line::from(
                "╚═╝░░╚═╝░╚═════╝░╚═════╝░░░░╚═╝░░░░╚════╝░╚═╝░░╚═╝╚═╝╚═╝░░╚═╝".to_string(),
            ),
        ]))
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Cyan));
        frame.render_widget(title, vertical_layout[0]);

        // --- Slogan ---
        let title_block = Block::default()
            .borders(Borders::NONE);
        let title = Paragraph::new(Text::from(vec![
            Line::from(Span::styled(
                "Seamless Hospital & Pharmacy Operations",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::ITALIC),
            )),
            Line::from(Span::styled(
                "Seamless Hospital & Pharmacy Operations",
                Style::default().fg(Color::Gray),
            )),
        ]))
        .block(title_block)
        .alignment(Alignment::Center);
        frame.render_widget(title, vertical_layout[1]);

        // --- "Login to Rustoria" ---
        let subtitle = Paragraph::new(Span::styled(
            "Login to Rustoria",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ))
        .alignment(Alignment::Center);
        frame.render_widget(subtitle, vertical_layout[3]);

        // --- Username ---
        let username_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(if self.selected_index == 0 {
                " Username (press `TAB` or `Arrow Keys` to switch) "
            } else {
                " Username (press `TAB` or `Arrow Keys` to switch) "
            })
            .style(
                Style::default()
                    .fg(if self.selected_index == 0 {
                        Color::Cyan
                    } else {
                        Color::White
                    }),
            );

        let username_input = Paragraph::new(self.username.clone()).block(username_block);
        frame.render_widget(
            username_input,
            vertical_layout[5].inner(Margin {
                vertical: 0,
                horizontal: 1,
            }),
        );

        // --- Password ---
        let password_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(if self.selected_index == 1 {
                " Password (press `TAB` or `Arrow Keys` to switch) "
            } else {
                " Password (press `TAB` or `Arrow Keys` to switch) "
            })
            .style(
                Style::default()
                    .fg(if self.selected_index == 1 {
                        Color::Cyan
                    } else {
                        Color::White
                    }),
            );
        let password_input = Paragraph::new("•".repeat(self.password.len())).block(password_block);
        frame.render_widget(
            password_input,
            vertical_layout[6].inner(Margin {
                vertical: 0,
                horizontal: 1,
            }),
        );

        // --- Error Message (if any) ---
        if let Some(error) = &self.error_message {
            let error_paragraph = Paragraph::new(error.as_str())
                .style(Style::default().fg(Color::Red))
                .alignment(Alignment::Center); // Center the error message
            frame.render_widget(error_paragraph, vertical_layout[7]);
        }

        // --- Exit Text ---
        let exit_text = Paragraph::new(Span::styled(
            "Exit",
            Style::default()
                .fg(if self.selected_index == 2 {
                    Color::Yellow
                } else {
                    Color::Gray
                })
                .add_modifier(Modifier::BOLD),
        ))
        .alignment(Alignment::Center);
        frame.render_widget(exit_text, vertical_layout[9]);

        // --- Exit Confirmation Dialog ---
        if self.show_exit_dialog {
            let dialog_area = centered_rect(60, 20, frame.area());
            let dialog_block = Block::default()
                .title("Confirm Exit")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded);

            let text = vec![
                Line::from("Are you sure you want to quit?"),
                Line::from(""), // Add an empty line for better spacing
                Line::from(vec![
                    Span::styled(
                        " Yes ",
                         Style::default().fg(if self.exit_dialog_selected == 0 {
                            Color::Green
                        } else {
                            Color::DarkGray
                        }),
                    ),
                    Span::raw("  "), // Keep the spaces for visual separation
                    Span::styled(
                        " No ",
                        Style::default().fg(if self.exit_dialog_selected == 1 {
                            Color::Red
                        } else {
                            Color::DarkGray
                        }),
                    ),
                ]),
            ];

            let dialog_paragraph = Paragraph::new(text)
                .block(dialog_block)
                .alignment(Alignment::Center);

            frame.render_widget(Clear, dialog_area);
            frame.render_widget(dialog_paragraph, dialog_area);
        }
    }
}

/// Helper function to create a centered rectangle.
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}