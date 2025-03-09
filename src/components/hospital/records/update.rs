//! `UpdateRecord` Component for the Hospital application.
//!
//! This module provides the `UpdateRecord` component, allowing users to select, view, and modify medical records within the application. The component encapsulates the logic for fetching records, filtering them based on search input, displaying them in a table, and handling user input for record updates. The `UpdateRecord` component is central to the functionality of updating existing medical records. It exposes the `UpdateRecord` struct, which manages the state and behavior of the update record UI, and interacts with the database through the `db` module to fetch and update medical records.
//! The primary types exposed are `UpdateRecord`, `UpdateState`, and `ConfirmAction`.

use crate::app::SelectedApp;
use crate::components::Component;
use crate::db;
use crate::models::{MedicalRecord, Patient};
use crate::tui::Frame;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{prelude::*, widgets::*};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// `ConfirmAction` Enum for confirmation dialog actions.
///
/// This enum defines the possible actions that can be taken after a user confirms an action in the confirmation dialog.
/// It's used to determine which action to execute when the user confirms their choice.
enum ConfirmAction {
    /// Represents the action to update a medical record.
    UpdateRecord,
}

/// `UpdateState` Enum to manage the UI flow.
///
/// This enum represents the different states of the `UpdateRecord` component,
/// guiding the user through the process of selecting and editing a medical record.
enum UpdateState {
    /// Represents the initial state where the user is selecting a record from a list.
    SelectingRecord,
    /// Represents the state after a record has been selected,
    /// allowing the user to view and edit its fields.
    EditingRecord,
}

/// `UpdateRecord` Component for updating medical records.
///
/// This struct represents the component responsible for displaying and managing the update process for medical records within the hospital application.
/// It handles user interactions such as selecting records, editing fields, and saving changes.
/// The component fetches data from the database, displays it in a user-friendly format, and allows for modifications.
///
/// # Fields
///
/// * `all_records`: A vector containing all medical records fetched from the database.
/// * `filtered_records`: A vector containing medical records filtered based on the search input.
/// * `search_input`: A string representing the current search query entered by the user.
/// * `is_searching`: A boolean flag indicating whether the component is in search mode.
/// * `table_state`: The state of the table used for record selection, managing the currently selected row.
/// * `update_state`: The current state of the component, determining the UI displayed (selecting or editing).
/// * `record_id_input`: A string representing the user's input for the record ID.
/// * `record`: The `MedicalRecord` object that is currently being updated.
/// * `loaded`: A boolean flag indicating whether a record has been loaded for editing.
/// * `selected_field`: An optional index indicating the currently selected field for editing.
/// * `edit_table_state`: The state of the table used for editing the record fields.
/// * `input_value`: A string representing the current input value for the selected field being edited.
/// * `editing`: A boolean flag indicating whether the user is currently editing a field.
/// * `error_message`: An optional string containing an error message to be displayed to the user.
/// * `error_timer`: An optional `Instant` representing the time when the error message was set, used for timeout.
/// * `success_message`: An optional string containing a success message to be displayed to the user.
/// * `success_timer`: An optional `Instant` representing the time when the success message was set, used for timeout.
/// * `show_confirmation`: A boolean flag indicating whether to show a confirmation dialog.
/// * `confirmation_message`: A string containing the message displayed in the confirmation dialog.
/// * `confirmed_action`: An optional `ConfirmAction` indicating which action to perform after confirmation.
/// * `confirmation_selected`: The index of the selected button in the confirmation dialog (0 for Yes, 1 for No).
pub struct UpdateRecord {
    all_records: Vec<MedicalRecord>,      // All records for selection
    filtered_records: Vec<MedicalRecord>, // Filtered records for selection
    patients: HashMap<i64, Patient>,      // Map of patient ID to patient info
    search_input: String,                 // Search input
    is_searching: bool,                   // Search mode flag
    table_state: TableState,              // Table selection state
    update_state: UpdateState,            // Current state
    record_id_input: String,              // Record ID input
    record: MedicalRecord,                // The record being updated
    loaded: bool,                         // Flag if record is loaded for editing
    selected_field: Option<usize>,        // Currently selected field for editing
    edit_table_state: TableState,         // Table state for editing view
    input_value: String,                  // Current input value
    editing: bool,                        // Whether we're editing a field
    error_message: Option<String>,
    error_timer: Option<Instant>,
    success_message: Option<String>,
    success_timer: Option<Instant>,
    show_confirmation: bool,      // Whether to show confirmation dialog
    confirmation_message: String, // Message in the confirmation dialog
    confirmed_action: Option<ConfirmAction>, // Action to perform if confirmed
    confirmation_selected: usize, // Which confirmation button is selected (0 for Yes, 1 for No)
}

// Field constants
const ID_INPUT: usize = 0;
const PATIENT_ID_INPUT: usize = 1;
const DOCTOR_NOTES_INPUT: usize = 2;
const NURSE_NOTES_INPUT: usize = 3;
const DIAGNOSIS_INPUT: usize = 4;
const PRESCRIPTION_INPUT: usize = 5;
const INPUT_FIELDS: usize = 5; // Corrected

impl UpdateRecord {
    /// Creates a new `UpdateRecord` component, initializing its state.
    ///
    /// This function constructs a new instance of the `UpdateRecord` component with default values.
    /// It initializes the internal state, including table selection, and other flags and data structures required for operation.
    ///
    /// # Returns
    ///
    /// A new `UpdateRecord` instance with default configurations.
    pub fn new() -> Self {
        let mut selection_state = TableState::default();
        selection_state.select(Some(0)); // Default selection

        let mut edit_table_state = TableState::default();
        edit_table_state.select(Some(0));

        Self {
            all_records: Vec::new(),
            filtered_records: Vec::new(),
            patients: HashMap::new(),
            search_input: String::new(),
            is_searching: false,
            table_state: selection_state,
            update_state: UpdateState::SelectingRecord,
            record_id_input: String::new(),
            record: MedicalRecord {
                id: 0,
                patient_id: 0,
                doctor_notes: String::new(),
                nurse_notes: None,
                diagnosis: String::new(),
                prescription: None,
            },
            loaded: false,
            selected_field: Some(0),
            edit_table_state,
            input_value: String::new(),
            editing: false,
            error_message: None,
            error_timer: None,
            success_message: None,
            success_timer: None,
            show_confirmation: false,
            confirmation_message: String::new(),
            confirmed_action: None,
            confirmation_selected: 0, // Default to "Yes"
        }
    }

    /// Fetches all medical records from the database and updates the component's state.
    ///
    /// This method retrieves all medical records from the database using the `db::get_all_medical_records()` function.
    /// It then updates the `all_records` field with the fetched records, and filters the records
    /// based on the current search input.  Any errors during database interaction are propagated.
    ///
    /// # Errors
    ///
    /// Returns an `anyhow::Result` which will contain an error if the database query fails.
    pub fn fetch_records(&mut self) -> Result<()> {
        self.all_records = db::get_all_medical_records()?;
        self.fetch_patients_data()?;
        self.filter_records();
        Ok(())
    }

    /// Fetches patient data for all patient IDs in the records.
    ///
    /// This method retrieves patient information for all patient IDs in the records and
    /// stores it in the patients HashMap for quick lookup.
    ///
    /// # Errors
    ///
    /// Returns an `anyhow::Result` which will contain an error if the database query fails.
    fn fetch_patients_data(&mut self) -> Result<()> {
        // Clear existing patients
        self.patients.clear();

        // Get all patients from database
        match db::get_all_patients() {
            Ok(all_patients) => {
                // Create map of patient ID to patient data
                for patient in all_patients {
                    self.patients.insert(patient.id, patient);
                }
                Ok(())
            }
            Err(e) => {
                self.set_error(format!("Failed to fetch patient data: {}", e));
                Ok(()) // Continue program but with error message
            }
        }
    }

    /// Returns the patient information for a given patient ID.
    ///
    /// This function retrieves the patient information for the specified patient ID from the `patients` HashMap.
    ///
    /// # Parameters
    ///
    /// - `patient_id`: The ID of the patient to retrieve information for.
    ///
    /// # Returns
    ///
    /// - `Some(&Patient)` if the patient information exists.
    /// - `None` if the patient information does not exist.
    fn get_patient(&self, patient_id: i64) -> Option<&Patient> {
        self.patients.get(&patient_id)
    }

    /// Filters records based on the search term in the `search_input`.
    ///
    /// This method filters the `all_records` based on the content of `search_input`.
    /// If `search_input` is empty, it clones all records into `filtered_records`.
    /// Otherwise, it filters records where the `patient_id`, `doctor_notes`, or `diagnosis` contains the search term (case-insensitive).
    fn filter_records(&mut self) {
        if self.search_input.is_empty() {
            self.filtered_records = self.all_records.clone();
        } else {
            let search_term = self.search_input.to_lowercase();
            self.filtered_records = self
                .all_records
                .iter()
                .filter(|r| {
                    // Check if patient name matches search term
                    let patient_name_match = if let Some(patient) = self.patients.get(&r.patient_id)
                    {
                        patient.first_name.to_lowercase().contains(&search_term)
                            || patient.last_name.to_lowercase().contains(&search_term)
                    } else {
                        false
                    };

                    r.patient_id.to_string().contains(&search_term)
                        || r.doctor_notes.to_lowercase().contains(&search_term)
                        || r.diagnosis.to_lowercase().contains(&search_term)
                        || patient_name_match
                })
                .cloned()
                .collect();
        }

        // Maintain selection if possible
        if let Some(selected) = self.table_state.selected() {
            if selected >= self.filtered_records.len() && !self.filtered_records.is_empty() {
                self.table_state.select(Some(0));
            }
        }
    }

    /// Loads a medical record by ID from the database.
    ///
    /// This method attempts to retrieve a medical record from the database based on the provided `record_id`.
    /// If the record is found, it updates the `record` field with the fetched data, sets `loaded` to true,
    /// changes `update_state` to `EditingRecord`, and updates the input value.
    /// If the record is not found, it sets an error message.
    ///
    /// # Parameters
    ///
    /// * `record_id`: The ID of the medical record to load.
    ///
    /// # Errors
    ///
    /// Returns an `anyhow::Result` which will contain an error if the record is not found or if there is a database error.
    fn load_record_by_id(&mut self, record_id: i64) -> Result<()> {
        match db::get_medical_record(record_id) {
            Ok(record) => {
                self.record = record;
                self.loaded = true;
                self.update_state = UpdateState::EditingRecord;
                self.update_input_value();
                Ok(())
            }
            Err(_) => {
                self.set_error(format!("Record with ID {} doesn't exist", record_id));
                Err(anyhow::anyhow!("Record not found"))
            }
        }
    }

    /// Loads a record based on the ID input field.
    ///
    /// This method attempts to load a medical record using the value in `record_id_input`.
    /// It first parses the input string into an `i64`, and then calls `load_record_by_id` to fetch the record.
    /// If the parsing fails or the record doesn't exist, it sets an appropriate error message.
    ///
    /// # Errors
    ///
    /// Returns an `anyhow::Result` which will contain an error if the record ID is invalid or the record is not found.
    fn load_record(&mut self) -> Result<()> {
        if !self.loaded {
            if let Ok(record_id) = self.record_id_input.parse::<i64>() {
                match self.load_record_by_id(record_id) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e),
                }
            } else {
                self.set_error("Invalid Record ID format.".to_string());
                Err(anyhow::anyhow!("Invalid Record ID format"))
            }
        } else {
            Ok(())
        }
    }

    /// Loads the currently selected record from the table.
    ///
    /// This method loads a record based on the current selection in the table.
    /// It retrieves the ID of the selected record from `filtered_records` and calls `load_record_by_id` to load the record data.
    ///
    /// # Errors
    ///
    /// Returns an `anyhow::Result` which will contain an error if no record is selected.
    fn load_selected_record(&mut self) -> Result<()> {
        if let Some(selected) = self.table_state.selected() {
            if selected < self.filtered_records.len() {
                let record_id = self.filtered_records[selected].id;
                self.record_id_input = record_id.to_string();
                return self.load_record_by_id(record_id);
            }
        }
        self.set_error("No record selected".to_string());
        Err(anyhow::anyhow!("No record selected"))
    }

    /// Updates the `input_value` based on the `selected_field`.
    ///
    /// This method sets the `input_value` to the appropriate value from the current `record` based on the `selected_field`.
    /// It is called whenever the selected field changes or the input value needs to be refreshed.
    fn update_input_value(&mut self) {
        if !self.loaded {
            self.input_value = self.record_id_input.clone();
            return;
        }

        if let Some(field_index) = self.selected_field {
            self.input_value = match field_index {
                ID_INPUT => self.record.id.to_string(),
                PATIENT_ID_INPUT => self.record.patient_id.to_string(),
                DOCTOR_NOTES_INPUT => self.record.doctor_notes.clone(),
                NURSE_NOTES_INPUT => self.record.nurse_notes.clone().unwrap_or_default(),
                DIAGNOSIS_INPUT => self.record.diagnosis.clone(),
                PRESCRIPTION_INPUT => self.record.prescription.clone().unwrap_or_default(),
                _ => String::new(),
            };
        }
    }

    /// Applies the edited value to the selected field in the record data.
    ///
    /// This method updates the corresponding field in the `record` with the value in `input_value`.
    /// It is called when the user confirms the changes in the edit mode.
    fn apply_edited_value(&mut self) {
        if !self.editing || !self.loaded {
            return;
        }

        if let Some(field_index) = self.selected_field {
            match field_index {
                PATIENT_ID_INPUT => {
                    if let Ok(patient_id) = self.input_value.parse::<i64>() {
                        self.record.patient_id = patient_id;
                    } else {
                        self.set_error("Invalid Patient ID format.".to_string());
                        return; // Don't exit editing mode
                    }
                }
                DOCTOR_NOTES_INPUT => self.record.doctor_notes = self.input_value.clone(),
                NURSE_NOTES_INPUT => self.record.nurse_notes = Some(self.input_value.clone()),
                DIAGNOSIS_INPUT => self.record.diagnosis = self.input_value.clone(),
                PRESCRIPTION_INPUT => self.record.prescription = Some(self.input_value.clone()),
                _ => {}
            }
        }
        self.editing = false;
    }

    /// Shows a confirmation dialog with the provided message and action.
    ///
    /// This method sets the state of the component to show a confirmation dialog,
    /// allowing the user to confirm an action before it is performed.
    ///
    /// # Parameters
    ///
    /// * `message`: The message to display in the confirmation dialog.
    /// * `action`: The `ConfirmAction` to be taken if the user confirms.
    fn show_confirmation(&mut self, message: String, action: ConfirmAction) {
        self.show_confirmation = true;
        self.confirmation_message = message;
        self.confirmed_action = Some(action);
        self.confirmation_selected = 0; // Default Yes
    }

    /// Updates the record in the database.
    ///
    /// This method attempts to update the medical record in the database.
    /// If successful, it displays a success message, and reloads all records from the database.
    /// If an error occurs during the database update, an error message is displayed.
    ///
    /// # Errors
    ///
    /// Returns an `anyhow::Result` which will contain an error if the database update fails.
    fn update_record(&mut self) -> Result<()> {
        match db::update_medical_record(&self.record) {
            Ok(_) => {
                self.success_message = Some("Record updated successfully!".to_string());
                self.success_timer = Some(Instant::now());

                if let Ok(records) = db::get_all_medical_records() {
                    self.all_records = records.clone();
                    self.filtered_records = records;
                    self.filter_records();
                }

                Ok(())
            }
            Err(e) => {
                self.set_error(format!("Database error: {}", e));
                Err(e)
            }
        }
    }

    /// Returns to the record selection state, resetting the component's state.
    ///
    /// This method resets the component's state to allow for selecting a new record.
    /// It clears the loaded record, the input value, and any existing error or success messages.
    fn back_to_selection(&mut self) {
        self.update_state = UpdateState::SelectingRecord;
        self.loaded = false;
        self.record_id_input = String::new();
        self.editing = false;
        self.clear_error();
        self.clear_success();
    }

    /// Handles user input events for the `UpdateRecord` component.
    ///
    /// This method processes key events, updating the component's state based on user input.
    /// It manages input for record selection, searching, editing, and confirmation dialogs.
    ///
    /// # Parameters
    ///
    /// * `key`: The `KeyEvent` to handle.
    ///
    /// # Returns
    ///
    /// Returns a `Result<Option<SelectedApp>>`.  `Some(SelectedApp::None)` indicates
    /// the component handled the input and no app change is needed.  `Some(SelectedApp::Some(app))`
    /// indicates the app should change to the specified app.  `None` indicates the input
    /// was not handled.
    fn handle_input(&mut self, key: KeyEvent) -> Result<Option<SelectedApp>> {
        self.check_timeouts();

        // Handle confirmation dialog
        if self.show_confirmation {
            match key.code {
                KeyCode::Left | KeyCode::Right => {
                    self.confirmation_selected = 1 - self.confirmation_selected;
                }
                KeyCode::Enter => {
                    if self.confirmation_selected == 0 {
                        if let Some(ConfirmAction::UpdateRecord) = self.confirmed_action.take() {
                            let _ = self.update_record();
                        }
                    }
                    self.show_confirmation = false;
                }
                KeyCode::Esc => {
                    self.show_confirmation = false;
                    self.confirmed_action = None;
                }
                _ => {}
            }
            return Ok(None);
        }

        if self.editing {
            match key.code {
                KeyCode::Char(c) => {
                    self.input_value.push(c);
                }
                KeyCode::Backspace => {
                    self.input_value.pop();
                }
                KeyCode::Enter => {
                    self.apply_edited_value();
                }
                KeyCode::Esc => {
                    self.editing = false;
                    self.update_input_value();
                }
                _ => {}
            }
            return Ok(None);
        }

        if matches!(self.update_state, UpdateState::SelectingRecord) {
            match key.code {
                // Search mode
                KeyCode::Char(c) if self.is_searching => {
                    self.search_input.push(c);
                    self.filter_records();
                    self.clear_error();
                }
                KeyCode::Backspace if self.is_searching => {
                    self.search_input.pop();
                    self.filter_records();
                    self.clear_error();
                }
                KeyCode::Down if self.is_searching && !self.filtered_records.is_empty() => {
                    self.is_searching = false;
                    self.table_state.select(Some(0));
                }
                KeyCode::Esc if self.is_searching => {
                    self.is_searching = false;
                    self.search_input.clear();
                    self.filter_records();
                }
                KeyCode::Char('/') | KeyCode::Char('s') | KeyCode::Char('S')
                    if !self.is_searching =>
                {
                    self.is_searching = true;
                }

                // ID input
                KeyCode::Char(c) if !self.is_searching => {
                    self.record_id_input.push(c);
                    self.input_value = self.record_id_input.clone();
                    self.clear_error();
                }
                KeyCode::Backspace if !self.is_searching => {
                    self.record_id_input.pop();
                    self.input_value = self.record_id_input.clone();
                    self.clear_error();
                }

                // Navigation
                KeyCode::Up if !self.is_searching => {
                    let selected = self.table_state.selected().unwrap_or(0);
                    if selected > 0 {
                        self.table_state.select(Some(selected - 1));
                    }
                }
                KeyCode::Down if !self.is_searching => {
                    let selected = self.table_state.selected().unwrap_or(0);
                    if selected < self.filtered_records.len().saturating_sub(1) {
                        self.table_state.select(Some(selected + 1));
                    }
                }

                KeyCode::Enter => {
                    if self.is_searching {
                        if !self.filtered_records.is_empty() {
                            self.is_searching = false;
                            self.table_state.select(Some(0));
                        }
                    } else {
                        if !self.record_id_input.is_empty() {
                            let _ = self.load_record();
                        } else if !self.filtered_records.is_empty() {
                            let _ = self.load_selected_record();
                        }
                    }
                }
                KeyCode::Esc => return Ok(Some(SelectedApp::None)),
                _ => {}
            }
            return Ok(None);
        }

        // Record editing state
        match key.code {
            KeyCode::Up => {
                if let Some(selected) = self.selected_field {
                    if selected > 0 {
                        self.selected_field = Some(selected - 1);
                        self.edit_table_state.select(Some(selected - 1));
                        self.update_input_value();
                    }
                }
            }
            KeyCode::Down => {
                if let Some(selected) = self.selected_field {
                    if selected < INPUT_FIELDS {
                        self.selected_field = Some(selected + 1);
                        self.edit_table_state.select(Some(selected + 1));
                        self.update_input_value();
                    }
                }
            }
            KeyCode::Enter => {
                self.editing = true;
            }
            KeyCode::Char('s') | KeyCode::Char('S')
                if key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                self.show_confirmation(
                    "Are you sure you want to update this record?".to_string(),
                    ConfirmAction::UpdateRecord,
                );
            }
            KeyCode::Char('e') | KeyCode::Char('E') => {
                self.editing = true;
            }
            KeyCode::Esc => {
                self.back_to_selection();
                return Ok(None);
            }
            _ => {}
        }

        Ok(None)
    }

    /// Clears the error message and resets the error timer.
    fn clear_error(&mut self) {
        self.error_message = None;
        self.error_timer = None;
    }

    /// Sets the error message and starts the error timer.
    ///
    /// # Parameters
    ///
    /// * `message`: The error message to display.
    fn set_error(&mut self, message: String) {
        self.error_message = Some(message);
        self.error_timer = Some(Instant::now());
    }

    /// Clears the success message and resets the success timer.
    fn clear_success(&mut self) {
        self.success_message = None;
        self.success_timer = None;
    }

    /// Checks for success message timeout and clears the message if the timeout has elapsed.
    fn check_success_timeout(&mut self) {
        if let Some(timer) = self.success_timer {
            if timer.elapsed() > Duration::from_secs(5) {
                self.clear_success();
            }
        }
    }

    /// Checks for error message timeout and clears the message if the timeout has elapsed.
    fn check_error_timeout(&mut self) {
        if let Some(timer) = self.error_timer {
            if timer.elapsed() > Duration::from_secs(5) {
                self.clear_error();
            }
        }
    }

    /// Checks both error and success timeouts.
    fn check_timeouts(&mut self) {
        self.check_error_timeout();
        self.check_success_timeout();
    }
}

impl Default for UpdateRecord {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for UpdateRecord {
    /// Handles input events, forwarding them to the internal handler.
    ///
    /// This method implements the `handle_input` method from the `Component` trait.
    /// It calls the internal `handle_input` method to process the key event.
    ///
    /// # Parameters
    ///
    /// * `event`: The `KeyEvent` to handle.
    ///
    /// # Returns
    ///
    /// Returns a `Result<Option<SelectedApp>>`.  See `UpdateRecord::handle_input` for details.
    fn handle_input(&mut self, event: KeyEvent) -> Result<Option<SelectedApp>> {
        match self.handle_input(event)? {
            Some(_) => Ok(Some(SelectedApp::None)),
            None => Ok(None),
        }
    }

    /// Renders the component to the provided frame.
    ///
    /// This method renders the `UpdateRecord` component's UI to the terminal.
    /// It determines the appropriate layout and calls the rendering functions
    /// for each sub-component based on the current state.
    ///
    /// # Parameters
    ///
    /// * `frame`: A mutable reference to a `Frame` for rendering the UI elements.
    fn render(&self, frame: &mut Frame) {
        let area = frame.area();
        frame.render_widget(
            Block::default().style(Style::default().bg(Color::Rgb(16, 16, 28))),
            area,
        );

        match self.update_state {
            UpdateState::SelectingRecord => self.render_record_selection(frame, area),
            UpdateState::EditingRecord => self.render_record_editing(frame, area),
        }

        if self.show_confirmation {
            self.render_confirmation_dialog(frame, area);
        }
    }
}

impl UpdateRecord {
    /// Renders the record selection UI, including the search input, record ID input, and the records table.
    ///
    /// This method renders the UI elements required for selecting a record to be updated.
    /// It includes a header, search input field, record ID input field, a table displaying the records,
    /// and informational messages (error, success, and help).
    ///
    /// # Parameters
    ///
    /// * `frame`: A mutable reference to the `Frame` for rendering.
    /// * `area`: The `Rect` defining the rendering area.
    fn render_record_selection(&self, frame: &mut Frame, area: Rect) {
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Length(3), // Search input
                Constraint::Length(3), // Record ID Input
                Constraint::Min(10),   // Record table
                Constraint::Length(1), // Message
                Constraint::Length(2), // Help
            ])
            .margin(1)
            .split(area);

        // Header
        let header_block = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(Color::Rgb(75, 75, 120)))
            .style(Style::default().bg(Color::Rgb(16, 16, 28)));
        frame.render_widget(header_block, main_layout[0]);

        let title = Paragraph::new("✍️  SELECT RECORD TO UPDATE")
            .style(
                Style::default()
                    .fg(Color::Rgb(230, 230, 250))
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Rgb(16, 16, 28)),
            )
            .alignment(Alignment::Center);
        frame.render_widget(title, main_layout[0]);

        // Search
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
                Style::default().fg(Color::Rgb(250, 250, 110))
            } else {
                Style::default().fg(Color::Rgb(140, 140, 200))
            })
            .style(Style::default().bg(Color::Rgb(26, 26, 36)));

        let search_paragraph = Paragraph::new(self.search_input.clone())
            .style(
                Style::default()
                    .fg(Color::Rgb(220, 220, 240))
                    .bg(Color::Rgb(26, 26, 36)),
            )
            .block(search_block);
        frame.render_widget(search_paragraph, main_layout[1]);

        // ID
        let id_input_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(Span::styled(
                " Record ID ",
                Style::default()
                    .fg(Color::Rgb(230, 230, 250))
                    .add_modifier(Modifier::BOLD),
            ))
            .border_style(if !self.is_searching {
                Style::default().fg(Color::Rgb(250, 250, 110)) // Yellow
            } else {
                Style::default().fg(Color::Rgb(140, 140, 200))
            })
            .style(Style::default().bg(Color::Rgb(26, 26, 36)));

        let id_input_paragraph = Paragraph::new(self.record_id_input.clone())
            .style(
                Style::default()
                    .fg(Color::Rgb(220, 220, 240))
                    .bg(Color::Rgb(26, 26, 36)),
            )
            .block(id_input_block);
        frame.render_widget(id_input_paragraph, main_layout[2]);

        // Record table
        if self.filtered_records.is_empty() {
            let no_records = Paragraph::new(if self.search_input.is_empty() {
                "No records found in database"
            } else {
                "No records match your search criteria"
            })
            .style(Style::default().fg(Color::Rgb(220, 220, 240)))
            .alignment(Alignment::Center);
            frame.render_widget(no_records, main_layout[3]);
        } else {
            let records_rows: Vec<Row> = self
                .filtered_records
                .iter()
                .map(|r| {
                    // Get patient name from patients HashMap
                    let (first_name, last_name) = match self.get_patient(r.patient_id) {
                        Some(patient) => (patient.first_name.clone(), patient.last_name.clone()),
                        None => ("Unknown".to_string(), "Patient".to_string()),
                    };

                    Row::new(vec![
                        r.id.to_string(),
                        first_name,
                        last_name,
                        r.diagnosis.clone(),
                    ])
                    .style(Style::default().fg(Color::Rgb(220, 220, 240)))
                    .height(1)
                    .bottom_margin(0)
                })
                .collect();

            let selected_style = Style::default()
                .fg(Color::Rgb(250, 250, 110))
                .bg(Color::Rgb(40, 40, 60))
                .add_modifier(Modifier::BOLD);

            let header = Row::new(vec!["ID", "First Name", "Last Name", "Diagnosis"])
                .style(
                    Style::default()
                        .fg(Color::Rgb(220, 220, 240))
                        .bg(Color::Rgb(80, 60, 130))
                        .add_modifier(Modifier::BOLD),
                )
                .height(1);

            let widths = [
                Constraint::Percentage(10),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(50),
            ];

            let records_table = Table::new(records_rows, widths)
                .header(header)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .title(format!(" Records ({}) ", self.filtered_records.len()))
                        .border_style(Style::default().fg(Color::Rgb(140, 140, 200)))
                        .style(Style::default().bg(Color::Rgb(26, 26, 36))),
                )
                .column_spacing(2)
                .row_highlight_style(selected_style)
                .highlight_symbol("► ");

            frame.render_stateful_widget(
                records_table,
                main_layout[3],
                &mut self.table_state.clone(),
            );
        }

        // Message
        if let Some(error) = &self.error_message {
            let error_paragraph = Paragraph::new(error.as_str())
                .style(
                    Style::default()
                        .fg(Color::Rgb(255, 100, 100))
                        .add_modifier(Modifier::BOLD),
                )
                .alignment(Alignment::Center);
            frame.render_widget(error_paragraph, main_layout[4]);
        } else if let Some(success) = &self.success_message {
            let success_paragraph = Paragraph::new(success.as_str())
                .style(
                    Style::default()
                        .fg(Color::Rgb(140, 219, 140))
                        .add_modifier(Modifier::BOLD),
                )
                .alignment(Alignment::Center);
            frame.render_widget(success_paragraph, main_layout[4]);
        }

        // Help
        let help_text = if self.is_searching {
            "Type to search | ↓: To results | Esc: Cancel search"
        } else {
            "/ or s: Search | ↑/↓: Navigate records | Enter: Select record | Esc: Back"
        };

        let help_paragraph = Paragraph::new(help_text)
            .style(Style::default().fg(Color::Rgb(220, 220, 240)))
            .alignment(Alignment::Center);
        frame.render_widget(help_paragraph, main_layout[5]);
    }

    /// Renders the record editing UI, including the record data, input field, and editing controls.
    ///
    /// This method renders the UI elements required for editing a medical record.
    /// It includes a header, a table displaying the record's fields, an input field for editing,
    /// and informational messages (error, success, and help).
    ///
    /// # Parameters
    ///
    /// * `frame`: A mutable reference to the `Frame` for rendering.
    /// * `area`: The `Rect` defining the rendering area.
    fn render_record_editing(&self, frame: &mut Frame, area: Rect) {
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(14),   // Table
                Constraint::Length(3), // Input field
                Constraint::Length(1), // Error/Success message
                Constraint::Length(2), // Help text
            ])
            .margin(1)
            .split(area);

        // Header
        let header_block = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(Color::Rgb(75, 75, 120)))
            .style(Style::default().bg(Color::Rgb(16, 16, 28)));
        frame.render_widget(header_block, main_layout[0]);

        let title_text = if self.editing {
            "✍️  EDITING RECORD"
        } else {
            "✍️  UPDATE RECORD"
        };

        let title = Paragraph::new(title_text)
            .style(
                Style::default()
                    .fg(Color::Rgb(230, 230, 250))
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Rgb(16, 16, 28)),
            )
            .alignment(Alignment::Center);
        frame.render_widget(title, main_layout[0]);

        // Record data table
        let id_str = self.record.id.to_string();
        let patient_id_str = self.record.patient_id.to_string();
        let nurse_notes_str = self.record.nurse_notes.clone().unwrap_or_default();
        let prescription_str = self.record.prescription.clone().unwrap_or_default();

        let table_items = vec![
            Row::new(vec!["ID", &id_str])
                .style(Style::default().fg(Color::Rgb(220, 220, 240)))
                .height(1)
                .bottom_margin(0),
            Row::new(vec!["Patient ID", &patient_id_str])
                .style(Style::default().fg(Color::Rgb(220, 220, 240)))
                .height(1)
                .bottom_margin(0),
            Row::new(vec!["Doctor's Notes", &self.record.doctor_notes])
                .style(Style::default().fg(Color::Rgb(220, 220, 240)))
                .height(1)
                .bottom_margin(0),
            Row::new(vec!["Nurse's Notes", &nurse_notes_str])
                .style(Style::default().fg(Color::Rgb(220, 220, 240)))
                .height(1)
                .bottom_margin(0),
            Row::new(vec!["Diagnosis", &self.record.diagnosis])
                .style(Style::default().fg(Color::Rgb(220, 220, 240)))
                .height(1)
                .bottom_margin(0),
            Row::new(vec!["Prescription", &prescription_str])
                .style(Style::default().fg(Color::Rgb(220, 220, 240)))
                .height(1)
                .bottom_margin(0),
        ];

        let selected_style = Style::default()
            .fg(Color::Rgb(250, 250, 110))
            .bg(Color::Rgb(40, 40, 60))
            .add_modifier(Modifier::BOLD);

        let header = Row::new(vec!["Field", "Value"])
            .style(
                Style::default()
                    .fg(Color::Rgb(220, 220, 240))
                    .bg(Color::Rgb(80, 60, 130))
                    .add_modifier(Modifier::BOLD),
            )
            .height(1);

        let widths = [Constraint::Percentage(30), Constraint::Percentage(70)];

        let table = Table::new(table_items, widths)
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(" Record Data ")
                    .border_style(Style::default().fg(Color::Rgb(140, 140, 200)))
                    .style(Style::default().bg(Color::Rgb(26, 26, 36))),
            )
            .column_spacing(2)
            .row_highlight_style(selected_style)
            .highlight_symbol("► ");

        frame.render_stateful_widget(table, main_layout[1], &mut self.edit_table_state.clone());

        // Input
        let input_label = match self.selected_field {
            Some(ID_INPUT) => "ID",
            Some(PATIENT_ID_INPUT) => "Patient ID",
            Some(DOCTOR_NOTES_INPUT) => "Doctor's Notes",
            Some(NURSE_NOTES_INPUT) => "Nurse's Notes",
            Some(DIAGNOSIS_INPUT) => "Diagnosis",
            Some(PRESCRIPTION_INPUT) => "Prescription",
            _ => "Field",
        };

        let input_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(format!(
                " {} {} ",
                if self.editing { "Editing" } else { "Selected" },
                input_label
            ))
            .border_style(if self.editing {
                Style::default().fg(Color::Rgb(140, 219, 140)) // Green
            } else {
                Style::default().fg(Color::Rgb(140, 140, 200))
            })
            .style(Style::default().bg(Color::Rgb(26, 26, 36)));

        let input_paragraph = Paragraph::new(self.input_value.clone())
            .style(
                Style::default()
                    .fg(Color::Rgb(220, 220, 240))
                    .bg(Color::Rgb(26, 26, 36)),
            )
            .block(input_block);
        frame.render_widget(input_paragraph, main_layout[2]);

        // Message
        if let Some(error) = &self.error_message {
            let error_paragraph = Paragraph::new(error.as_str())
                .style(
                    Style::default()
                        .fg(Color::Rgb(255, 100, 100))
                        .add_modifier(Modifier::BOLD),
                )
                .alignment(Alignment::Center);
            frame.render_widget(error_paragraph, main_layout[3]);
        } else if let Some(success) = &self.success_message {
            let success_paragraph = Paragraph::new(success.as_str())
                .style(
                    Style::default()
                        .fg(Color::Rgb(140, 219, 140))
                        .add_modifier(Modifier::BOLD),
                )
                .alignment(Alignment::Center);
            frame.render_widget(success_paragraph, main_layout[3]);
        }

        // Help
        let help_text = if self.editing {
            "Enter: Save Changes | Esc: Cancel Editing"
        } else {
            "↑/↓: Navigate | E: Edit Field | Ctrl+S: Save Record | Esc: Back"
        };

        let help_paragraph = Paragraph::new(help_text)
            .style(Style::default().fg(Color::Rgb(220, 220, 240)))
            .alignment(Alignment::Center);
        frame.render_widget(help_paragraph, main_layout[4]);
    }

    /// Renders the confirmation dialog, allowing the user to confirm an action.
    ///
    /// This method renders a confirmation dialog with "Yes" and "No" buttons,
    /// allowing the user to confirm or cancel the action.
    ///
    /// # Parameters
    ///
    /// * `frame`: A mutable reference to the `Frame` for rendering.
    /// * `area`: The `Rect` defining the rendering area.
    fn render_confirmation_dialog(&self, frame: &mut Frame, area: Rect) {
        let dialog_width = 50;
        let dialog_height = 8;
        let dialog_area = Rect::new(
            (area.width.saturating_sub(dialog_width)) / 2,
            (area.height.saturating_sub(dialog_height)) / 2,
            dialog_width,
            dialog_height,
        );

        frame.render_widget(Clear, dialog_area);

        let dialog_block = Block::default()
            .title(" Update Record ")
            .title_style(
                Style::default()
                    .fg(Color::Rgb(230, 230, 250))
                    .add_modifier(Modifier::BOLD),
            )
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Rgb(140, 140, 200)))
            .style(Style::default().bg(Color::Rgb(30, 30, 46)));

        frame.render_widget(dialog_block.clone(), dialog_area);

        let inner_area = dialog_block.inner(dialog_area);
        let content_layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Length(2), Constraint::Length(2)])
            .split(inner_area);

        let message = Paragraph::new(self.confirmation_message.as_str())
            .style(Style::default().fg(Color::Rgb(220, 220, 240)))
            .add_modifier(Modifier::BOLD)
            .alignment(Alignment::Center);
        frame.render_widget(message, content_layout[0]);

        let buttons_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(content_layout[1]);

        let yes_style = if self.confirmation_selected == 0 {
            Style::default()
                .fg(Color::Rgb(140, 219, 140))
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Rgb(180, 180, 200))
        };
        let no_style = if self.confirmation_selected == 1 {
            Style::default()
                .fg(Color::Rgb(255, 100, 100))
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Rgb(180, 180, 200))
        };

        let yes_text = if self.confirmation_selected == 0 {
            "► Yes ◄"
        } else {
            "  Yes  "
        };
        let no_text = if self.confirmation_selected == 1 {
            "► No ◄"
        } else {
            "  No  "
        };

        let yes_button = Paragraph::new(yes_text)
            .style(yes_style)
            .alignment(Alignment::Center);
        let no_button = Paragraph::new(no_text)
            .style(no_style)
            .alignment(Alignment::Center);

        frame.render_widget(yes_button, buttons_layout[0]);
        frame.render_widget(no_button, buttons_layout[1]);
    }
}
