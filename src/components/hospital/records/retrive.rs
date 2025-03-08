//! Module for retrieving medical records.
//!
//! This module provides a user interface component for displaying and searching medical records from the hospital database. It allows users to view a list of records, filter them based on search input, and view detailed information about a selected record. It encapsulates the logic for fetching records, filtering them, handling user input, and rendering the user interface. It exposes the `RetrieveRecords` struct, which implements the `Component` trait for integration with the application's UI framework.
//!
//! This module primarily uses the `MedicalRecord` struct from the `models` module and interacts with the database through functions provided in the `db` module.

use crate::app::SelectedApp;
use crate::components::Component;
use crate::db;
use crate::models::MedicalRecord;
use crate::tui::Frame;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{prelude::*, widgets::*};

// Constants defining focusable elements
const SEARCH_FIELD: usize = 0;
const RECORD_LIST: usize = 1;
const BACK_BUTTON: usize = 2;

/// Component for listing and searching medical records.
///
/// This struct represents the UI component for displaying and interacting with medical records. It manages the state of the record list, including the displayed records, search input, selection state, and error messages. The `RetrieveRecords` component allows users to view, search, and select medical records within the application.
///
/// The `RetrieveRecords` component encapsulates the following:
/// - A list of all medical records fetched from the database.
/// - A list of filtered medical records, which is a subset of the records based on the search input.
/// - The current search input string.
/// - A boolean flag indicating whether the component is in search mode.
/// - The state of the table selection, using `TableState`.
/// - An optional error message to display to the user.
/// - A boolean flag to control the display of detailed record information.
/// - An index to track the currently focused UI element (search field, record list, or back button).
///
/// This component implements the `Component` trait, enabling it to receive input events and render its UI. It interacts with the database to fetch records, filters records based on user input, and displays the records using the `ratatui` library.
pub struct RetrieveRecords {
    records: Vec<MedicalRecord>,          // All records
    filtered_records: Vec<MedicalRecord>, // Filtered records
    search_input: String,                 // Search input
    is_searching: bool,                   // Search mode flag
    state: TableState,                    // Table selection state
    error_message: Option<String>,        // Error message, if any
    show_details: bool,                   // Flag to show details of selected record
    focus_index: usize,                   // Focus index
}

impl RetrieveRecords {
    /// Creates a new `RetrieveRecords` component.
    ///
    /// This function constructs a new instance of the `RetrieveRecords` component, initializing its internal state. It sets up empty vectors for records and filtered records, clears the search input, disables search mode, and initializes the table state. The focus is initially set on the record list.
    ///
    /// # Returns
    ///
    /// A new instance of `RetrieveRecords` with default values.
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
            filtered_records: Vec::new(),
            search_input: String::new(),
            is_searching: false,
            state: TableState::default(),
            error_message: None,
            show_details: false,
            focus_index: RECORD_LIST,
        }
    }

    /// Fetches medical records from the database.
    ///
    /// This function retrieves all medical records from the database using the `db::get_all_medical_records()` function. Upon successful retrieval, it updates the `records` field, filters the records based on the current search input, and selects the first record in the list. If no records are found, the selection is cleared. In case of an error during retrieval, the function sets an error message.
    ///
    /// # Errors
    ///
    /// Returns an `anyhow::Error` if the database query fails.
    ///
    /// # Side Effects
    ///
    /// - Updates `records` with the records from the database.
    /// - Updates `filtered_records` based on the current search input.
    /// - Sets the selected record in `state`.
    /// - Sets an error message in `error_message` if the database query fails.
    ///
    /// # Postconditions
    ///
    /// - `records` will contain all medical records from the database.
    /// - `filtered_records` will be populated based on the current search input.
    /// - `state` will have a record selected if any are available; otherwise, no record will be selected.
    pub fn fetch_records(&mut self) -> Result<()> {
        match db::get_all_medical_records() {
            Ok(records) => {
                self.records = records;
                self.filter_records();

                // Select the first record if records exist
                if self.filtered_records.is_empty() {
                    self.state.select(None);
                } else {
                    let selection = self
                        .state
                        .selected()
                        .unwrap_or(0)
                        .min(self.filtered_records.len() - 1);
                    self.state.select(Some(selection));
                }
                self.error_message = None;
                Ok(())
            }
            Err(e) => {
                // Set error message to be displayed in UI
                self.error_message = Some(format!("Failed to fetch records: {}", e));
                Ok(()) // Return Ok to continue program, display error
            }
        }
    }

    /// Filters the records based on the current search input.
    ///
    /// This function filters the `records` based on the `search_input`. It converts the search input to lowercase and checks if the `patient_id`, `doctor_notes`, or `diagnosis` of each record contains the search term. The filtered records are stored in `filtered_records`. If the search input is empty, all records are considered as matches. The selection state is reset if the selected record is out of bounds after filtering, or if no records match the search criteria.
    ///
    /// # Side Effects
    ///
    /// - Updates `filtered_records` to contain the filtered records.
    /// - May update the selected index in `state`.
    ///
    /// # Postconditions
    ///
    /// - `filtered_records` will contain only the records that match the current `search_input`.
    /// - The `state`'s selected index will be updated to a valid index or `None`.
    fn filter_records(&mut self) {
        if self.search_input.is_empty() {
            self.filtered_records = self.records.clone();
        } else {
            let search_term = self.search_input.to_lowercase();
            self.filtered_records = self
                .records
                .iter()
                .filter(|r| {
                    r.patient_id.to_string().contains(&search_term)
                        || r.doctor_notes.to_lowercase().contains(&search_term)
                        || r.diagnosis.to_lowercase().contains(&search_term)
                })
                .cloned()
                .collect();
        }

        // Reset selection if it's out of bounds
        if let Some(selected) = self.state.selected() {
            if selected >= self.filtered_records.len() && !self.filtered_records.is_empty() {
                self.state.select(Some(0));
            } else if self.filtered_records.is_empty() {
                self.state.select(None);
            }
        }
    }

    /// Selects the next record in the list.
    ///
    /// This function moves the selection to the next record in the `filtered_records` list. If the current selection is at the end of the list, it wraps around to the beginning. If the record list is empty, it does nothing.
    ///
    /// # Side Effects
    ///
    /// Updates the `state` to select the next record.
    fn select_next(&mut self) {
        if self.filtered_records.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.filtered_records.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    /// Selects the previous record in the list.
    ///
    /// This function moves the selection to the previous record in the `filtered_records` list. If the current selection is at the beginning of the list, it wraps around to the end. If the record list is empty, it does nothing.
    ///
    /// # Side Effects
    ///
    /// Updates the `state` to select the previous record.
    fn select_previous(&mut self) {
        if self.filtered_records.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.filtered_records.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    /// Toggles detailed view of the selected record.
    ///
    /// This function toggles the visibility of detailed information for the currently selected record. It only toggles the visibility if there are filtered records and a record is selected.
    ///
    /// # Side Effects
    ///
    /// Modifies the `show_details` flag.
    fn toggle_details(&mut self) {
        if !self.filtered_records.is_empty() && self.state.selected().is_some() {
            self.show_details = !self.show_details;
        }
    }

    /// Moves focus to the next UI element.
    ///
    /// This function shifts the focus to the next interactive element in the UI. It cycles through the search field, the record list, and the back button. It also updates the `is_searching` flag based on the new focus.
    ///
    /// # Side Effects
    ///
    /// Updates the `focus_index` and `is_searching` fields.
    fn focus_next(&mut self) {
        self.focus_index = (self.focus_index + 1) % 3;
        self.is_searching = self.focus_index == SEARCH_FIELD;
    }

    /// Moves focus to the previous UI element.
    ///
    /// This function shifts the focus to the previous interactive element in the UI. It cycles through the search field, the record list, and the back button. It also updates the `is_searching` flag based on the new focus.
    ///
    /// # Side Effects
    ///
    /// Updates the `focus_index` and `is_searching` fields.
    fn focus_previous(&mut self) {
        self.focus_index = (self.focus_index + 2) % 3;
        self.is_searching = self.focus_index == SEARCH_FIELD;
    }

    /// Handles keyboard input events.
    ///
    /// This function processes keyboard input to manage user interactions with the component. It handles input for search mode, navigation, record selection, toggling details, and returning to the previous screen.
    ///
    /// # Parameters
    ///
    /// - `key`: The `KeyEvent` representing the user's keyboard input.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Some(SelectedApp::None))` when the back button is pressed, indicating that the application should return to the previous screen. Returns `Ok(Some(SelectedApp::None))` when Esc is pressed when showing details. Returns `Ok(None)` in all other cases, indicating that the current app is selected.
    ///
    /// # Errors
    ///
    /// Returns an `anyhow::Result` that can contain an error if fetching records fails.
    ///
    /// # Side Effects
    ///
    /// - Updates the component's state based on the key pressed, including search input, selected record, focus, and visibility of details.
    /// - May trigger a database fetch if the 'r' key is pressed.
    ///
    /// # Preconditions
    ///
    /// - The component must be initialized.
    ///
    /// # Postconditions
    ///
    /// - The component's state will be updated to reflect the user's input.
    /// - If the back button is pressed, the function returns `Some(SelectedApp::None)`.
    /// - If the 'r' key is pressed, the function attempts to fetch records.
    pub fn handle_input(&mut self, key: KeyEvent) -> Result<Option<SelectedApp>> {
        if self.is_searching {
            // Handle input in search mode
            match key.code {
                KeyCode::Char(c) => {
                    self.search_input.push(c);
                    self.filter_records();
                }
                KeyCode::Backspace => {
                    self.search_input.pop();
                    self.filter_records();
                }
                KeyCode::Enter | KeyCode::Down | KeyCode::Tab => {
                    if !self.filtered_records.is_empty() {
                        self.is_searching = false;
                        self.focus_index = RECORD_LIST;
                        self.state.select(Some(0));
                    }
                }
                KeyCode::Esc => {
                    self.is_searching = false;
                    self.focus_index = RECORD_LIST;
                }
                _ => {}
            }
            return Ok(None);
        }

        // Normal mode input handling
        match key.code {
            KeyCode::Char('/') | KeyCode::Char('s') | KeyCode::Char('S') => {
                self.is_searching = true;
                self.focus_index = SEARCH_FIELD;
                return Ok(None);
            }
            KeyCode::Tab => self.focus_next(),
            KeyCode::BackTab => self.focus_previous(),
            KeyCode::Down | KeyCode::Right => {
                if self.focus_index == RECORD_LIST {
                    self.select_next();
                } else {
                    self.focus_next();
                }
            }
            KeyCode::Up | KeyCode::Left => {
                if self.focus_index == RECORD_LIST {
                    self.select_previous();
                } else {
                    self.focus_previous();
                }
            }
            KeyCode::Enter => {
                if self.focus_index == BACK_BUTTON {
                    return Ok(Some(SelectedApp::None));
                } else if self.focus_index == RECORD_LIST {
                    self.toggle_details();
                } else if self.focus_index == SEARCH_FIELD {
                    self.is_searching = true;
                }
            }
            KeyCode::Char('b') | KeyCode::Char('B') => {
                return Ok(Some(SelectedApp::None));
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                self.fetch_records()?;
            }
            KeyCode::Esc => {
                if self.show_details {
                    self.show_details = false;
                } else {
                    return Ok(Some(SelectedApp::None));
                }
            }
            _ => {}
        }
        Ok(None)
    }

    /// Returns a reference to the currently selected record, if any.
    ///
    /// This function retrieves a reference to the `MedicalRecord` that is currently selected in the UI. It uses the `state` to determine the selected index and then retrieves the corresponding record from the `filtered_records` vector.
    ///
    /// # Returns
    ///
    /// - `Some(&MedicalRecord)` if a record is selected.
    /// - `None` if no record is selected.
    fn selected_record(&self) -> Option<&MedicalRecord> {
        self.state
            .selected()
            .and_then(|i| self.filtered_records.get(i))
    }
}

impl Component for RetrieveRecords {
    /// Handles input events at the component level.
    ///
    /// This function is part of the `Component` trait implementation and forwards the `KeyEvent` to the `handle_input` method of the `RetrieveRecords` struct.
    ///
    /// # Parameters
    ///
    /// - `event`: The `KeyEvent` representing the user's input.
    ///
    /// # Returns
    ///
    /// Returns the result of the internal `handle_input` method, which is an `Result<Option<SelectedApp>>`.
    fn handle_input(&mut self, event: KeyEvent) -> Result<Option<SelectedApp>> {
        self.handle_input(event)
    }

    /// Renders the component to the terminal frame.
    ///
    /// This function is responsible for rendering the `RetrieveRecords` component to the terminal using the provided `Frame`. It defines the layout of the UI elements, renders the search field, the record list, detailed record information, help text, and the back button. It also handles displaying error messages if they exist.
    ///
    /// # Parameters
    ///
    /// - `frame`: A mutable reference to the `Frame` where the component will be rendered.
    fn render(&self, frame: &mut Frame) {
        let area = frame.area();
        frame.render_widget(
            Block::default().style(Style::default().bg(Color::Rgb(16, 16, 28))),
            area,
        );

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(3),
                Constraint::Length(2),
                Constraint::Length(1),
            ])
            .margin(1)
            .split(area);

        // Header
        let header_block = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(Color::Rgb(75, 75, 120)))
            .style(Style::default().bg(Color::Rgb(16, 16, 28)));
        frame.render_widget(header_block, layout[0]);

        let title = Paragraph::new("🏥 MEDICAL RECORDS")
            .style(
                Style::default()
                    .fg(Color::Rgb(230, 230, 250))
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Rgb(16, 16, 28)),
            )
            .alignment(Alignment::Center);
        frame.render_widget(title, layout[0]);

        // Search field
        let search_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(Span::styled(
                " Search Records ",
                Style::default()
                    .fg(Color::Rgb(230, 230, 250))
                    .add_modifier(Modifier::BOLD),
            ))
            .border_style(if self.is_searching {
                Style::default().fg(Color::Rgb(250, 250, 110)) // Yellow
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

        // Table headers
        let header_cells = ["ID", "Patient ID", "Diagnosis"]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().fg(Color::Rgb(230, 230, 250))));
        let header = Row::new(header_cells)
            .style(Style::default().bg(Color::Rgb(80, 60, 130)))
            .height(1);

        // Map records to table rows
        let rows = self.filtered_records.iter().map(|record| {
            let cells = vec![
                Cell::from(record.id.to_string()),
                Cell::from(record.patient_id.to_string()),
                Cell::from(record.diagnosis.clone()),
            ];
            Row::new(cells)
                .height(1)
                .bottom_margin(0)
                .style(Style::default().fg(Color::Rgb(220, 220, 240)))
        });

        let selected_style = Style::default()
            .fg(Color::Rgb(250, 250, 110))
            .bg(Color::Rgb(40, 40, 60))
            .add_modifier(Modifier::BOLD);

        let table_title = if !self.search_input.is_empty() {
            format!(
                " Records ({} of {} matches) ",
                self.filtered_records.len(),
                self.records.len()
            )
        } else {
            format!(" Records ({}) ", self.records.len())
        };

        let table = Table::new(
            rows,
            [
                Constraint::Percentage(10),
                Constraint::Percentage(20),
                Constraint::Percentage(70),
            ],
        )
        .header(header)
        .block(
            Block::default()
                .title(table_title.clone())
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Rgb(75, 75, 120)))
                .style(Style::default().bg(Color::Rgb(22, 22, 35))),
        )
        .row_highlight_style(selected_style)
        .highlight_symbol(if self.focus_index == RECORD_LIST {
            "► "
        } else {
            "  "
        });

        // "No records" message
        if self.filtered_records.is_empty() {
            let message = if self.search_input.is_empty() {
                "No records found in database"
            } else {
                "No records match your search criteria"
            };

            let no_records = Paragraph::new(message)
                .style(Style::default().fg(Color::Rgb(220, 220, 240)))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .title(table_title.clone()) // Reuse the variable
                        .title_alignment(Alignment::Center)
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(Color::Rgb(75, 75, 120)))
                        .style(Style::default().bg(Color::Rgb(22, 22, 35))),
                );
            frame.render_widget(no_records, layout[2]);
        } else {
            frame.render_stateful_widget(table, layout[2], &mut self.state.clone());
        }

        // Details or help text
        if self.show_details && self.state.selected().is_some() {
            if let Some(record) = self.selected_record() {
                let details = format!(
                    "Record ID: {}\nPatient ID: {}\nDoctor's Notes: {}\nNurse's Notes: {}\nDiagnosis: {}\nPrescription: {}",
                    record.id,
                    record.patient_id,
                    record.doctor_notes,
                    record.nurse_notes.clone().unwrap_or_else(|| "N/A".to_string()),
                    record.diagnosis,
                    record.prescription.clone().unwrap_or_else(|| "N/A".to_string())
                );

                let details_widget = Paragraph::new(details)
                    .style(Style::default().fg(Color::Rgb(200, 200, 220)))
                    .block(
                        Block::default()
                            .title(" Record Details ")
                            .borders(Borders::ALL)
                            .border_type(BorderType::Rounded)
                            .border_style(Style::default().fg(Color::Rgb(75, 75, 120))),
                    )
                    .wrap(Wrap { trim: true });

                frame.render_widget(details_widget, layout[3]);
            }
        } else {
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

        // Back button
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

        // Error message
        if let Some(error) = &self.error_message {
            let error_paragraph = Paragraph::new(error.as_str())
                .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
                .alignment(Alignment::Center);
            frame.render_widget(error_paragraph, layout[5]);
        }
    }
}

impl Default for RetrieveRecords {
    /// Creates a default instance of `RetrieveRecords`.
    ///
    /// This function allows for the creation of a `RetrieveRecords` component with its default settings. It simply calls the `new` function to create a new component instance.
    ///
    /// # Returns
    ///
    /// A new `RetrieveRecords` component with default settings.
    fn default() -> Self {
        Self::new()
    }
}
