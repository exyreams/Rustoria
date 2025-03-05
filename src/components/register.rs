//! Registration component for Rustoria.

use crate::app::SelectedApp;
use crate::components::Component;
use crate::tui::Frame;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, Paragraph},
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
                //  Removed extra `return Ok(true);` -  Unnecessary, handled below
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
                        //  Crucially, we now `return Ok(None)` after setting the error.
                        return Ok(None);
                    }
                }
            }
        }
        Ok(None)
    }

    fn render(&self, frame: &mut Frame) {
        // Set the global background color to match the home page
        frame.render_widget(
            Block::default().style(Style::default().bg(Color::Rgb(16, 16, 28))),
            frame.area(),
        );

        // Render a background container for the entire form
        let form_container = centered_rect(70, 70, frame.area());
        let container_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Rgb(75, 75, 120)))
            .style(Style::default().bg(Color::Rgb(22, 22, 35)));

        frame.render_widget(container_block.clone(), form_container);

        // Create layout within the form container
        let vertical_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(2), // Title
                    Constraint::Length(1), // Spacing
                    Constraint::Length(3), // Username
                    Constraint::Length(1), // Spacing
                    Constraint::Length(3), // Password
                    Constraint::Length(1), // Spacing
                    Constraint::Length(3), // Confirm Password
                    Constraint::Length(2), // Error/Success Message
                    Constraint::Length(2), // Back to Login
                    Constraint::Length(1), // Help text
                    Constraint::Min(0),    // Remaining space
                ]
                .as_ref(),
            )
            .margin(2)
            .split(container_block.inner(form_container));

        // --- Title ---
        let title = Paragraph::new("Create Account")
            .style(
                Style::default()
                    .fg(Color::Rgb(230, 230, 250))
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center);
        frame.render_widget(title, vertical_layout[0]);

        // --- Username ---
        let username_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Username ")
            .title_style(
                Style::default()
                    .fg(Color::Rgb(230, 230, 250))
                    .add_modifier(Modifier::BOLD),
            )
            .border_style(if self.focus_index == 0 {
                Style::default().fg(Color::Rgb(250, 250, 110)) // Yellow highlight when selected
            } else {
                Style::default().fg(Color::Rgb(140, 140, 200))
            })
            .style(Style::default().bg(Color::Rgb(26, 26, 36)));

        let username_input = Paragraph::new(self.username.clone())
            .block(username_block)
            .style(Style::default().fg(Color::Rgb(220, 220, 240)))
            .alignment(Alignment::Left);
        frame.render_widget(username_input, vertical_layout[2]);

        // --- Password ---
        let password_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Password ")
            .title_style(
                Style::default()
                    .fg(Color::Rgb(230, 230, 250))
                    .add_modifier(Modifier::BOLD),
            )
            .border_style(if self.focus_index == 1 {
                Style::default().fg(Color::Rgb(250, 250, 110)) // Yellow highlight when selected
            } else {
                Style::default().fg(Color::Rgb(140, 140, 200))
            })
            .style(Style::default().bg(Color::Rgb(26, 26, 36)));

        let password_input = Paragraph::new("•".repeat(self.password.len()))
            .block(password_block)
            .style(Style::default().fg(Color::Rgb(220, 220, 240)))
            .alignment(Alignment::Left);
        frame.render_widget(password_input, vertical_layout[4]);

        // --- Confirm Password ---
        let confirm_password_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Confirm Password ")
            .title_style(
                Style::default()
                    .fg(Color::Rgb(230, 230, 250))
                    .add_modifier(Modifier::BOLD),
            )
            .border_style(if self.focus_index == 2 {
                Style::default().fg(Color::Rgb(250, 250, 110)) // Yellow highlight when selected
            } else {
                Style::default().fg(Color::Rgb(140, 140, 200))
            })
            .style(Style::default().bg(Color::Rgb(26, 26, 36)));

        let confirm_password_input = Paragraph::new("•".repeat(self.confirm_password.len()))
            .block(confirm_password_block)
            .style(Style::default().fg(Color::Rgb(220, 220, 240)))
            .alignment(Alignment::Left);
        frame.render_widget(confirm_password_input, vertical_layout[6]);

        // --- Messages (Error or Success) ---
        if let Some(error) = &self.error_message {
            let error_paragraph = Paragraph::new(error.as_str())
                .style(
                    Style::default()
                        .fg(Color::Rgb(255, 100, 100))
                        .add_modifier(Modifier::BOLD),
                )
                .alignment(Alignment::Center);
            frame.render_widget(error_paragraph, vertical_layout[7]);
        } else if self.registration_success {
            let success_paragraph =
                Paragraph::new("✓ Registration successful! You can now log in.")
                    .style(
                        Style::default()
                            .fg(Color::Rgb(140, 219, 140))
                            .add_modifier(Modifier::BOLD),
                    )
                    .alignment(Alignment::Center);
            frame.render_widget(success_paragraph, vertical_layout[7]);
        }

        // --- Back to Login Text ---
        let back_style = if self.focus_index == 3 {
            Style::default()
                .fg(Color::Rgb(129, 199, 245))
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Rgb(180, 180, 200))
        };

        let back_to_login_text = Paragraph::new(if self.focus_index == 3 {
            "► Back to Login ◄"
        } else {
            "  Back to Login  "
        })
        .style(back_style)
        .alignment(Alignment::Center);
        frame.render_widget(back_to_login_text, vertical_layout[8]);

        // --- Help Text ---
        let help_text =
            Paragraph::new("TAB/Arrow Keys: Navigate | ENTER: Select | ESC: Back to Login")
                .style(Style::default().fg(Color::Rgb(140, 140, 170)))
                .alignment(Alignment::Center);
        frame.render_widget(help_text, vertical_layout[9]);
    }
}
