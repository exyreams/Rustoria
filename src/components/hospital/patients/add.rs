//! Add Patient component for the Hospital application.

use crate::components::hospital::patients::PatientAction;
use crate::components::Component;
use crate::db;
use crate::models::{Gender, Patient};
use crate::tui::Frame;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{prelude::*, widgets::*};
use std::time::{Duration, Instant};

pub struct AddPatient {
    first_name: String,
    last_name: String,
    dob: String,
    gender: Gender,
    address: String,
    phone: String,
    email: Option<String>,           // Optional
    medical_history: Option<String>, // Optional
    allergies: Option<String>,       // Optional
    medications: Option<String>,     // Optional
    focus_index: usize,              // Track which input field has focus
    error_message: Option<String>,   // For displaying errors
    error_timer: Option<Instant>,    // Timer for error message
    success_message: Option<String>, // For displaying success message
    success_timer: Option<Instant>,  // Timer for success message
}

const INPUT_FIELDS: usize = 10; // Number of input fields + 1 for "Back"

impl Default for AddPatient {
    fn default() -> Self {
        AddPatient {
            first_name: String::new(),
            last_name: String::new(),
            dob: String::new(),   // Use a date type later
            gender: Gender::Male, // Default, consider making this optional
            address: String::new(),
            phone: String::new(),
            email: None,
            medical_history: None,
            allergies: None,
            medications: None,
            focus_index: 0,
            error_message: None,
            error_timer: None,
            success_message: None,
            success_timer: None,
        }
    }
}

impl AddPatient {
    pub fn new() -> Self {
        Self::default() // Use Default to initialize
    }

    fn clear_error(&mut self) {
        self.error_message = None;
        self.error_timer = None;
    }

    fn set_error(&mut self, message: String) {
        self.error_message = Some(message);
        self.error_timer = Some(Instant::now());
    }

    pub fn check_error_timeout(&mut self) {
        if let Some(timer) = self.error_timer {
            if timer.elapsed() > Duration::from_secs(5) {
                self.clear_error();
            }
        }
    }

    fn clear_success(&mut self) {
        self.success_message = None;
        self.success_timer = None;
    }

    #[allow(dead_code)]
    fn set_success(&mut self, message: String) {
        self.success_message = Some(message);
        self.success_timer = Some(Instant::now());
    }

    pub fn check_success_timeout(&mut self) {
        if let Some(timer) = self.success_timer {
            if timer.elapsed() > Duration::from_secs(5) {
                self.clear_success();
            }
        }
    }

    pub fn handle_input(&mut self, key: KeyEvent) -> Result<Option<PatientAction>> {
        self.check_error_timeout(); // Clear error messages if timed out
        self.check_success_timeout(); // Clear success messages if timed out
        match key.code {
            KeyCode::Char(c) => {
                match self.focus_index {
                    0 => self.first_name.push(c),
                    1 => self.last_name.push(c),
                    2 => self.dob.push(c),
                    3 => {
                        // Handle gender as text input
                        if c.to_ascii_lowercase() == 'f' {
                            self.gender = Gender::Female;
                        } else if c.to_ascii_lowercase() == 'm' {
                            self.gender = Gender::Male;
                        } else if c.to_ascii_lowercase() == 'o' {
                            self.gender = Gender::Other; // Assuming you have an Other variant
                        }
                    }
                    4 => self.address.push(c),
                    5 => self.phone.push(c),
                    6 => {
                        if let Some(ref mut email) = self.email {
                            email.push(c);
                        } else {
                            self.email = Some(c.to_string());
                        }
                    }
                    7 => {
                        if let Some(ref mut history) = self.medical_history {
                            history.push(c);
                        } else {
                            self.medical_history = Some(c.to_string());
                        }
                    }
                    8 => {
                        if let Some(ref mut allergies) = self.allergies {
                            allergies.push(c);
                        } else {
                            self.allergies = Some(c.to_string());
                        }
                    }
                    9 => {
                        if let Some(ref mut medications) = self.medications {
                            medications.push(c);
                        } else {
                            self.medications = Some(c.to_string());
                        }
                    }
                    _ => {}
                }
                self.clear_error(); // Clear error on valid input
            }
            KeyCode::Backspace => {
                match self.focus_index {
                    0 => self.first_name.pop(),
                    1 => self.last_name.pop(),
                    2 => self.dob.pop(),
                    3 => None, // Gender selection - handled elsewhere
                    4 => self.address.pop(),
                    5 => self.phone.pop(),
                    6 => self.email.as_mut().and_then(|email| email.pop()),
                    7 => {
                        if let Some(ref mut history) = self.medical_history {
                            history.pop() // Return the Option<char> from pop()
                        } else {
                            None // Return None if there's no history
                        }
                    }
                    8 => {
                        if let Some(ref mut allergies) = self.allergies {
                            allergies.pop() // Remove the semicolon to return the Option<char>
                        } else {
                            None // Return None if there are no allergies
                        }
                    }
                    9 => {
                        if let Some(ref mut medications) = self.medications {
                            medications.pop() // Remove the semicolon to return the Option<char>
                        } else {
                            None // Return None if there are no medications
                        }
                    }
                    _ => None,
                };
                self.clear_error();
            }
            KeyCode::Tab => {
                // If we're in any input field (0-9)
                if self.focus_index <= 9 {
                    // Jump directly to Submit button
                    self.focus_index = INPUT_FIELDS;
                }
                // If we're at Submit button
                else if self.focus_index == INPUT_FIELDS {
                    // Go to Back button
                    self.focus_index = INPUT_FIELDS + 1;
                }
                // If we're at Back button
                else {
                    // Cycle back to first field
                    self.focus_index = 0;
                }
            }
            KeyCode::Down => {
                // Original navigation behavior - field by field
                self.focus_index = (self.focus_index + 1) % (INPUT_FIELDS + 2);
            }
            KeyCode::Up => {
                self.focus_index = (self.focus_index + INPUT_FIELDS + 1) % (INPUT_FIELDS + 2);
                // +2 to include Submit and Back
            }
            KeyCode::Left => {
                // If we're in the right column (email, medical history, allergies, medications),
                // move to the corresponding field in the left column
                if self.focus_index >= 6 && self.focus_index <= 9 {
                    self.focus_index -= 6;
                }
            }
            KeyCode::Right => {
                // If we're in the left column (name, dob, gender, address, phone),
                // move to the corresponding field in the right column
                if self.focus_index <= 5 {
                    self.focus_index = std::cmp::min(self.focus_index + 6, 9);
                }
            }
            KeyCode::Esc => {
                return Ok(Some(PatientAction::BackToHome));
            }
            KeyCode::Enter => {
                if self.focus_index == INPUT_FIELDS + 1 {
                    // "Back" button
                    return Ok(Some(PatientAction::BackToHome));
                } else if self.focus_index == INPUT_FIELDS {
                    // "Submit" button
                    // All fields entered. Add the patient.
                    if self.first_name.is_empty() {
                        self.set_error("First Name cannot be empty".to_string());
                        return Ok(None);
                    }
                    if self.last_name.is_empty() {
                        self.set_error("Last Name cannot be empty".to_string());
                        return Ok(None);
                    }
                    if self.dob.is_empty() {
                        self.set_error("Date of Birth cannot be empty".to_string());
                        return Ok(None);
                    }
                    if self.address.is_empty() {
                        self.set_error("Address cannot be empty".to_string());
                        return Ok(None);
                    }
                    if self.phone.is_empty() {
                        self.set_error("Phone Number cannot be empty".to_string());
                        return Ok(None);
                    }

                    let new_patient = Patient {
                        id: 0, // The database will assign the ID
                        first_name: self.first_name.clone(),
                        last_name: self.last_name.clone(),
                        date_of_birth: self.dob.clone(), // Use a date type later
                        gender: self.gender.clone(),
                        address: self.address.clone(),
                        phone_number: self.phone.clone(),
                        email: self.email.clone(),
                        medical_history: self.medical_history.clone(),
                        allergies: self.allergies.clone(),
                        current_medications: self.medications.clone(),
                    };

                    match db::create_patient(&new_patient) {
                        Ok(_) => {
                            // Successfully added to database. Now, clear the fields
                            self.first_name.clear();
                            self.last_name.clear();
                            self.dob.clear();
                            self.gender = Gender::Male; // Reset
                            self.address.clear();
                            self.phone.clear();
                            self.email = None;
                            self.medical_history = None;
                            self.allergies = None;
                            self.medications = None;
                            self.focus_index = 0; // Reset

                            // Display a success message (you could add this)
                            self.success_message = Some("Patient added successfully!".to_string());

                            // Clear any error message
                            self.clear_error();

                            // return Ok(Some(PatientAction::BackToHome)); // Or stay on add screen
                            return Ok(None); // stay on add screen for now.
                        }
                        Err(e) => {
                            self.set_error(format!("Database error: {}", e));
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(None)
    }
}

impl Component for AddPatient {
    fn handle_input(&mut self, event: KeyEvent) -> Result<Option<crate::app::SelectedApp>> {
        // Delegate the PatientAction and convert it.
        match self.handle_input(event)? {
            Some(PatientAction::BackToHome) => Ok(Some(crate::app::SelectedApp::None)),
            None => Ok(None),
        }
    }

    fn render(&self, frame: &mut Frame) {
        // Set the overall background color to black
        let area = frame.area();
        frame.render_widget(
            Block::default().style(Style::default().bg(Color::Black)),
            area,
        );

        // Main layout with header, body, and footer sections
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(22),   // Body content with input fields
                Constraint::Length(2), // 2 spaces above error message
                Constraint::Length(1), // Error message itself
                Constraint::Length(1), // 1 space below error message
                Constraint::Length(6), // Footer for vertical buttons
            ])
            .margin(1)
            .split(frame.area());

        // Header with title
        let header = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(Color::Cyan))
            .style(Style::default().bg(Color::Black));
        frame.render_widget(header, main_layout[0]);

        let title = Paragraph::new("🏥 PATIENT REGISTRATION")
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Black),
            )
            .alignment(Alignment::Center);
        frame.render_widget(title, main_layout[0]);

        // Body section - split into two columns
        let body_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(main_layout[1]);

        // Left column for primary info - FIXED: removed extra spacing
        let left_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Section title
                Constraint::Length(3), // First Name
                Constraint::Length(3), // Last Name
                Constraint::Length(3), // DOB
                Constraint::Length(3), // Gender
                Constraint::Length(3), // Address
                Constraint::Length(3), // Phone
            ])
            .margin(1)
            .split(body_layout[0]);

        // Right column for secondary info - FIXED: removed extra spacing
        let right_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Section title
                Constraint::Length(3), // Email
                Constraint::Length(5), // Medical History (taller)
                Constraint::Length(5), // Allergies (taller)
                Constraint::Length(5), // Medications (taller)
            ])
            .margin(1)
            .split(body_layout[1]);

        // Section titles - removed borders
        let primary_title = Paragraph::new("● REQUIRED INFORMATION").style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
                .bg(Color::Black),
        );
        frame.render_widget(primary_title, left_layout[0]);

        let secondary_title = Paragraph::new("○ OPTIONAL INFORMATION").style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
                .bg(Color::Black),
        );
        frame.render_widget(secondary_title, right_layout[0]);

        // Primary Info Fields (Left Column)
        // Each field gets styled differently when focused
        let required_style = Style::default().fg(Color::Cyan);

        // First Name (required)
        let first_name_input = Paragraph::new(self.first_name.clone())
            .style(Style::default().bg(Color::Black))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(if self.focus_index == 0 {
                        BorderType::Thick
                    } else {
                        BorderType::Rounded
                    })
                    .title(Span::styled(" First Name* ", required_style))
                    .border_style(Style::default().fg(if self.focus_index == 0 {
                        Color::Cyan
                    } else {
                        Color::White
                    }))
                    .style(Style::default().bg(Color::Black)),
            );
        frame.render_widget(first_name_input, left_layout[1]);

        // Last Name (required)
        let last_name_input = Paragraph::new(self.last_name.clone())
            .style(Style::default().bg(Color::Black))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(if self.focus_index == 1 {
                        BorderType::Thick
                    } else {
                        BorderType::Rounded
                    })
                    .title(Span::styled(" Last Name* ", required_style))
                    .border_style(Style::default().fg(if self.focus_index == 1 {
                        Color::Cyan
                    } else {
                        Color::White
                    }))
                    .style(Style::default().bg(Color::Black)),
            );
        frame.render_widget(last_name_input, left_layout[2]);

        // Date of Birth (required)
        let dob_input = Paragraph::new(self.dob.clone())
            .style(Style::default().bg(Color::Black))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(if self.focus_index == 2 {
                        BorderType::Thick
                    } else {
                        BorderType::Rounded
                    })
                    .title(Span::styled(" Date of Birth* ", required_style))
                    .title_alignment(Alignment::Left)
                    .border_style(Style::default().fg(if self.focus_index == 2 {
                        Color::Cyan
                    } else {
                        Color::White
                    }))
                    .style(Style::default().bg(Color::Black)),
            );
        frame.render_widget(dob_input, left_layout[3]);

        // Gender as an input field
        let gender_text = match self.gender {
            Gender::Male => "Male",
            Gender::Female => "Female",
            Gender::Other => "Other",
        };

        let gender_input = Paragraph::new(gender_text)
            .style(Style::default().bg(Color::Black))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(if self.focus_index == 3 {
                        BorderType::Thick
                    } else {
                        BorderType::Rounded
                    })
                    .title(Span::styled(" Gender* ", required_style))
                    .border_style(Style::default().fg(if self.focus_index == 3 {
                        Color::Cyan
                    } else {
                        Color::White
                    }))
                    .style(Style::default().bg(Color::Black)),
            );
        frame.render_widget(gender_input, left_layout[4]);

        // Address (required)
        let address_input = Paragraph::new(self.address.clone())
            .style(Style::default().bg(Color::Black))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(if self.focus_index == 4 {
                        BorderType::Thick
                    } else {
                        BorderType::Rounded
                    })
                    .title(Span::styled(" Address* ", required_style))
                    .border_style(Style::default().fg(if self.focus_index == 4 {
                        Color::Cyan
                    } else {
                        Color::White
                    }))
                    .style(Style::default().bg(Color::Black)),
            );
        frame.render_widget(address_input, left_layout[5]);

        // Phone (required)
        let phone_input = Paragraph::new(self.phone.clone())
            .style(Style::default().bg(Color::Black))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(if self.focus_index == 5 {
                        BorderType::Thick
                    } else {
                        BorderType::Rounded
                    })
                    .title(Span::styled(" Phone* ", required_style))
                    .border_style(Style::default().fg(if self.focus_index == 5 {
                        Color::Cyan
                    } else {
                        Color::White
                    }))
                    .style(Style::default().bg(Color::Black)),
            );
        frame.render_widget(phone_input, left_layout[6]);

        // Secondary Info Fields (Right Column)
        // Email (optional)
        let email_input = Paragraph::new(self.email.clone().unwrap_or_default())
            .style(Style::default().bg(Color::Black))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(if self.focus_index == 6 {
                        BorderType::Thick
                    } else {
                        BorderType::Rounded
                    })
                    .title(" Email (optional) ")
                    .border_style(Style::default().fg(if self.focus_index == 6 {
                        Color::Cyan
                    } else {
                        Color::White
                    }))
                    .style(Style::default().bg(Color::Black)),
            );
        frame.render_widget(email_input, right_layout[1]);

        // Medical History (optional) - multi-line
        let history_input = Paragraph::new(self.medical_history.clone().unwrap_or_default())
            .style(Style::default().bg(Color::Black))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(if self.focus_index == 7 {
                        BorderType::Thick
                    } else {
                        BorderType::Rounded
                    })
                    .title(" Medical History (optional) ")
                    .border_style(Style::default().fg(if self.focus_index == 7 {
                        Color::Cyan
                    } else {
                        Color::White
                    }))
                    .style(Style::default().bg(Color::Black)),
            )
            .wrap(Wrap { trim: true });
        frame.render_widget(history_input, right_layout[2]);

        // Allergies (optional) - multi-line
        let allergies_input = Paragraph::new(self.allergies.clone().unwrap_or_default())
            .style(Style::default().bg(Color::Black))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(if self.focus_index == 8 {
                        BorderType::Thick
                    } else {
                        BorderType::Rounded
                    })
                    .title(" Allergies (optional) ")
                    .border_style(Style::default().fg(if self.focus_index == 8 {
                        Color::Cyan
                    } else {
                        Color::White
                    }))
                    .style(Style::default().bg(Color::Black)),
            )
            .wrap(Wrap { trim: true });
        frame.render_widget(allergies_input, right_layout[3]);

        // Medications (optional) - multi-line
        let medications_input = Paragraph::new(self.medications.clone().unwrap_or_default())
            .style(Style::default().bg(Color::Black))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(if self.focus_index == 9 {
                        BorderType::Thick
                    } else {
                        BorderType::Rounded
                    })
                    .title(" Medications (optional) ")
                    .border_style(Style::default().fg(if self.focus_index == 9 {
                        Color::Cyan
                    } else {
                        Color::White
                    }))
                    .style(Style::default().bg(Color::Black)),
            )
            .wrap(Wrap { trim: true });
        frame.render_widget(medications_input, right_layout[4]);

        // Error message area - right after the input fields
        let status_message = if let Some(success) = &self.success_message {
            Paragraph::new(format!("✓ {}", success))
                .style(
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD)
                        .bg(Color::Black),
                )
                .alignment(Alignment::Center)
        } else if let Some(error) = &self.error_message {
            Paragraph::new(format!("⚠️ {}", error))
                .style(
                    Style::default()
                        .fg(Color::Red)
                        .add_modifier(Modifier::BOLD)
                        .bg(Color::Black),
                )
                .alignment(Alignment::Center)
        } else {
            Paragraph::new("").style(Style::default().bg(Color::Black))
        };
        frame.render_widget(status_message, main_layout[3]);

        // CHANGED: Footer with buttons in vertical layout and help text
        let footer_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Submit button
                Constraint::Length(2), // Back button
                Constraint::Min(2),    // Help text
            ])
            .split(main_layout[5]);

        // Submit button with active state
        let submit_text = if self.focus_index == INPUT_FIELDS {
            "► Submit ◄"
        } else {
            "  Submit  "
        };

        let submit_button = Paragraph::new(submit_text)
            .style(
                Style::default()
                    .fg(if self.focus_index == INPUT_FIELDS {
                        Color::Green
                    } else {
                        Color::White
                    })
                    .add_modifier(if self.focus_index == INPUT_FIELDS {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    })
                    .bg(Color::Black),
            )
            .alignment(Alignment::Center);
        frame.render_widget(submit_button, footer_layout[0]);

        // Back button with active state
        let back_text = if self.focus_index == INPUT_FIELDS + 1 {
            "► Back ◄"
        } else {
            "  Back  "
        };

        let back_button = Paragraph::new(back_text)
            .style(
                Style::default()
                    .fg(if self.focus_index == INPUT_FIELDS + 1 {
                        Color::Cyan
                    } else {
                        Color::Cyan
                    })
                    .add_modifier(if self.focus_index == INPUT_FIELDS + 1 {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    })
                    .bg(Color::Black),
            )
            .alignment(Alignment::Center);
        frame.render_widget(back_button, footer_layout[1]);

        // Help text
        let help_text = Paragraph::new("Tab: Switch Focus | Arrow Keys: Switch Fields | Enter: Submit | Esc: Back\nFor Gender: Type 'M' for Male, 'F' for Female, 'O' for Others")
            .style(Style::default().fg(Color::DarkGray).bg(Color::Black))
            .alignment(Alignment::Center);
        frame.render_widget(help_text, footer_layout[2]);
    }
}
