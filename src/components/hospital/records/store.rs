//! `StoreRecord` - Component for adding new medical records to the hospital application.
//!
//! This module provides the `StoreRecord` struct, a component responsible for collecting patient information and storing new medical records in the database. It encapsulates the form fields, validation logic, and database interaction required to create new medical records, and exposes the `StoreRecord` struct.
//! The primary type exposed is `StoreRecord`, which implements the `Component` trait.

use crate::app::SelectedApp;
use crate::components::Component;
use crate::db;
use crate::models::{MedicalRecord, Patient};
use crate::tui::Frame;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{prelude::*, widgets::*};
use std::time::{Duration, Instant};

/// Component to add a new medical record.
///
/// `StoreRecord` is a UI component used within the hospital application to capture and store information for new medical records. It presents a form with fields for patient ID, doctor's notes, optional nurse's notes, diagnosis, and an optional prescription. Upon submission, the component validates the input, checks for patient existence, and saves the new medical record to the database.
/// This component uses the `Component` trait to handle user input and render its UI. It manages state such as input field values, focus, error messages, and success messages.
///
/// # Fields
///
/// *   `patient_id_input`:  The text input for the patient's ID.
/// *   `doctor_notes`: The text input for the doctor's notes.
/// *   `nurse_notes`: The optional text input for the nurse's notes.
/// *   `diagnosis`: The text input for the diagnosis.
/// *   `prescription`: The optional text input for the prescription.
/// *   `focus_index`: The index of the currently focused input field.
/// *   `error_message`: An optional string containing an error message to display to the user.
/// *   `error_timer`:  An optional `Instant` used to time the display of error messages.
/// *   `success_message`:  An optional string containing a success message to display to the user.
/// *   `success_timer`: An optional `Instant` used to time the display of success messages.
/// *   `all_patients`: A vector containing all patients for patient ID validation.
pub struct StoreRecord {
    patient_id_input: String,        // Input for patient ID
    doctor_notes: String,            // Doctor's notes
    nurse_notes: Option<String>,     // Optional nurse's notes
    diagnosis: String,               // Diagnosis
    prescription: Option<String>,    // Optional prescription
    focus_index: usize,              // Track which input field has focus
    error_message: Option<String>,   // Error message, if any
    error_timer: Option<Instant>,    // Timer for error message display
    success_message: Option<String>, // Success message, if any
    success_timer: Option<Instant>,  // Timer for success message display
    all_patients: Vec<Patient>,      // List of all patients for ID validation
}

const INPUT_FIELDS: usize = 5; // Number of input fields + "Back"

impl Default for StoreRecord {
    /// Creates a default `StoreRecord` component with initialized fields.
    ///
    /// This function is used when a new instance of `StoreRecord` is created without specifying initial values. It sets default values for all input fields, initializes the focus index to the first input, and clears any existing error or success messages. The patient list is initialized as an empty vector.
    ///
    /// # Returns
    ///
    /// *   `Self`: A new `StoreRecord` instance with default values.
    fn default() -> Self {
        StoreRecord {
            patient_id_input: String::new(),
            doctor_notes: String::new(),
            nurse_notes: None,
            diagnosis: String::new(),
            prescription: None,
            focus_index: 0,
            error_message: None,
            error_timer: None,
            success_message: None,
            success_timer: None,
            all_patients: Vec::new(), // Initialize
        }
    }
}

impl StoreRecord {
    /// Creates a new `StoreRecord` component.
    ///
    /// This function constructs a new `StoreRecord` component using the `default()` method. This ensures that the component starts with clean, initialized state.
    ///
    /// # Returns
    ///
    /// *   `Self`: A new `StoreRecord` instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Loads all patients from the database.
    ///
    /// This function retrieves all patient records from the database and stores them in the `all_patients` field. This data is used to validate the patient ID entered by the user when creating a new medical record.
    ///
    /// # Errors
    ///
    /// This function returns an `anyhow::Result` which will contain an error if the database query fails.
    ///
    /// # Side effects
    ///
    /// *   Updates the `all_patients` field with the list of patients fetched from the database.
    pub fn load_patients(&mut self) -> Result<()> {
        self.all_patients = db::get_all_patients()?;
        Ok(())
    }

    /// Clears the current error message.
    ///
    /// This function removes the current error message and resets the error timer. This is typically called after an error has been handled or when transitioning to a new state.
    ///
    /// # Side effects
    ///
    /// *   Sets `error_message` to `None`.
    /// *   Sets `error_timer` to `None`.
    fn clear_error(&mut self) {
        self.error_message = None;
        self.error_timer = None;
    }

    /// Sets an error message.
    ///
    /// This function sets an error message to be displayed to the user and starts a timer to automatically clear the message after a short duration.  It's used to provide feedback to the user when an error occurs during input validation or database operations.
    ///
    /// # Parameters
    ///
    /// *   `message`:  A `String` containing the error message to display.
    ///
    /// # Side effects
    ///
    /// *   Sets `error_message` to `Some(message)`.
    /// *   Sets `error_timer` to `Some(Instant::now())`.
    fn set_error(&mut self, message: String) {
        self.error_message = Some(message);
        self.error_timer = Some(Instant::now());
    }

    /// Clears the success message.
    ///
    /// This function clears the success message and resets the associated timer. It is used to remove the success message after its display duration has elapsed.
    ///
    /// # Side effects
    ///
    /// *   Sets `success_message` to `None`.
    /// *   Sets `success_timer` to `None`.
    fn clear_success(&mut self) {
        self.success_message = None;
        self.success_timer = None;
    }

    /// Checks and clears the error message if its timeout has expired.
    ///
    /// This function checks if the error message has been displayed for longer than 5 seconds. If the timeout has expired, it clears the error message using the `clear_error()` method.
    /// This ensures that error messages are displayed for a reasonable duration and do not persist indefinitely.
    ///
    /// # Side effects
    ///
    /// *   Calls `clear_error()` if the error message has timed out.
    pub fn check_error_timeout(&mut self) {
        if let Some(timer) = self.error_timer {
            if timer.elapsed() > Duration::from_secs(5) {
                self.clear_error();
            }
        }
    }

    /// Checks and clears the success message if its timeout has expired.
    ///
    /// This function checks if the success message has been displayed for longer than 5 seconds. If the timeout has expired, it clears the success message using the `clear_success()` method.
    /// This ensures that success messages are displayed for a reasonable duration and do not persist indefinitely.
    ///
    /// # Side effects
    ///
    /// *   Calls `clear_success()` if the success message has timed out.
    pub fn check_success_timeout(&mut self) {
        if let Some(timer) = self.success_timer {
            if timer.elapsed() > Duration::from_secs(5) {
                self.clear_success();
            }
        }
    }

    /// Handles user input events.
    ///
    /// This function processes user input events, such as key presses, to update the component's state. It handles input for the text fields, navigation between fields, and the submission or cancellation of the form.
    ///
    /// # Parameters
    ///
    /// *   `key`:  A `KeyEvent` representing the user's key press.
    ///
    /// # Returns
    ///
    /// *   `Result<Option<SelectedApp>>`: Returns `Ok(Some(SelectedApp::None))` to indicate that the user wants to return to the previous screen on pressing Escape, or `Ok(None)` if the input was handled within the component and no navigation is required.
    ///
    /// # Errors
    ///
    /// This function does not directly return errors, but it calls other functions that might return errors (e.g., database interaction).
    ///
    /// # Side effects
    ///
    /// *   Modifies the internal state of the component based on user input, including updating the values of input fields, changing the focus, and displaying or clearing error and success messages.
    /// *   May call the `db::create_medical_record` function to save a new record to the database, which can result in a database operation.
    pub fn handle_input(&mut self, key: KeyEvent) -> Result<Option<SelectedApp>> {
        self.check_error_timeout();
        self.check_success_timeout();

        match key.code {
            KeyCode::Char(c) => match self.focus_index {
                0 => self.patient_id_input.push(c),
                1 => self.doctor_notes.push(c),
                2 => {
                    if let Some(ref mut notes) = self.nurse_notes {
                        notes.push(c);
                    } else {
                        self.nurse_notes = Some(c.to_string());
                    }
                }
                3 => self.diagnosis.push(c),
                4 => {
                    if let Some(ref mut prescription) = self.prescription {
                        prescription.push(c);
                    } else {
                        self.prescription = Some(c.to_string());
                    }
                }
                _ => {}
            },
            KeyCode::Backspace => match self.focus_index {
                0 => {
                    self.patient_id_input.pop();
                }
                1 => {
                    self.doctor_notes.pop();
                }
                2 => {
                    if let Some(notes) = self.nurse_notes.as_mut() {
                        notes.pop();
                    }
                }
                3 => {
                    self.diagnosis.pop();
                }
                4 => {
                    if let Some(prescription) = self.prescription.as_mut() {
                        prescription.pop();
                    }
                }
                _ => {}
            },
            KeyCode::Tab => {
                if self.focus_index <= 4 {
                    self.focus_index = INPUT_FIELDS; // Jump to submit
                } else if self.focus_index == INPUT_FIELDS {
                    self.focus_index = INPUT_FIELDS + 1; // Jump to Back
                } else {
                    self.focus_index = 0;
                }
            }
            KeyCode::Down => {
                self.focus_index = (self.focus_index + 1) % (INPUT_FIELDS + 2);
            }
            KeyCode::Up => {
                self.focus_index = (self.focus_index + INPUT_FIELDS + 1) % (INPUT_FIELDS + 2);
            }
            KeyCode::Enter => {
                if self.focus_index == INPUT_FIELDS + 1 {
                    // "Back" button
                    return Ok(Some(SelectedApp::None));
                } else if self.focus_index == INPUT_FIELDS {
                    // "Submit" button
                    if self.patient_id_input.is_empty() {
                        self.set_error("Patient ID cannot be empty".to_string());
                        return Ok(None);
                    }
                    if self.doctor_notes.is_empty() {
                        self.set_error("Doctor's Notes cannot be empty".to_string());
                        return Ok(None);
                    }
                    if self.diagnosis.is_empty() {
                        self.set_error("Diagnosis cannot be empty".to_string());
                        return Ok(None);
                    }

                    // Validate Patient ID (check if it exists)
                    let patient_id = match self.patient_id_input.parse::<i64>() {
                        Ok(id) => id,
                        Err(_) => {
                            self.set_error("Invalid Patient ID format.".to_string());
                            return Ok(None);
                        }
                    };

                    if !self.all_patients.iter().any(|p| p.id == patient_id) {
                        self.set_error("Patient with given ID does not exist.".to_string());
                        return Ok(None);
                    }

                    let new_record = MedicalRecord {
                        id: 0, // Database will assign
                        patient_id,
                        doctor_notes: self.doctor_notes.clone(),
                        nurse_notes: self.nurse_notes.clone(),
                        diagnosis: self.diagnosis.clone(),
                        prescription: self.prescription.clone(),
                    };

                    match db::create_medical_record(&new_record) {
                        Ok(_) => {
                            self.success_message =
                                Some("Medical record added successfully!".to_string());
                            self.success_timer = Some(Instant::now());

                            // Clear form
                            self.patient_id_input.clear();
                            self.doctor_notes.clear();
                            self.nurse_notes = None;
                            self.diagnosis.clear();
                            self.prescription = None;
                            self.focus_index = 0;
                            self.clear_error();
                        }
                        Err(e) => {
                            self.set_error(format!("Database error: {}", e));
                        }
                    }
                }
            }
            KeyCode::Esc => return Ok(Some(SelectedApp::None)),

            _ => {}
        }
        Ok(None)
    }
}

impl Component for StoreRecord {
    /// Handles input events.
    ///
    /// This function passes the input event to the component's `handle_input` method for processing.
    ///
    /// # Parameters
    ///
    /// *   `event`: A `KeyEvent` representing the user's key press.
    ///
    /// # Returns
    ///
    /// *   `Result<Option<SelectedApp>>`: Returns the result of the `handle_input` method.
    fn handle_input(&mut self, event: KeyEvent) -> Result<Option<SelectedApp>> {
        self.handle_input(event)
    }

    /// Renders the `StoreRecord` component to the terminal.
    ///
    /// This function draws the user interface of the `StoreRecord` component using the `ratatui` library. It includes the form elements, input fields, labels, and any error or success messages.
    ///
    /// # Parameters
    ///
    /// *   `frame`: A mutable reference to a `Frame` object, used for rendering the UI elements.
    fn render(&self, frame: &mut Frame) {
        let area = frame.area();
        frame.render_widget(
            Block::default().style(Style::default().bg(Color::Rgb(16, 16, 28))),
            area,
        );

        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(15),
                Constraint::Length(0),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(6),
            ])
            .margin(1)
            .split(frame.area());

        // Header with title
        let header = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(Color::Rgb(75, 75, 120)))
            .style(Style::default().bg(Color::Rgb(16, 16, 28)));
        frame.render_widget(header, main_layout[0]);

        let title = Paragraph::new("📝 ADD MEDICAL RECORD")
            .style(
                Style::default()
                    .fg(Color::Rgb(230, 230, 250))
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Rgb(16, 16, 28)),
            )
            .alignment(Alignment::Center);
        frame.render_widget(title, main_layout[0]);

        let body_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Rgb(75, 75, 120)))
            .style(Style::default().bg(Color::Rgb(22, 22, 35)));

        frame.render_widget(body_block.clone(), main_layout[1]);
        let body_inner = body_block.inner(main_layout[1]);

        let body_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Patient ID
                Constraint::Length(3), // Doctor's Notes
                Constraint::Length(3), // Nurse's Notes
                Constraint::Length(3), // Diagnosis
                Constraint::Length(3), // Prescription
            ])
            .margin(1)
            .split(body_inner);

        let required_style = Style::default().fg(Color::Rgb(230, 230, 250));

        // Patient ID
        let patient_id_input = Paragraph::new(self.patient_id_input.clone())
            .style(
                Style::default()
                    .fg(Color::Rgb(220, 220, 240))
                    .bg(Color::Rgb(26, 26, 36)),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(Span::styled(" Patient ID* ", required_style))
                    .border_style(if self.focus_index == 0 {
                        Style::default().fg(Color::Rgb(250, 250, 110))
                    } else {
                        Style::default().fg(Color::Rgb(140, 140, 200))
                    })
                    .style(Style::default().bg(Color::Rgb(26, 26, 36))),
            );
        frame.render_widget(patient_id_input, body_layout[0]);

        // Doctor's Notes
        let doctor_notes_input = Paragraph::new(self.doctor_notes.clone())
            .style(
                Style::default()
                    .fg(Color::Rgb(220, 220, 240))
                    .bg(Color::Rgb(26, 26, 36)),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(Span::styled(" Doctor's Notes* ", required_style))
                    .border_style(if self.focus_index == 1 {
                        Style::default().fg(Color::Rgb(250, 250, 110))
                    } else {
                        Style::default().fg(Color::Rgb(140, 140, 200))
                    })
                    .style(Style::default().bg(Color::Rgb(26, 26, 36))),
            );
        frame.render_widget(doctor_notes_input, body_layout[1]);

        // Nurse's Notes (Optional)
        let nurse_notes_input = Paragraph::new(self.nurse_notes.clone().unwrap_or_default())
            .style(
                Style::default()
                    .fg(Color::Rgb(220, 220, 240))
                    .bg(Color::Rgb(26, 26, 36)),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(Span::styled(" Nurse's Notes ", required_style))
                    .border_style(if self.focus_index == 2 {
                        Style::default().fg(Color::Rgb(250, 250, 110))
                    } else {
                        Style::default().fg(Color::Rgb(140, 140, 200))
                    })
                    .style(Style::default().bg(Color::Rgb(26, 26, 36))),
            );
        frame.render_widget(nurse_notes_input, body_layout[2]);

        // Diagnosis
        let diagnosis_input = Paragraph::new(self.diagnosis.clone())
            .style(
                Style::default()
                    .fg(Color::Rgb(220, 220, 240))
                    .bg(Color::Rgb(26, 26, 36)),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(Span::styled(" Diagnosis* ", required_style))
                    .border_style(if self.focus_index == 3 {
                        Style::default().fg(Color::Rgb(250, 250, 110))
                    } else {
                        Style::default().fg(Color::Rgb(140, 140, 200))
                    })
                    .style(Style::default().bg(Color::Rgb(26, 26, 36))),
            );
        frame.render_widget(diagnosis_input, body_layout[3]);

        // Prescription (Optional)
        let prescription_input = Paragraph::new(self.prescription.clone().unwrap_or_default())
            .style(
                Style::default()
                    .fg(Color::Rgb(220, 220, 240))
                    .bg(Color::Rgb(26, 26, 36)),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(Span::styled(" Prescription ", required_style))
                    .border_style(if self.focus_index == 4 {
                        Style::default().fg(Color::Rgb(250, 250, 110))
                    } else {
                        Style::default().fg(Color::Rgb(140, 140, 200))
                    })
                    .style(Style::default().bg(Color::Rgb(26, 26, 36))),
            );
        frame.render_widget(prescription_input, body_layout[4]);

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
        let submit_text = if self.focus_index == INPUT_FIELDS {
            "► Submit ◄"
        } else {
            "  Submit  "
        };

        let submit_style = if self.focus_index == INPUT_FIELDS {
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
        let back_text = if self.focus_index == INPUT_FIELDS + 1 {
            "► Back ◄"
        } else {
            "  Back  "
        };

        let back_style = if self.focus_index == INPUT_FIELDS + 1 {
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
        let help_text = "Tab: Switch Focus | Arrow Keys: Switch Fields | Enter: Submit | Esc: Back";
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
