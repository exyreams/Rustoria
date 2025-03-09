//! `StoreRecord` - Component for adding new medical records, with separate patient selection and record entry screens.
//!
//! This module provides the `StoreRecord` struct, a UI component that allows users to add new medical records.
//! It displays a searchable, selectable list of patients.  Once a patient is selected,
//! form fields appear for entering the medical record details (doctor's notes, nurse's notes,
//! diagnosis, and prescription).  This replaces the direct patient ID input with a more
//! user-friendly selection process.

use crate::app::SelectedApp;
use crate::components::Component;
use crate::db;
use crate::models::{MedicalRecord, Patient};
use crate::tui::Frame;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{prelude::*, widgets::*};
use std::time::{Duration, Instant};

/// Represents the different states of the StoreRecord component.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreRecordState {
    SelectingPatient,
    EnteringDetails,
}

const PATIENT_SELECTION: usize = 0; // Focus index for patient selection
const INPUT_FIELDS: usize = 4; // Number of input fields (doctor_notes, nurse_notes, diagnosis, prescription)
const SUBMIT_BUTTON: usize = 4; // Focus index for Submit button.
const BACK_BUTTON: usize = 5; // Focus index for the "Back" button

/// Component to add a new medical record, with integrated patient selection.
pub struct StoreRecord {
    all_patients: Vec<Patient>,        // All patients (for selection)
    filtered_patients: Vec<Patient>,   // Filtered patients (for selection)
    selected_patient: Option<Patient>, // The currently selected patient
    search_input: String,              // Search input for patient filtering
    is_searching: bool,                // Flag to indicate search mode
    table_state: TableState,           // State for the patient selection table
    doctor_notes: String,              // Doctor's notes
    nurse_notes: Option<String>,       // Optional nurse's notes
    diagnosis: String,                 // Diagnosis
    prescription: Option<String>,      // Optional prescription
    focus_index: usize,                // Track which input field has focus
    state: StoreRecordState,           // The current state of the component
    error_message: Option<String>,     // Error message, if any
    error_timer: Option<Instant>,      // Timer for error message display
    success_message: Option<String>,   // Success message, if any
    success_timer: Option<Instant>,    // Timer for success message display
}

impl Default for StoreRecord {
    fn default() -> Self {
        let mut table_state = TableState::default();
        table_state.select(Some(0)); // Select first by default
        StoreRecord {
            all_patients: Vec::new(),
            filtered_patients: Vec::new(),
            selected_patient: None,
            search_input: String::new(),
            is_searching: false,
            table_state,
            doctor_notes: String::new(),
            nurse_notes: None,
            diagnosis: String::new(),
            prescription: None,
            focus_index: PATIENT_SELECTION, // Start with patient selection
            state: StoreRecordState::SelectingPatient, // NEW: Start in selection state.
            error_message: None,
            error_timer: None,
            success_message: None,
            success_timer: None,
        }
    }
}

impl StoreRecord {
    /// Creates a new `StoreRecord` component.
    pub fn new() -> Self {
        Self::default()
    }

    /// Loads all patients from the database for the selection table.
    pub fn load_patients(&mut self) -> Result<()> {
        self.all_patients = db::get_all_patients()?;
        self.filter_patients(); // Initial filtering
        Ok(())
    }

    /// Filters the patient list based on the `search_input`.
    fn filter_patients(&mut self) {
        if self.search_input.is_empty() {
            self.filtered_patients = self.all_patients.clone();
        } else {
            let search_term = self.search_input.to_lowercase();
            self.filtered_patients = self
                .all_patients
                .iter()
                .filter(|p| {
                    p.first_name.to_lowercase().contains(&search_term)
                        || p.last_name.to_lowercase().contains(&search_term)
                        || p.id.to_string().contains(&search_term)
                })
                .cloned()
                .collect();
        }

        // Reset selection in the table
        if !self.filtered_patients.is_empty() {
            self.table_state.select(Some(0));
        } else {
            self.table_state.select(None); // clear, if no patients
        }
    }

    /// Selects the next patient in the table.
    fn select_next_patient(&mut self) {
        if self.filtered_patients.is_empty() {
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => {
                if i >= self.filtered_patients.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    /// Selects the previous patient in the table.
    fn select_previous_patient(&mut self) {
        if self.filtered_patients.is_empty() {
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.filtered_patients.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    /// Clears the current error message.
    fn clear_error(&mut self) {
        self.error_message = None;
        self.error_timer = None;
    }

    /// Sets an error message to display to the user.
    fn set_error(&mut self, message: String) {
        self.error_message = Some(message);
        self.error_timer = Some(Instant::now());
    }

    /// Clears the success message.
    fn clear_success(&mut self) {
        self.success_message = None;
        self.success_timer = None;
    }

    /// Checks and clears the error message if its timeout has expired.
    pub fn check_error_timeout(&mut self) {
        if let Some(timer) = self.error_timer {
            if timer.elapsed() > Duration::from_secs(5) {
                self.clear_error();
            }
        }
    }

    /// Checks and clears the success message if its timeout has expired.
    pub fn check_success_timeout(&mut self) {
        if let Some(timer) = self.success_timer {
            if timer.elapsed() > Duration::from_secs(5) {
                self.clear_success();
            }
        }
    }

    /// Handles user input events, including patient selection and form input.
    pub fn handle_input(&mut self, key: KeyEvent) -> Result<Option<SelectedApp>> {
        self.check_error_timeout();
        self.check_success_timeout();

        match self.state {
            StoreRecordState::SelectingPatient => {
                match key.code {
                    // Search mode handling
                    KeyCode::Char(c) if self.is_searching => {
                        self.search_input.push(c);
                        self.filter_patients();
                        self.clear_error();
                    }
                    KeyCode::Backspace if self.is_searching => {
                        self.search_input.pop();
                        self.filter_patients();
                        self.clear_error();
                    }
                    KeyCode::Down if self.is_searching && !self.filtered_patients.is_empty() => {
                        self.is_searching = false;
                        self.table_state.select(Some(0)); // Move to table
                    }
                    KeyCode::Esc if self.is_searching => {
                        self.is_searching = false; // Exit search mode
                        self.search_input.clear();
                        self.filter_patients();
                    }
                    KeyCode::Char('/') | KeyCode::Char('s') | KeyCode::Char('S')
                        if !self.is_searching =>
                    {
                        self.is_searching = true;
                    }

                    KeyCode::Up => self.select_previous_patient(),
                    KeyCode::Down => self.select_next_patient(),
                    KeyCode::Tab => {
                        // Only Back button in this state
                        self.focus_index = if self.focus_index == PATIENT_SELECTION {
                            BACK_BUTTON
                        } else {
                            PATIENT_SELECTION
                        };
                    }
                    // marking, selecting with space key.
                    KeyCode::Char(' ') => {
                        if let Some(selected) = self.table_state.selected() {
                            if selected < self.filtered_patients.len() {
                                // Check if already selected
                                if let Some(patient) = &self.selected_patient {
                                    if patient.id == self.filtered_patients[selected].id {
                                        self.selected_patient = None; // Deselect
                                    } else {
                                        self.selected_patient =
                                            Some(self.filtered_patients[selected].clone());
                                        // Select Different
                                    }
                                } else {
                                    self.selected_patient =
                                        Some(self.filtered_patients[selected].clone());
                                    // Select
                                }
                            }
                        }
                    }

                    KeyCode::Enter => {
                        if self.focus_index == BACK_BUTTON {
                            return Ok(Some(SelectedApp::None));
                        }
                        if self.is_searching {
                            if !self.filtered_patients.is_empty() {
                                self.is_searching = false;
                                self.table_state.select(Some(0));
                            }
                        } else {
                            // Select Patient and switch to data entry state
                            if let Some(selected) = self.table_state.selected() {
                                if selected < self.filtered_patients.len() {
                                    // Check the Selection
                                    if let Some(patient) = &self.selected_patient {
                                        if patient.id == self.filtered_patients[selected].id {
                                            self.state = StoreRecordState::EnteringDetails; // Switch state
                                            self.focus_index = 0; // Focus on first input field
                                            return Ok(None);
                                        } else {
                                            self.set_error(
                                                "Please Select Patient with Spacebar".to_string(),
                                            );
                                            return Ok(None);
                                        }
                                    } else {
                                        self.set_error(
                                            "Please Select Patient with Spacebar".to_string(),
                                        );
                                        return Ok(None);
                                    }
                                }
                            }
                        }
                    }
                    KeyCode::Esc => return Ok(Some(SelectedApp::None)), // Exit component
                    _ => {}
                }
                return Ok(None); // No navigation needed yet
            }

            StoreRecordState::EnteringDetails => {
                match key.code {
                    KeyCode::Char(c) => match self.focus_index {
                        0 => self.doctor_notes.push(c),
                        1 => {
                            if let Some(ref mut notes) = self.nurse_notes {
                                notes.push(c);
                            } else {
                                self.nurse_notes = Some(c.to_string());
                            }
                        }
                        2 => self.diagnosis.push(c),
                        3 => {
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
                            self.doctor_notes.pop();
                        }
                        1 => {
                            if let Some(notes) = self.nurse_notes.as_mut() {
                                notes.pop();
                            }
                        }
                        2 => {
                            self.diagnosis.pop();
                        }
                        3 => {
                            if let Some(prescription) = self.prescription.as_mut() {
                                prescription.pop();
                            }
                        }
                        _ => {}
                    },
                    KeyCode::Tab => {
                        // Cycle through: inputs (0-3) -> Add Details (4) -> Back (5) -> inputs (0)
                        self.focus_index = (self.focus_index + 1) % (INPUT_FIELDS + 2);
                    }
                    KeyCode::Down => {
                        self.focus_index = (self.focus_index + 1) % (INPUT_FIELDS + 2);
                    }
                    KeyCode::Up => {
                        self.focus_index =
                            (self.focus_index + (INPUT_FIELDS + 1)) % (INPUT_FIELDS + 2);
                    }
                    KeyCode::Enter if self.focus_index == BACK_BUTTON => {
                        // Back button.
                        self.state = StoreRecordState::SelectingPatient; // Go back to patient selection
                        self.focus_index = PATIENT_SELECTION; // Reset to patient selection.
                        return Ok(None);
                    }
                    KeyCode::Enter if self.focus_index == SUBMIT_BUTTON => {
                        // "Submit" on last input field
                        if self.doctor_notes.is_empty() {
                            self.set_error("Doctor's Notes cannot be empty".to_string());
                            return Ok(None);
                        }
                        if self.diagnosis.is_empty() {
                            self.set_error("Diagnosis cannot be empty".to_string());
                            return Ok(None);
                        }
                        //validation for patient selection
                        if let Some(patient) = &self.selected_patient {
                            let new_record = MedicalRecord {
                                id: 0,
                                patient_id: patient.id, // Use the selected patient's ID
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

                                    self.doctor_notes.clear();
                                    self.nurse_notes = None;
                                    self.diagnosis.clear();
                                    self.prescription = None;
                                    self.state = StoreRecordState::SelectingPatient;
                                    self.focus_index = PATIENT_SELECTION; // Go back to selection,
                                    self.selected_patient = None; // Clear the selection
                                    self.clear_error(); //clear any previous error
                                }
                                Err(e) => {
                                    self.set_error(format!("Database error: {}", e));
                                }
                            }
                        } else {
                            // This should not happen since you are now forcing user to
                            // select a patient, but it's good to keep the error check.
                            self.set_error("Please select a patient first.".to_string());
                            return Ok(None);
                        }
                    }
                    KeyCode::Enter => {}

                    KeyCode::Esc => {
                        // Go back to the patient selection
                        self.state = StoreRecordState::SelectingPatient;
                        self.focus_index = PATIENT_SELECTION; // Reset focus
                        return Ok(None);
                    }
                    _ => {}
                }
            }
        }
        Ok(None)
    }
}

impl Component for StoreRecord {
    /// Delegates input handling to the `handle_input` method.
    fn handle_input(&mut self, event: KeyEvent) -> Result<Option<SelectedApp>> {
        self.handle_input(event)
    }

    /// Main render function that delegates to the appropriate state-specific render method
    fn render(&self, frame: &mut Frame) {
        // Set background color
        frame.render_widget(
            Block::default().style(Style::default().bg(Color::Rgb(16, 16, 28))),
            frame.area(),
        );

        // Render based on the current state - completely separate layouts
        match self.state {
            StoreRecordState::SelectingPatient => {
                self.render_patient_selection_page(frame);
            }
            StoreRecordState::EnteringDetails => {
                self.render_record_details_page(frame);
            }
        }
    }
}

// Implementation of state-specific rendering methods
impl StoreRecord {
    /// Renders the complete patient selection page
    fn render_patient_selection_page(&self, frame: &mut Frame) {
        let area = frame.area();

        // Define the layout for the entire patient selection page
        // UPDATED: Reduced table height by adding spacing around the back button
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(7),    // Content area (search + table) - REDUCED by 3
                Constraint::Length(1), // Status message area
                Constraint::Length(1), // Back button
                Constraint::Length(1), // Space 1
                Constraint::Length(1), // Space 2
                Constraint::Length(1), // Help text
            ])
            .margin(1)
            .split(area);

        // Header with title
        let header = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(Color::Rgb(75, 75, 120)))
            .style(Style::default().bg(Color::Rgb(16, 16, 28)));
        frame.render_widget(header, layout[0]);

        let title = Paragraph::new("📝 SELECT PATIENT")
            .style(
                Style::default()
                    .fg(Color::Rgb(230, 230, 250))
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Rgb(16, 16, 28)),
            )
            .alignment(Alignment::Center);
        frame.render_widget(title, layout[0]);

        // Content: Search + Table
        self.render_patient_selection_content(frame, layout[1]);

        // Status message (error or success)
        self.render_status_message(frame, layout[2]);

        // Back button
        let back_text = if self.focus_index == BACK_BUTTON {
            "► Back ◄"
        } else {
            "  Back  "
        };
        let back_style = if self.focus_index == BACK_BUTTON {
            Style::default()
                .fg(Color::Rgb(129, 199, 245)) // Blue
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Rgb(180, 180, 200)) // Light gray
        };
        frame.render_widget(
            Paragraph::new(back_text)
                .style(back_style)
                .alignment(Alignment::Center),
            layout[3],
        );

        // Empty spaces (4 and 5)

        // Help text
        frame.render_widget(
            Paragraph::new(
                "/ or s: Search, ↑/↓: Navigate | Spacebar: Select | Enter: Confirm | Tab: Back | Esc: Exit"
            )
            .style(Style::default().fg(Color::Rgb(180, 180, 200)))
            .alignment(Alignment::Center),
            layout[6],
        );
    }

    /// Renders the search box and patient table
    fn render_patient_selection_content(&self, frame: &mut Frame, area: Rect) {
        // Layout for search box and table
        let content_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Search Input
                Constraint::Min(4),    // Patient Table
            ])
            .split(area);

        // Search box
        let search_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(Span::styled(
                " Search Patients ",
                Style::default()
                    .fg(Color::Rgb(230, 230, 250))
                    .add_modifier(Modifier::BOLD),
            ))
            .border_style(
                if self.is_searching && self.focus_index == PATIENT_SELECTION {
                    Style::default().fg(Color::Rgb(250, 250, 110))
                } else {
                    Style::default().fg(Color::Rgb(75, 75, 120))
                },
            )
            .style(Style::default().bg(Color::Rgb(22, 22, 35)));

        let search_paragraph = Paragraph::new(self.search_input.clone())
            .style(Style::default().fg(Color::Rgb(220, 220, 240)))
            .block(search_block);
        frame.render_widget(search_paragraph, content_layout[0]);

        // Patient selection table
        let table_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(if !self.search_input.is_empty() {
                format!(
                    " Select Patient ({} of {} matches) ",
                    self.filtered_patients.len(),
                    self.all_patients.len()
                )
            } else {
                format!(" Select Patient ({}) ", self.all_patients.len())
            })
            .title_style(
                Style::default()
                    .fg(Color::Rgb(230, 230, 250))
                    .add_modifier(Modifier::BOLD),
            )
            .border_style(
                if self.focus_index == PATIENT_SELECTION && !self.is_searching {
                    Style::default().fg(Color::Rgb(250, 250, 110)) // Yellow when active
                } else {
                    Style::default().fg(Color::Rgb(140, 140, 200))
                },
            )
            .style(Style::default().bg(Color::Rgb(26, 26, 36)));

        let selected_style = Style::default()
            .bg(Color::Rgb(45, 45, 60))
            .fg(Color::Rgb(250, 250, 250))
            .add_modifier(Modifier::BOLD);
        let normal_style = Style::default()
            .bg(Color::Rgb(26, 26, 36))
            .fg(Color::Rgb(220, 220, 240));

        // Create table rows
        let mut rows = Vec::new();
        for patient in &self.filtered_patients {
            let selected_indicator = if let Some(selected) = &self.selected_patient {
                if selected.id == patient.id {
                    "✓"
                } else {
                    ""
                }
            } else {
                ""
            };

            rows.push(Row::new(vec![
                Cell::from(selected_indicator.to_string()).style(normal_style),
                Cell::from(patient.id.to_string()).style(normal_style),
                Cell::from(patient.first_name.clone()).style(normal_style),
                Cell::from(patient.last_name.clone()).style(normal_style),
                Cell::from(patient.phone_number.clone()).style(normal_style),
            ]));
        }

        if self.filtered_patients.is_empty() {
            let message = if self.search_input.is_empty() {
                "No patients found in database"
            } else {
                "No patients match your search criteria"
            };

            rows.push(Row::new(vec![
                Cell::from(""),
                Cell::from(""),
                Cell::from(message).style(Style::default().fg(Color::Rgb(180, 180, 200))),
                Cell::from(""),
                Cell::from(""),
            ]));
        }

        let table = Table::new(
            rows,
            [
                Constraint::Length(3),  // Selected indicator
                Constraint::Length(8),  // ID
                Constraint::Length(15), // First Name
                Constraint::Length(15), // Last Name
                Constraint::Min(15),    // Phone Number
            ],
        )
        .header(
            Row::new(vec![
                Cell::from(""),
                Cell::from("ID").style(Style::default().add_modifier(Modifier::BOLD)),
                Cell::from("First Name").style(Style::default().add_modifier(Modifier::BOLD)),
                Cell::from("Last Name").style(Style::default().add_modifier(Modifier::BOLD)),
                Cell::from("Phone").style(Style::default().add_modifier(Modifier::BOLD)),
            ])
            .style(
                Style::default()
                    .bg(Color::Rgb(80, 60, 130))
                    .fg(Color::Rgb(180, 180, 250)),
            )
            .height(1),
        )
        .block(table_block)
        .row_highlight_style(selected_style)
        .highlight_symbol("► ");

        frame.render_stateful_widget(table, content_layout[1], &mut self.table_state.clone());
    }

    /// Renders the complete record details page
    fn render_record_details_page(&self, frame: &mut Frame) {
        let area = frame.area();

        // Define the layout for the entire record details page
        // UPDATED: Precise layout with exactly the spacing requested
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Header
                Constraint::Length(12), // Form fields (exactly 4 fields at 3 lines each)
                Constraint::Length(1),  // Space after forms
                Constraint::Length(1),  // Status message area
                Constraint::Length(1),  // Add Details button
                Constraint::Length(1),  // Space
                Constraint::Length(1),  // Back button
                Constraint::Length(1),  // Space 1
                Constraint::Length(1),  // Space 2
                Constraint::Length(1),  // Help text
            ])
            .margin(1)
            .split(area);

        // Header with title
        let header = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(Color::Rgb(75, 75, 120)))
            .style(Style::default().bg(Color::Rgb(16, 16, 28)));
        frame.render_widget(header, layout[0]);

        let title = Paragraph::new("📝 ADD RECORD DETAILS")
            .style(
                Style::default()
                    .fg(Color::Rgb(230, 230, 250))
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Rgb(16, 16, 28)),
            )
            .alignment(Alignment::Center);
        frame.render_widget(title, layout[0]);

        // Form fields
        self.render_record_form_fields(frame, layout[1]);

        // Space after forms (layout[2])

        // Status message (error or success)
        self.render_status_message(frame, layout[3]);

        // Add Details button
        let submit_text = if self.focus_index == SUBMIT_BUTTON {
            "► Add Details ◄"
        } else {
            "  Add Details  "
        };
        let submit_style = if self.focus_index == SUBMIT_BUTTON {
            Style::default()
                .fg(Color::Rgb(140, 219, 140)) // Green
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Rgb(180, 180, 200)) // Light gray
        };
        frame.render_widget(
            Paragraph::new(submit_text)
                .style(submit_style)
                .alignment(Alignment::Center),
            layout[4],
        );

        // Space after Add Details (layout[5])

        // Back button
        let back_text = if self.focus_index == BACK_BUTTON {
            "► Back ◄"
        } else {
            "  Back  "
        };
        let back_style = if self.focus_index == BACK_BUTTON {
            Style::default()
                .fg(Color::Rgb(129, 199, 245)) // Blue
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Rgb(180, 180, 200)) // Light gray
        };
        frame.render_widget(
            Paragraph::new(back_text)
                .style(back_style)
                .alignment(Alignment::Center),
            layout[6],
        );

        // Spaces after Back (layout[7] and layout[8])

        // Help text
        frame.render_widget(
            Paragraph::new("Tab: Switch Focus, ↑/↓: Navigate | Enter: Submit | Esc: Back")
                .style(Style::default().fg(Color::Rgb(180, 180, 200)))
                .alignment(Alignment::Center),
            layout[9],
        );
    }

    /// Renders the form fields for the record details
    fn render_record_form_fields(&self, frame: &mut Frame, area: Rect) {
        let form_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Doctor's Notes
                Constraint::Length(3), // Nurse's Notes
                Constraint::Length(3), // Diagnosis
                Constraint::Length(3), // Prescription
            ])
            .horizontal_margin(3) // Make forms wider by reducing horizontal margins
            .split(area);

        let required_style = Style::default().fg(Color::Rgb(230, 230, 250));

        // Doctor's Notes
        let doctor_notes_input = Paragraph::new(self.doctor_notes.clone())
            .style(if self.focus_index == 0 {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Rgb(220, 220, 240))
            })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(Span::styled(" Doctor's Notes* ", required_style))
                    .border_style(if self.focus_index == 0 {
                        Style::default().fg(Color::Rgb(250, 250, 110))
                    } else {
                        Style::default().fg(Color::Rgb(140, 140, 200))
                    })
                    .style(Style::default().bg(Color::Rgb(26, 26, 36))),
            );
        frame.render_widget(doctor_notes_input, form_layout[0]);

        // Nurse's Notes (Optional)
        let nurse_notes_input = Paragraph::new(self.nurse_notes.clone().unwrap_or_default())
            .style(if self.focus_index == 1 {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Rgb(220, 220, 240))
            })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(Span::styled(" Nurse's Notes ", required_style))
                    .border_style(if self.focus_index == 1 {
                        Style::default().fg(Color::Rgb(250, 250, 110))
                    } else {
                        Style::default().fg(Color::Rgb(140, 140, 200))
                    })
                    .style(Style::default().bg(Color::Rgb(26, 26, 36))),
            );
        frame.render_widget(nurse_notes_input, form_layout[1]);

        // Diagnosis
        let diagnosis_input = Paragraph::new(self.diagnosis.clone())
            .style(if self.focus_index == 2 {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Rgb(220, 220, 240))
            })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(Span::styled(" Diagnosis* ", required_style))
                    .border_style(if self.focus_index == 2 {
                        Style::default().fg(Color::Rgb(250, 250, 110))
                    } else {
                        Style::default().fg(Color::Rgb(140, 140, 200))
                    })
                    .style(Style::default().bg(Color::Rgb(26, 26, 36))),
            );
        frame.render_widget(diagnosis_input, form_layout[2]);

        // Prescription (Optional)
        let prescription_input = Paragraph::new(self.prescription.clone().unwrap_or_default())
            .style(if self.focus_index == 3 {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Rgb(220, 220, 240))
            })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(Span::styled(" Prescription ", required_style))
                    .border_style(if self.focus_index == 3 {
                        Style::default().fg(Color::Rgb(250, 250, 110))
                    } else {
                        Style::default().fg(Color::Rgb(140, 140, 200))
                    })
                    .style(Style::default().bg(Color::Rgb(26, 26, 36))),
            );
        frame.render_widget(prescription_input, form_layout[3]);
    }

    /// Renders the status message (error or success)
    fn render_status_message(&self, frame: &mut Frame, area: Rect) {
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
        frame.render_widget(status_message, area);
    }
}
