//! Add Staff component for the Hospital application.
//!
//! This module provides a TUI form for adding new staff members to the hospital system.
//! It includes form validation, error handling, and a user-friendly interface.

use crate::app::SelectedApp;
use crate::components::Component;
use crate::db;
use crate::models::{StaffMember, StaffRole};
use crate::tui::Frame;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{prelude::*, widgets::*};
use std::time::{Duration, Instant};

/// Component to add a new staff member.
///
/// Manages the form state, input validation, and rendering
/// for the staff addition process.
pub struct AddStaff {
    /// Staff member's full name
    name: String,
    /// Staff role selection (Doctor, Nurse, Admin, Technician)
    role: StaffRole,
    /// Contact phone number
    phone: String,
    /// Optional email address
    email: Option<String>,
    /// Staff member's address
    address: String,
    /// Currently focused input field index
    focus_index: usize,
    /// Error message to display (if any)
    error_message: Option<String>,
    /// Timer for auto-clearing error messages
    error_timer: Option<Instant>,
    /// Success message to display (if any)
    success_message: Option<String>,
    /// Timer for auto-clearing success messages
    success_timer: Option<Instant>,
}

/// Constants for form field indices
const INPUT_FIELDS: usize = 5; // Number of input fields
const SUBMIT_BUTTON: usize = 5;
const BACK_BUTTON: usize = 6;

impl AddStaff {
    /// Creates a new `AddStaff` component with default values.
    ///
    /// # Returns
    ///
    /// A new instance of `AddStaff` with empty fields and default role.
    pub fn new() -> Self {
        Self {
            name: String::new(),
            role: StaffRole::Doctor, // Default role
            phone: String::new(),
            email: None,
            address: String::new(),
            focus_index: 0,
            error_message: None,
            error_timer: None,
            success_message: None,
            success_timer: None,
        }
    }

    /// Clears any existing error message.
    fn clear_error(&mut self) {
        self.error_message = None;
        self.error_timer = None;
    }

    /// Sets an error message to display to the user.
    ///
    /// # Arguments
    ///
    /// * `message` - The error message to display
    fn set_error(&mut self, message: String) {
        self.error_message = Some(message);
        self.error_timer = Some(Instant::now());
    }

    /// Clears any existing success message.
    fn clear_success(&mut self) {
        self.success_message = None;
        self.success_timer = None;
    }

    /// Processes input events for the form.
    ///
    /// Handles character input, navigation between fields, and form submission.
    ///
    /// # Arguments
    ///
    /// * `key` - The keyboard event to process
    ///
    /// # Returns
    ///
    /// Optionally returns a new application state if navigation occurs
    fn process_input(&mut self, key: KeyEvent) -> Result<Option<SelectedApp>> {
        self.check_timeouts();
        match key.code {
            KeyCode::Char(c) => {
                match self.focus_index {
                    0 => self.name.push(c),
                    1 => {
                        // Handle Role Selection with key
                        if c.to_ascii_lowercase() == 'd' {
                            self.role = StaffRole::Doctor;
                        } else if c.to_ascii_lowercase() == 'n' {
                            self.role = StaffRole::Nurse;
                        } else if c.to_ascii_lowercase() == 'a' {
                            self.role = StaffRole::Admin;
                        } else if c.to_ascii_lowercase() == 't' {
                            self.role = StaffRole::Technician;
                        }
                    }
                    2 => self.phone.push(c),
                    3 => {
                        if let Some(ref mut email) = self.email {
                            email.push(c);
                        } else {
                            self.email = Some(c.to_string());
                        }
                    }
                    4 => self.address.push(c),
                    _ => {}
                }
                self.clear_error();
            }
            KeyCode::Backspace => {
                match self.focus_index {
                    0 => {
                        self.name.pop();
                    }
                    1 => { /* Role handled separately */ }
                    2 => {
                        self.phone.pop();
                    }
                    3 => {
                        if let Some(email) = self.email.as_mut() {
                            email.pop();
                        }
                    }
                    4 => {
                        self.address.pop();
                    }
                    _ => {}
                }
                self.clear_error();
            }
            KeyCode::Tab => {
                // If we're in any input field (0-4)
                if self.focus_index <= INPUT_FIELDS - 1 {
                    // Jump directly to Submit button
                    self.focus_index = SUBMIT_BUTTON;
                }
                // If we're at Submit button
                else if self.focus_index == SUBMIT_BUTTON {
                    // Go to Back button
                    self.focus_index = BACK_BUTTON;
                }
                // If we're at Back button
                else {
                    // Cycle back to first field
                    self.focus_index = 0;
                }
            }
            KeyCode::Down => {
                self.focus_index = (self.focus_index + 1) % (BACK_BUTTON + 1);
            }
            KeyCode::Up => {
                self.focus_index = (self.focus_index + BACK_BUTTON) % (BACK_BUTTON + 1);
            }
            KeyCode::Left => {
                // No left behavior
            }
            KeyCode::Right => {
                // No right behavior
            }
            KeyCode::Enter => {
                if self.focus_index == BACK_BUTTON {
                    // Back button
                    return Ok(Some(SelectedApp::None));
                }
                if self.focus_index == SUBMIT_BUTTON {
                    // Submit - Validate data
                    if self.name.is_empty() {
                        self.set_error("Name cannot be empty".to_string());
                        return Ok(None);
                    }
                    if self.phone.is_empty() {
                        self.set_error("Phone number cannot be empty".to_string());
                        return Ok(None);
                    }
                    if self.address.is_empty() {
                        self.set_error("Address cannot be empty".to_string());
                        return Ok(None);
                    }

                    let new_staff_member = StaffMember {
                        id: 0, // Database will assign
                        name: self.name.clone(),
                        role: self.role.clone(),
                        phone_number: self.phone.clone(),
                        email: self.email.clone(),
                        address: self.address.clone(),
                    };

                    match db::create_staff_member(&new_staff_member) {
                        Ok(_) => {
                            self.success_message =
                                Some("Staff member added successfully!".to_string());
                            self.success_timer = Some(Instant::now());
                            // Clear form
                            self.name.clear();
                            self.role = StaffRole::Doctor;
                            self.phone.clear();
                            self.email = None;
                            self.address.clear();
                            self.focus_index = 0;
                            self.clear_error();
                        }
                        Err(e) => {
                            self.set_error(format!("Database error: {}", e));
                        }
                    }
                }
            }
            KeyCode::Esc => {
                // Go back to the staff list (or hospital main menu)
                return Ok(Some(SelectedApp::None)); // Go back
            }
            _ => {}
        }

        Ok(None)
    }

    /// Checks if the success message has timed out and clears it if needed.
    fn check_success_timeout(&mut self) {
        if let Some(timer) = self.success_timer {
            if timer.elapsed() > Duration::from_secs(5) {
                self.clear_success();
            }
        }
    }

    /// Checks if the error message has timed out and clears it if needed.
    fn check_error_timeout(&mut self) {
        if let Some(timer) = self.error_timer {
            if timer.elapsed() > Duration::from_secs(5) {
                self.clear_error();
            }
        }
    }

    /// Checks both error and success message timeouts.
    fn check_timeouts(&mut self) {
        self.check_error_timeout();
        self.check_success_timeout();
    }
}

impl Default for AddStaff {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for AddStaff {
    fn handle_input(&mut self, event: KeyEvent) -> Result<Option<SelectedApp>> {
        self.check_timeouts();

        // Directly return the SelectedApp if input leads to a selection.
        if let Some(selected_app) = self.process_input(event)? {
            return Ok(Some(selected_app));
        }
        Ok(None)
    }

    fn render(&self, frame: &mut Frame) {
        // Set the global background color to match our theme
        let area = frame.area();
        frame.render_widget(
            Block::default().style(Style::default().bg(Color::Rgb(16, 16, 28))),
            area,
        );

        // Main layout with header, body, and footer sections
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(22),   // Body content with input fields
                Constraint::Length(0), // 2 spaces above error message
                Constraint::Length(1), // Error message itself
                Constraint::Length(1), // 1 space below error message
                Constraint::Length(6), // Footer for vertical buttons
            ])
            .margin(1)
            .split(frame.area());

        // Header with title
        let header = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(Color::Rgb(75, 75, 120)))
            .style(Style::default().bg(Color::Rgb(16, 16, 28)));
        frame.render_widget(header, main_layout[0]);

        let title = Paragraph::new("👥 ADD STAFF")
            .style(
                Style::default()
                    .fg(Color::Rgb(230, 230, 250))
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Rgb(16, 16, 28)),
            )
            .alignment(Alignment::Center);
        frame.render_widget(title, main_layout[0]);

        // Body container
        let body_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Rgb(75, 75, 120)))
            .style(Style::default().bg(Color::Rgb(22, 22, 35)));

        frame.render_widget(body_block.clone(), main_layout[1]);
        let body_inner = body_block.inner(main_layout[1]);

        // Body section
        let body_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Name
                Constraint::Length(3), // Role
                Constraint::Length(3), // Phone
                Constraint::Length(3), // Email
                Constraint::Length(3), // Address
            ])
            .margin(1)
            .split(body_inner);

        // Name Input
        let name_input = Paragraph::new(self.name.clone())
            .style(
                Style::default()
                    .fg(Color::Rgb(220, 220, 240))
                    .bg(Color::Rgb(26, 26, 36)),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(Span::styled(
                        " Full Name ",
                        Style::default().fg(if self.focus_index == 0 {
                            Color::Rgb(250, 250, 110)
                        } else {
                            Color::Rgb(140, 140, 200)
                        }),
                    ))
                    .border_style(if self.focus_index == 0 {
                        Style::default().fg(Color::Rgb(250, 250, 110))
                    } else {
                        Style::default().fg(Color::Rgb(140, 140, 200))
                    })
                    .style(Style::default().bg(Color::Rgb(26, 26, 36))),
            );
        frame.render_widget(name_input, body_layout[0]);

        // Role as an input field
        let role_text = match self.role {
            StaffRole::Doctor => "Doctor",
            StaffRole::Nurse => "Nurse",
            StaffRole::Admin => "Admin",
            StaffRole::Technician => "Technician",
        };

        let role_input = Paragraph::new(role_text)
            .style(
                Style::default()
                    .fg(Color::Rgb(220, 220, 240))
                    .bg(Color::Rgb(26, 26, 36)),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(Span::styled(
                        " Role (D/N/A/T) ",
                        Style::default().fg(if self.focus_index == 1 {
                            Color::Rgb(250, 250, 110)
                        } else {
                            Color::Rgb(140, 140, 200)
                        }),
                    ))
                    .border_style(if self.focus_index == 1 {
                        Style::default().fg(Color::Rgb(250, 250, 110))
                    } else {
                        Style::default().fg(Color::Rgb(140, 140, 200))
                    })
                    .style(Style::default().bg(Color::Rgb(26, 26, 36))),
            );
        frame.render_widget(role_input, body_layout[1]);

        // Phone
        let phone_input = Paragraph::new(self.phone.clone())
            .style(
                Style::default()
                    .fg(Color::Rgb(220, 220, 240))
                    .bg(Color::Rgb(26, 26, 36)),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(Span::styled(
                        " Phone ",
                        Style::default().fg(if self.focus_index == 2 {
                            Color::Rgb(250, 250, 110)
                        } else {
                            Color::Rgb(140, 140, 200)
                        }),
                    ))
                    .border_style(if self.focus_index == 2 {
                        Style::default().fg(Color::Rgb(250, 250, 110))
                    } else {
                        Style::default().fg(Color::Rgb(140, 140, 200))
                    })
                    .style(Style::default().bg(Color::Rgb(26, 26, 36))),
            );
        frame.render_widget(phone_input, body_layout[2]);

        // Email
        let email_input = Paragraph::new(self.email.clone().unwrap_or_default())
            .style(
                Style::default()
                    .fg(Color::Rgb(220, 220, 240))
                    .bg(Color::Rgb(26, 26, 36)),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(Span::styled(
                        " Email ",
                        Style::default().fg(if self.focus_index == 3 {
                            Color::Rgb(250, 250, 110)
                        } else {
                            Color::Rgb(140, 140, 200)
                        }),
                    ))
                    .border_style(if self.focus_index == 3 {
                        Style::default().fg(Color::Rgb(250, 250, 110))
                    } else {
                        Style::default().fg(Color::Rgb(140, 140, 200))
                    })
                    .style(Style::default().bg(Color::Rgb(26, 26, 36))),
            );
        frame.render_widget(email_input, body_layout[3]);

        // Address
        let address_input = Paragraph::new(self.address.clone())
            .style(
                Style::default()
                    .fg(Color::Rgb(220, 220, 240))
                    .bg(Color::Rgb(26, 26, 36)),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(Span::styled(
                        " Address ",
                        Style::default().fg(if self.focus_index == 4 {
                            Color::Rgb(250, 250, 110)
                        } else {
                            Color::Rgb(140, 140, 200)
                        }),
                    ))
                    .border_style(if self.focus_index == 4 {
                        Style::default().fg(Color::Rgb(250, 250, 110))
                    } else {
                        Style::default().fg(Color::Rgb(140, 140, 200))
                    })
                    .style(Style::default().bg(Color::Rgb(26, 26, 36))),
            );
        frame.render_widget(address_input, body_layout[4]);

        // Error or success message area
        let status_message = if let Some(success) = &self.success_message {
            Paragraph::new(format!("✓ {}", success))
                .style(
                    Style::default()
                        .fg(Color::Rgb(140, 219, 140))
                        .add_modifier(Modifier::BOLD)
                        .bg(Color::Rgb(16, 16, 28)),
                )
                .alignment(Alignment::Center)
        } else if let Some(error) = &self.error_message {
            Paragraph::new(format!("⚠️ {}", error))
                .style(
                    Style::default()
                        .fg(Color::Rgb(255, 100, 100))
                        .add_modifier(Modifier::BOLD)
                        .bg(Color::Rgb(16, 16, 28)),
                )
                .alignment(Alignment::Center)
        } else {
            Paragraph::new("").style(Style::default().bg(Color::Rgb(16, 16, 28)))
        };
        frame.render_widget(status_message, main_layout[3]);

        let footer_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Submit button
                Constraint::Length(2), // Back button
                Constraint::Min(2),    // Help text
            ])
            .split(main_layout[5]);

        // Submit button - simple text version
        let submit_text = if self.focus_index == SUBMIT_BUTTON {
            "► Submit ◄"
        } else {
            "  Submit  "
        };

        let submit_style = if self.focus_index == SUBMIT_BUTTON {
            Style::default()
                .fg(Color::Rgb(140, 219, 140))
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Rgb(180, 180, 200))
        };

        let submit_button = Paragraph::new(submit_text)
            .style(submit_style)
            .alignment(Alignment::Center);
        frame.render_widget(submit_button, footer_layout[0]);

        // Back button - simple text version
        let back_text = if self.focus_index == BACK_BUTTON {
            "► Back ◄"
        } else {
            "  Back  "
        };

        let back_style = if self.focus_index == BACK_BUTTON {
            Style::default()
                .fg(Color::Rgb(129, 199, 245))
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Rgb(180, 180, 200))
        };

        let back_button = Paragraph::new(back_text)
            .style(back_style)
            .alignment(Alignment::Center);
        frame.render_widget(back_button, footer_layout[1]);

        // Help text
        let help_text = "Tab: Switch Focus | Arrow Keys: Switch Fields | Enter: Submit | Esc: Back\nFor Role: Type 'D' for Doctor, 'N' for Nurse, 'A' for Admin, 'T' for Technician";
        let help_paragraph = Paragraph::new(help_text)
            .style(
                Style::default()
                    .fg(Color::Rgb(140, 140, 170))
                    .bg(Color::Rgb(16, 16, 28)),
            )
            .alignment(Alignment::Center);
        frame.render_widget(help_paragraph, footer_layout[2]);
    }
}
