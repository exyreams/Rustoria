//! List Patient component for the Hospital application.
//!
//! This module provides functionality to display and search through a list of patients.
//! It supports:
//! - Viewing all patients in a tabular format
//! - Searching patients by name, ID, phone, or address
//! - Navigating through patients using keyboard shortcuts
//! - Viewing detailed information about selected patients

use crate::components::hospital::patients::PatientAction;
use crate::components::Component;
use crate::db;
use crate::models::Patient;
use crate::tui::Frame;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{prelude::*, widgets::*};

// Constants for focus indices
/// Index for the search field focus
const SEARCH_FIELD: usize = 0;
/// Index for the patient list focus
const PATIENT_LIST: usize = 1;
/// Index for the back button focus
const BACK_BUTTON: usize = 2;

/// Component to display and interact with a list of patients.
///
/// Provides functionality to view, search, and navigate through patients,
/// as well as view detailed information about individual patients.
pub struct ListPatients {
    /// All patients retrieved from the database
    patients: Vec<Patient>,
    /// Patients filtered by the search criteria
    filtered_patients: Vec<Patient>,
    /// Current search query text
    search_input: String,
    /// Flag indicating if user is actively searching
    is_searching: bool,
    /// Stateful table selection for the patient list
    state: TableState,
    /// Optional error message to display
    error_message: Option<String>,
    /// Flag indicating if detailed view is active
    show_details: bool,
    /// Current UI focus location (search, list, or back button)
    focus_index: usize,
}

impl ListPatients {
    /// Creates a new `ListPatients` component.
    ///
    /// Initializes with empty patients list and default settings.
    pub fn new() -> Self {
        Self {
            patients: Vec::new(),
            filtered_patients: Vec::new(),
            search_input: String::new(),
            is_searching: false,
            state: TableState::default(),
            error_message: None,
            show_details: false,
            focus_index: PATIENT_LIST,
        }
    }

    /// Fetches the patient data from the database.
    ///
    /// Updates both the main patients list and the filtered list based on
    /// any active search criteria. Handles selection state to ensure it remains
    /// valid after the update.
    ///
    /// # Returns
    /// * `Ok(())` if patients were fetched successfully or if an error occurred but was handled
    /// * `Err` is never returned as errors are stored in the component's state
    pub fn fetch_patients(&mut self) -> Result<()> {
        match db::get_all_patients() {
            Ok(patients) => {
                self.patients = patients;
                // Apply any existing search filter
                self.filter_patients();

                if self.filtered_patients.is_empty() {
                    // Start with no selection if no patients exist
                    self.state.select(None);
                } else {
                    // Ensure selection is within bounds, default to first patient
                    let selection = self
                        .state
                        .selected()
                        .unwrap_or(0)
                        .min(self.filtered_patients.len() - 1);
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

    /// Filter patients based on search input.
    ///
    /// Applies the current search query to filter the patient list. Searches across
    /// multiple fields: first name, last name, ID, phone number, and address.
    /// The search is case-insensitive and matches partial strings.
    fn filter_patients(&mut self) {
        if self.search_input.is_empty() {
            // If search is empty, show all patients
            self.filtered_patients = self.patients.clone();
        } else {
            let search_term = self.search_input.to_lowercase();
            self.filtered_patients = self
                .patients
                .iter()
                .filter(|p| {
                    // Case-insensitive search in multiple fields
                    p.first_name.to_lowercase().contains(&search_term)
                        || p.last_name.to_lowercase().contains(&search_term)
                        || p.id.to_string().contains(&search_term)
                        || p.phone_number.to_lowercase().contains(&search_term)
                        || p.address.to_lowercase().contains(&search_term)
                })
                .cloned()
                .collect();
        }

        // Reset selection if it's now out of bounds
        if let Some(selected) = self.state.selected() {
            if selected >= self.filtered_patients.len() && !self.filtered_patients.is_empty() {
                self.state.select(Some(0));
            } else if self.filtered_patients.is_empty() {
                self.state.select(None);
            }
        }
    }

    /// Selects the next patient in the list.
    ///
    /// Handles wrapping around to the first patient when at the end of the list.
    /// Does nothing if the list is empty.
    fn select_next(&mut self) {
        if self.filtered_patients.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.filtered_patients.len() - 1 {
                    0 // Wrap around to the first patient
                } else {
                    i + 1
                }
            }
            None => 0, // Select the first patient if nothing is selected
        };
        self.state.select(Some(i));
    }

    /// Selects the previous patient in the list.
    ///
    /// Handles wrapping around to the last patient when at the beginning of the list.
    /// Does nothing if the list is empty.
    fn select_previous(&mut self) {
        if self.filtered_patients.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.filtered_patients.len() - 1 // Wrap around to the last patient
                } else {
                    i - 1
                }
            }
            None => 0, // Select the first patient if nothing is selected
        };
        self.state.select(Some(i));
    }

    /// Toggles the detailed view for the currently selected patient.
    ///
    /// Shows or hides additional patient information. Does nothing if no patient
    /// is selected or if the patient list is empty.
    fn toggle_details(&mut self) {
        if !self.filtered_patients.is_empty() && self.state.selected().is_some() {
            self.show_details = !self.show_details;
        }
    }

    /// Focuses the next UI element in the tab order.
    ///
    /// Cycles through search field, patient list, and back button.
    /// Updates the is_searching flag appropriately.
    fn focus_next(&mut self) {
        self.focus_index = (self.focus_index + 1) % 3;
        if self.focus_index == SEARCH_FIELD {
            self.is_searching = true;
        } else {
            self.is_searching = false;
        }
    }

    /// Focuses the previous UI element in the tab order.
    ///
    /// Cycles through search field, patient list, and back button in reverse.
    /// Updates the is_searching flag appropriately.
    fn focus_previous(&mut self) {
        self.focus_index = (self.focus_index + 2) % 3; // +2 instead of -1 to avoid negative numbers
        if self.focus_index == SEARCH_FIELD {
            self.is_searching = true;
        } else {
            self.is_searching = false;
        }
    }

    /// Handles input events for the component, specifically for patient selection.
    ///
    /// Processes keyboard inputs to handle navigation, searching, and actions like
    /// viewing details or returning to the home screen.
    ///
    /// # Arguments
    /// * `key` - The keyboard event to handle
    ///
    /// # Returns
    /// * `Ok(Some(PatientAction))` if an action should be taken (like returning to home)
    /// * `Ok(None)` if the event was handled but no action is needed
    /// * `Err` if an error occurred during handling
    pub fn handle_input(&mut self, key: KeyEvent) -> Result<Option<PatientAction>> {
        // If searching, handle search input
        if self.is_searching {
            match key.code {
                KeyCode::Char(c) => {
                    self.search_input.push(c);
                    self.filter_patients();
                }
                KeyCode::Backspace => {
                    self.search_input.pop();
                    self.filter_patients();
                }
                KeyCode::Enter | KeyCode::Down | KeyCode::Tab => {
                    if !self.filtered_patients.is_empty() {
                        self.is_searching = false;
                        self.focus_index = PATIENT_LIST;
                        self.state.select(Some(0));
                    }
                }
                KeyCode::Esc => {
                    self.is_searching = false;
                    self.focus_index = PATIENT_LIST;
                }
                _ => {}
            }
            return Ok(None);
        }

        // Normal navigation when not searching
        match key.code {
            KeyCode::Char('/') | KeyCode::Char('s') | KeyCode::Char('S') => {
                self.is_searching = true;
                self.focus_index = SEARCH_FIELD;
                return Ok(None);
            }
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
                } else if self.focus_index == PATIENT_LIST {
                    self.toggle_details();
                } else if self.focus_index == SEARCH_FIELD {
                    self.is_searching = true;
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

    /// Gets the currently selected patient, if any.
    ///
    /// # Returns
    /// * `Some(&Patient)` - Reference to the selected patient
    /// * `None` - If no patient is selected or the list is empty
    fn selected_patient(&self) -> Option<&Patient> {
        self.state
            .selected()
            .and_then(|i| self.filtered_patients.get(i))
    }
}

impl Component for ListPatients {
    /// Handles input events at the component level.
    ///
    /// Translates PatientAction to SelectedApp actions for the main app to handle.
    ///
    /// # Arguments
    /// * `event` - The keyboard event to process
    ///
    /// # Returns
    /// * `Ok(Some(SelectedApp))` - If the app should change screens
    /// * `Ok(None)` - If no app-level action is needed
    /// * `Err` - If an error occurred during handling
    fn handle_input(&mut self, event: KeyEvent) -> Result<Option<crate::app::SelectedApp>> {
        match self.handle_input(event)? {
            Some(PatientAction::BackToHome) => Ok(Some(crate::app::SelectedApp::None)),
            Some(PatientAction::BackToList) => Ok(None),
            None => Ok(None),
        }
    }

    /// Renders the list patients component to the frame.
    ///
    /// Draws the entire UI including header, search field, patient table,
    /// detail view or help text, back button, and error messages if any.
    ///
    /// # Arguments
    /// * `frame` - The frame to render the component on
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
                Constraint::Length(3), // Search input
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

        // Search input field
        let search_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(Span::styled(
                " Search Patients ",
                Style::default()
                    .fg(Color::Rgb(230, 230, 250))
                    .add_modifier(Modifier::BOLD),
            ))
            .border_style(if self.is_searching {
                Style::default().fg(Color::Rgb(250, 250, 110)) // Yellow when active
            } else {
                Style::default().fg(Color::Rgb(75, 75, 120))
            })
            .style(Style::default().bg(Color::Rgb(22, 22, 35)));

        let search_paragraph = Paragraph::new(self.search_input.clone())
            .style(
                Style::default()
                    .fg(Color::Rgb(220, 220, 240))
                    .bg(Color::Rgb(22, 22, 35)),
            )
            .block(search_block);
        frame.render_widget(search_paragraph, layout[1]);

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
            .style(Style::default().bg(Color::Rgb(80, 60, 130)))
            .height(1);

        // Map the patient data to table rows - now using filtered_patients
        let rows = self.filtered_patients.iter().map(|patient| {
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

        // Create a title that includes the number of results if search is active
        let table_title = if !self.search_input.is_empty() {
            format!(
                " Patients ({} of {} matches) ",
                self.filtered_patients.len(),
                self.patients.len()
            )
        } else {
            format!(" Patients ({}) ", self.patients.len())
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
                .title(table_title.clone()) // Clone here to fix the move error
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

        // If no patients, show a message
        if self.filtered_patients.is_empty() {
            let message = if self.search_input.is_empty() {
                "No patients found in database"
            } else {
                "No patients match your search criteria"
            };

            let no_patients = Paragraph::new(message)
                .style(Style::default().fg(Color::Rgb(220, 220, 240)))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .title(table_title) // Now we can use table_title again
                        .title_alignment(Alignment::Center)
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(Color::Rgb(75, 75, 120)))
                        .style(Style::default().bg(Color::Rgb(22, 22, 35))),
                );
            frame.render_widget(no_patients, layout[2]);
        } else {
            // Render the table
            frame.render_stateful_widget(table, layout[2], &mut self.state.clone());
        }

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

                frame.render_widget(details_widget, layout[3]);
            }
        } else {
            // Help text - updated to include search shortcuts
            let help_text = if self.is_searching {
                "Type to search | ↓/Enter: To results | Esc: Cancel search"
            } else {
                "/ or s: Search | ↑↓: Navigate | Enter: View Details | R: Refresh | Tab: Focus"
            };

            let help_paragraph = Paragraph::new(help_text)
                .style(Style::default().fg(Color::Rgb(140, 140, 170)))
                .alignment(Alignment::Center);
            frame.render_widget(help_paragraph, layout[3]);
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
        frame.render_widget(back_button, layout[4]);

        // Check for errors and display if any
        if let Some(error) = &self.error_message {
            let error_paragraph = Paragraph::new(error.as_str())
                .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
                .alignment(Alignment::Center);
            frame.render_widget(error_paragraph, layout[5]);
        }
    }
}

impl Default for ListPatients {
    /// Provides a default instance of ListPatients.
    ///
    /// Simply calls new() to create the instance.
    fn default() -> Self {
        Self::new()
    }
}
