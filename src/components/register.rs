//! Registration component for Rustoria.

use crate::app::SelectedApp;
use crate::components::Component;
use crate::tui::Frame;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};
use std::time::{Duration, Instant};

/// Represents the registration UI component.
#[derive(Debug, Default)]
pub struct Register {
    /// The username input field.
    pub username: String,
    /// The password input field.
    pub password: String,
    /// The confirm password input field.
    pub confirm_password: String,
    /// Flag to track which input field has focus (0: username, 1: password, 2: confirm, 3: Back)
    focus_index: usize,
    /// Optional error message to display.
    pub error_message: Option<String>,
    /// Time when the error message was last shown.
    error_message_time: Option<Instant>,
    /// Flag to indicate successful registration (for displaying a message)
    pub registration_success: bool,
}

impl Register {
    /// Creates a new `Register` component.
    pub fn new() -> Self {
        Self::default()
    }

    fn handle_register_input(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Char(c) => {
                match self.focus_index {
                    0 => self.username.push(c),
                    1 => self.password.push(c),
                    2 => self.confirm_password.push(c),
                    _ => {}
                }
                self.clear_error_message(); // Clear error on any input
                self.registration_success = false; // Clear success message
            }
            KeyCode::Backspace => {
                match self.focus_index {
                    0 => self.username.pop(),
                    1 => self.password.pop(),
                    2 => self.confirm_password.pop(),
                    _ => None,
                };
                self.clear_error_message();
                self.registration_success = false;
            }
            KeyCode::Tab | KeyCode::Down => {
                self.focus_index = (self.focus_index + 1) % 4; // 4 options now
            }
            KeyCode::Up => {
                self.focus_index = (self.focus_index + 3) % 4; // 4 options
            }
            KeyCode::Enter => {
                if self.focus_index == 3 {
                    // Back to Login selected
                    return Ok(true);
                }
                // Return true to signal registration attempt (if not back).
                return Ok(true);
            }
            KeyCode::Esc => {
                // ESC key pressed - go back to login
                return Ok(true); // Return true to signal we want to go back
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

/// Helper function to create a centered rect using up certain percentage of the available rect
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

impl Component for Register {
    fn handle_input(&mut self, event: KeyEvent) -> Result<Option<crate::app::SelectedApp>> {
        self.check_error_timeout(); // Check for timeout

        // Handle input and return early if it's the "Back" button.
        if self.handle_register_input(event)? {
            if self.focus_index == 3 || event.code == KeyCode::Esc {
                return Ok(Some(SelectedApp::None)); // Back to login
            } else {
                // Validate fields *before* database call
                if self.username.is_empty() {
                    self.set_error_message("Username cannot be empty.".to_string());
                    return Ok(None); // Return early on error
                }
                if self.password.is_empty() {
                    self.set_error_message("Password cannot be empty.".to_string());
                    return Ok(None); // Return early on error
                }
                if self.password != self.confirm_password {
                    self.set_error_message("Passwords do not match.".to_string());
                    return Ok(None); // Return early on error
                }

                // Attempt registration (delegated to db module)
                match crate::db::create_user(&self.username, &self.password) {
                    Ok(_) => {
                        // Successful registration
                        self.registration_success = true; // Set success flag

                        // Optionally: Clear fields on success (good UX)
                        self.username.clear();
                        self.password.clear();
                        self.confirm_password.clear();
                        return Ok(Some(SelectedApp::None));
                    }
                    Err(err) => {
                        // Display error message
                        self.set_error_message(format!("{}", err));
                    }
                }
            }
        }
        Ok(None)
    }

    fn render(&self, frame: &mut Frame) {
        let vertical_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(1), // Title
                    Constraint::Length(1), // Spacing
                    Constraint::Length(3), // Username
                    Constraint::Length(1), // Username error
                    Constraint::Length(3), // Password
                    Constraint::Length(1), // Password error
                    Constraint::Length(3), // Confirm Password
                    Constraint::Length(1), // Confirm Password error
                    Constraint::Length(2), //  Spacing
                    Constraint::Length(1), // Back to Login text
                    Constraint::Length(2), // Spacing before help text
                    Constraint::Length(1), // Help text
                    Constraint::Min(0),    // Remaining space
                ]
                .as_ref(),
            )
            .margin(1)
            .split(frame.area());

        // --- Title ---
        let title = Paragraph::new("Create Account")
            .style(
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center);
        frame.render_widget(title, vertical_layout[0]);

        // --- Username ---
        let username_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(if self.focus_index == 0 {
                " Username "
            } else {
                " Username "
            })
            .style(Style::default().fg(if self.focus_index == 0 {
                Color::Cyan
            } else {
                Color::White
            }));

        // Create a narrower area for the username field (60% of width, centered)
        let username_area = centered_rect(60, 100, vertical_layout[2]);
        let username_input = Paragraph::new(self.username.clone())
            .block(username_block)
            .alignment(Alignment::Left);
        frame.render_widget(username_input, username_area);

        // --- Password ---
        let password_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(if self.focus_index == 1 {
                " Password "
            } else {
                " Password "
            })
            .style(Style::default().fg(if self.focus_index == 1 {
                Color::Cyan
            } else {
                Color::White
            }));

        // Create a narrower area for the password field (60% of width, centered)
        let password_area = centered_rect(60, 100, vertical_layout[4]);
        let password_input = Paragraph::new("•".repeat(self.password.len()))
            .block(password_block)
            .alignment(Alignment::Left);
        frame.render_widget(password_input, password_area);

        // --- Confirm Password ---
        let confirm_password_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(if self.focus_index == 2 {
                " Confirm Password "
            } else {
                " Confirm Password "
            })
            .style(Style::default().fg(if self.focus_index == 2 {
                Color::Cyan
            } else {
                Color::White
            }));

        // Create a narrower area for the confirm password field (60% of width, centered)
        let confirm_password_area = centered_rect(60, 100, vertical_layout[6]);
        let confirm_password_input = Paragraph::new("•".repeat(self.confirm_password.len()))
            .block(confirm_password_block)
            .alignment(Alignment::Left);
        frame.render_widget(confirm_password_input, confirm_password_area);

        // --- Error Message (if any) ---
        if let Some(error) = &self.error_message {
            let error_paragraph = Paragraph::new(error.as_str())
                .style(Style::default().fg(Color::Red))
                .alignment(Alignment::Center);
            frame.render_widget(error_paragraph, vertical_layout[7]); //  Below confirm password
        }

        // --- Back to Login Text ---
        let back_to_login_text = Paragraph::new(Span::styled(
            if self.focus_index == 3 {
                "◀ Back to Login"
            } else {
                "  Back to Login"
            },
            Style::default()
                .fg(if self.focus_index == 3 {
                    Color::Cyan
                } else {
                    Color::Gray
                })
                .add_modifier(Modifier::BOLD),
        ))
        .alignment(Alignment::Center);
        frame.render_widget(back_to_login_text, vertical_layout[9]);

        // --- Help Text ---
        let help_text = Paragraph::new(vec![Line::from(Span::styled(
            "TAB/Arrow Keys: Navigate | ENTER: Select | ESC: Back to Login",
            Style::default().fg(Color::DarkGray),
        ))])
        .alignment(Alignment::Center);
        frame.render_widget(help_text, vertical_layout[11]);
    }
}
