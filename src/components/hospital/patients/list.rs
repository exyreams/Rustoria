//! List Patient component for the Hospital application.

use crate::components::hospital::patients::PatientAction;
use crate::components::Component;
use crate::db;
use crate::models::Patient;
use crate::tui::Frame;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{prelude::*, widgets::*};

// Constants for focus indices
const PATIENT_LIST: usize = 0;
const BACK_BUTTON: usize = 1;

/// Component to display a list of patients.
pub struct ListPatients {
    patients: Vec<Patient>,
    state: TableState,
    error_message: Option<String>,
    show_details: bool,
    focus_index: usize,
}

impl ListPatients {
    /// Creates a new `ListPatients` component.
    pub fn new() -> Self {
        Self {
            patients: Vec::new(),
            state: TableState::default(),
            error_message: None,
            show_details: false,
            focus_index: PATIENT_LIST,
        }
    }

    /// Fetches the patient data from the database.
    pub fn fetch_patients(&mut self) -> Result<()> {
        match db::get_all_patients() {
            Ok(patients) => {
                self.patients = patients;
                if self.patients.is_empty() {
                    // Start with no selection if no patients exist
                    self.state.select(None);
                } else {
                    // Ensure selection is within bounds, default to first patient
                    let selection = self
                        .state
                        .selected()
                        .unwrap_or(0)
                        .min(self.patients.len() - 1);
                    self.state.select(Some(selection));
                }
                self.error_message = None; // Clear any previous error
                Ok(())
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to fetch patients: {}", e));
                //Don't return error to avoid stopping program, just display error
                Ok(())
            }
        }
    }

    /// Handles selection of a patient.
    fn select_next(&mut self) {
        if self.patients.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.patients.len() - 1 {
                    0 // Wrap around to the first patient
                } else {
                    i + 1
                }
            }
            None => 0, // Select the first patient if nothing is selected
        };
        self.state.select(Some(i));
    }

    /// Handles selection of the previous patient
    fn select_previous(&mut self) {
        if self.patients.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.patients.len() - 1 // Wrap around to the last patient
                } else {
                    i - 1
                }
            }
            None => 0, // Select the first patient if nothing is selected
        };
        self.state.select(Some(i));
    }

    /// Toggle patient details view
    fn toggle_details(&mut self) {
        if !self.patients.is_empty() && self.state.selected().is_some() {
            self.show_details = !self.show_details;
        }
    }

    /// Focus the next element
    fn focus_next(&mut self) {
        self.focus_index = (self.focus_index + 1) % 2;
    }

    /// Focus the previous element
    fn focus_previous(&mut self) {
        self.focus_index = if self.focus_index == 0 { 1 } else { 0 };
    }

    /// Handles input events for the component, specifically for patient selection.
    pub fn handle_input(&mut self, key: KeyEvent) -> Result<Option<PatientAction>> {
        match key.code {
            KeyCode::Tab => self.focus_next(),
            KeyCode::BackTab => self.focus_previous(),
            KeyCode::Down | KeyCode::Right => {
                if self.focus_index == PATIENT_LIST {
                    self.select_next();
                } else {
                    self.focus_next();
                }
            }
            KeyCode::Up | KeyCode::Left => {
                if self.focus_index == PATIENT_LIST {
                    self.select_previous();
                } else {
                    self.focus_previous();
                }
            }
            KeyCode::Enter => {
                if self.focus_index == BACK_BUTTON {
                    return Ok(Some(PatientAction::BackToHome));
                } else {
                    self.toggle_details();
                }
            }
            KeyCode::Char('b') | KeyCode::Char('B') => {
                return Ok(Some(PatientAction::BackToHome));
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                self.fetch_patients()?;
            }
            KeyCode::Esc => {
                if self.show_details {
                    self.show_details = false;
                } else {
                    return Ok(Some(PatientAction::BackToHome));
                }
            }
            _ => {}
        }
        Ok(None)
    }

    /// Get the currently selected patient, if any
    fn selected_patient(&self) -> Option<&Patient> {
        self.state.selected().and_then(|i| self.patients.get(i))
    }
}

impl Component for ListPatients {
    fn handle_input(&mut self, event: KeyEvent) -> Result<Option<crate::app::SelectedApp>> {
        match self.handle_input(event)? {
            Some(PatientAction::BackToHome) => Ok(Some(crate::app::SelectedApp::None)),
            Some(PatientAction::BackToList) => Ok(None),
            None => Ok(None),
        }
    }

    fn render(&self, frame: &mut Frame) {
        // Set the global background color
        let area = frame.area();
        frame.render_widget(
            Block::default().style(Style::default().bg(Color::Rgb(16, 16, 28))),
            area,
        );

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(10),   // Table
                Constraint::Length(3), // Patient details or help text
                Constraint::Length(2), // Back button
                Constraint::Length(1), // Error Message
            ])
            .margin(1)
            .split(area);

        // Header block
        let header_block = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(Color::Rgb(75, 75, 120)))
            .style(Style::default().bg(Color::Rgb(16, 16, 28)));
        frame.render_widget(header_block, layout[0]);

        // Header title
        let title = Paragraph::new("🏥 PATIENT LIST")
            .style(
                Style::default()
                    .fg(Color::Rgb(230, 230, 250))
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Rgb(16, 16, 28)),
            )
            .alignment(Alignment::Center);
        frame.render_widget(title, layout[0]);

        // Define the table headers
        let header_cells = [
            "ID",
            "First Name",
            "Last Name",
            "Date of Birth",
            "Gender",
            "Phone",
            "Address",
        ]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Rgb(230, 230, 250))));
        let header = Row::new(header_cells)
            .style(Style::default().bg(Color::Rgb(26, 26, 36)))
            .height(1)
            .bottom_margin(1);

        // Map the patient data to table rows
        let rows = self.patients.iter().map(|patient| {
            let cells = vec![
                Cell::from(patient.id.to_string()),
                Cell::from(patient.first_name.clone()),
                Cell::from(patient.last_name.clone()),
                Cell::from(patient.date_of_birth.clone()),
                Cell::from(match patient.gender {
                    crate::models::Gender::Male => "Male",
                    crate::models::Gender::Female => "Female",
                    crate::models::Gender::Other => "Other",
                }),
                Cell::from(patient.phone_number.clone()),
                Cell::from(patient.address.clone()),
            ];
            Row::new(cells).height(1).bottom_margin(0)
        });

        // Create the table widget with highlight style based on focus
        let table_highlight_style = if self.focus_index == PATIENT_LIST {
            Style::default()
                .bg(Color::Rgb(40, 40, 65))
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .bg(Color::Rgb(30, 30, 45))
                .add_modifier(Modifier::BOLD)
        };

        let table = Table::new(
            rows,
            [
                Constraint::Percentage(5),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(10),
                Constraint::Percentage(10),
                Constraint::Percentage(15),
                Constraint::Percentage(30),
            ],
        )
        .header(header)
        .block(
            Block::default()
                .title(" Patients ")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Rgb(75, 75, 120)))
                .style(Style::default().bg(Color::Rgb(22, 22, 35))),
        )
        .row_highlight_style(table_highlight_style)
        .highlight_symbol(if self.focus_index == PATIENT_LIST {
            "► "
        } else {
            "  "
        });

        // Render the table
        frame.render_stateful_widget(table, layout[1], &mut self.state.clone());

        // Patient details or help text
        if self.show_details && self.state.selected().is_some() {
            if let Some(patient) = self.selected_patient() {
                let details = format!(
                    "Details for {} {}: Born on {}, Gender: {}, Phone: {}, Address: {}",
                    patient.first_name,
                    patient.last_name,
                    patient.date_of_birth,
                    match patient.gender {
                        crate::models::Gender::Male => "Male",
                        crate::models::Gender::Female => "Female",
                        crate::models::Gender::Other => "Other",
                    },
                    patient.phone_number,
                    patient.address
                );

                let details_widget = Paragraph::new(details)
                    .style(Style::default().fg(Color::Rgb(200, 200, 220)))
                    .block(
                        Block::default()
                            .title(" Patient Details ")
                            .borders(Borders::ALL)
                            .border_type(BorderType::Rounded)
                            .border_style(Style::default().fg(Color::Rgb(75, 75, 120))),
                    )
                    .wrap(Wrap { trim: true });

                frame.render_widget(details_widget, layout[2]);
            }
        } else {
            // Help text
            let help_text =
                Paragraph::new("↑↓: Navigate | Enter: View Details | R: Refresh | Tab: Focus")
                    .style(Style::default().fg(Color::Rgb(140, 140, 170)))
                    .alignment(Alignment::Center);
            frame.render_widget(help_text, layout[2]);
        }

        // Back button - using the format you provided
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
        frame.render_widget(back_button, layout[3]);

        // Check for errors and display if any
        if let Some(error) = &self.error_message {
            let error_paragraph = Paragraph::new(error.as_str())
                .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
                .alignment(Alignment::Center);
            frame.render_widget(error_paragraph, layout[4]); // Render in the last section
        }
    }
}

impl Default for ListPatients {
    fn default() -> Self {
        Self::new()
    }
}
