//! Update Staff component for the Hospital application.
//!
//! Provides a UI for selecting and updating staff details. It has two main states:
//! 1. Staff Selection - Shows a table of staff and an ID input field
//! 2. Staff Editing - Shows staff details in a table with an editor below

use crate::app::SelectedApp;
use crate::components::Component;
use crate::db;
use crate::models::{StaffMember, StaffRole};
use crate::tui::Frame;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{prelude::*, widgets::*};
use std::time::{Duration, Instant};

/// Enum for confirmation dialog actions.
enum ConfirmAction {
    UpdateStaff,
}

/// Component states to manage the UI flow.
enum UpdateState {
    /// Initial state showing a list of staff to select from.
    SelectingStaff,
    /// After selection, shows editable fields for the staff member.
    EditingStaff,
}

/// Component to update an existing staff member's information.
pub struct UpdateStaff {
    all_staff: Vec<StaffMember>,      // All staff for selection table
    filtered_staff: Vec<StaffMember>, // Filtered staff based on search
    search_input: String,             // Search input text
    is_searching: bool,               // Whether user is currently typing in search box
    table_state: TableState,          // Track which row in the selection table is selected
    update_state: UpdateState,        // Current component state
    staff_id_input: String,           // Input for staff ID
    staff: StaffMember,               // The staff data being updated
    loaded: bool,                     // Flag: Has the initial staff data been loaded?
    selected_field: Option<usize>,    // Currently selected field
    edit_table_state: TableState,     // State for the editing table
    input_value: String,              // Current value being edited
    editing: bool,                    // Whether we're currently editing a value
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
const NAME_INPUT: usize = 1;
const ROLE_INPUT: usize = 2;
const PHONE_INPUT: usize = 3;
const EMAIL_INPUT: usize = 4;
const ADDRESS_INPUT: usize = 5;
const INPUT_FIELDS: usize = 5; // Corrected: This should be 5, as there are 6 fields (0-5)

impl UpdateStaff {
    /// Creates a new `UpdateStaff` component.
    ///
    /// Initializes the component with the staff selection view.
    pub fn new() -> Self {
        let mut selection_state = TableState::default();
        selection_state.select(Some(0)); // Start with the first row selected

        let mut edit_table_state = TableState::default();
        edit_table_state.select(Some(0));

        Self {
            all_staff: Vec::new(),      // Initialize as empty, data loaded by fetch_staff()
            filtered_staff: Vec::new(), // Initialize as empty
            search_input: String::new(),
            is_searching: false,
            table_state: selection_state,
            update_state: UpdateState::SelectingStaff,
            staff_id_input: String::new(),
            staff: StaffMember {
                // Correct: Ensure staff has a default state
                id: 0,
                name: String::new(),
                role: StaffRole::Doctor, // Or any other default
                phone_number: String::new(),
                email: None,
                address: String::new(),
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

    /// Fetches all staff from the database.
    ///
    /// This method should be called when the component is first displayed
    /// or when the staff list needs to be refreshed.
    pub fn fetch_staff(&mut self) -> Result<()> {
        self.all_staff = db::get_all_staff()?;
        self.filter_staff(); // Apply filtering after loading.
        Ok(())
    }

    /// Filters staff based on the search term.
    fn filter_staff(&mut self) {
        if self.search_input.is_empty() {
            // If search is empty, show all staff
            self.filtered_staff = self.all_staff.clone();
        } else {
            let search_term = self.search_input.to_lowercase();
            self.filtered_staff = self
                .all_staff
                .iter()
                .filter(|s| {
                    // Case-insensitive search in multiple fields
                    s.name.to_lowercase().contains(&search_term)
                        || s.id.to_string().contains(&search_term)
                        || s.phone_number.to_lowercase().contains(&search_term)
                })
                .cloned()
                .collect();
        }

        // Reset selection if it's now out of bounds
        if let Some(selected) = self.table_state.selected() {
            if selected >= self.filtered_staff.len() && !self.filtered_staff.is_empty() {
                self.table_state.select(Some(0));
            }
        }
    }

    /// Loads staff data by ID from the database.
    ///
    /// Transitions to editing mode if the staff member is found.
    fn load_staff_by_id(&mut self, staff_id: i64) -> Result<()> {
        match db::get_staff(staff_id) {
            Ok(staff) => {
                self.staff = staff;
                self.loaded = true;
                self.update_state = UpdateState::EditingStaff;
                self.update_input_value();
                Ok(())
            }
            Err(_) => {
                // Show a user-friendly error message
                self.set_error(format!("Staff with ID {} doesn't exist", staff_id));
                Err(anyhow::anyhow!("Staff not found"))
            }
        }
    }

    /// Loads staff data based on the ID input field.
    fn load_staff(&mut self) -> Result<()> {
        if !self.loaded {
            if let Ok(staff_id) = self.staff_id_input.parse::<i64>() {
                match self.load_staff_by_id(staff_id) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e),
                }
            } else {
                self.set_error("Invalid Staff ID format.".to_string());
                Err(anyhow::anyhow!("Invalid Staff ID format"))
            }
        } else {
            Ok(())
        }
    }

    /// Loads the currently selected staff member from the table.
    fn load_selected_staff(&mut self) -> Result<()> {
        if let Some(selected) = self.table_state.selected() {
            if selected < self.filtered_staff.len() {
                let staff_id = self.filtered_staff[selected].id;
                self.staff_id_input = staff_id.to_string();
                return self.load_staff_by_id(staff_id);
            }
        }
        self.set_error("No staff selected".to_string());
        Err(anyhow::anyhow!("No staff selected"))
    }

    /// Updates the input value based on the currently selected field.
    fn update_input_value(&mut self) {
        if !self.loaded {
            self.input_value = self.staff_id_input.clone();
            return;
        }

        if let Some(field_index) = self.selected_field {
            self.input_value = match field_index {
                ID_INPUT => self.staff.id.to_string(),
                NAME_INPUT => self.staff.name.clone(),
                ROLE_INPUT => match self.staff.role {
                    StaffRole::Doctor => "Doctor".to_string(),
                    StaffRole::Nurse => "Nurse".to_string(),
                    StaffRole::Admin => "Admin".to_string(),
                    StaffRole::Technician => "Technician".to_string(),
                },
                PHONE_INPUT => self.staff.phone_number.clone(),
                EMAIL_INPUT => self.staff.email.clone().unwrap_or_default(),
                ADDRESS_INPUT => self.staff.address.clone(),
                _ => String::new(),
            };
        }
    }

    /// Applies the edited value to the selected field in the staff data.
    fn apply_edited_value(&mut self) {
        if !self.editing || !self.loaded {
            return;
        }

        if let Some(field_index) = self.selected_field {
            match field_index {
                NAME_INPUT => self.staff.name = self.input_value.clone(),
                ROLE_INPUT => {
                    self.staff.role = match self.input_value.to_lowercase().as_str() {
                        "doctor" | "d" => StaffRole::Doctor,
                        "nurse" | "n" => StaffRole::Nurse,
                        "admin" | "a" => StaffRole::Admin,
                        "technician" | "t" => StaffRole::Technician,
                        _ => StaffRole::Doctor, // Default to Doctor
                    }
                }
                PHONE_INPUT => self.staff.phone_number = self.input_value.clone(),
                EMAIL_INPUT => self.staff.email = Some(self.input_value.clone()),
                ADDRESS_INPUT => self.staff.address = self.input_value.clone(),
                _ => {}
            }
        }
        self.editing = false;
    }

    /// Shows a confirmation dialog before performing an action.
    fn show_confirmation(&mut self, message: String, action: ConfirmAction) {
        self.show_confirmation = true;
        self.confirmation_message = message;
        self.confirmed_action = Some(action);
        self.confirmation_selected = 0; // Default Yes
    }

    /// Updates the staff member in the database.
    fn update_staff(&mut self) -> Result<()> {
        match db::update_staff_member(&self.staff) {
            Ok(_) => {
                self.success_message = Some("Staff updated successfully!".to_string());
                self.success_timer = Some(Instant::now());

                // Refresh the staff list
                if let Ok(staff) = db::get_all_staff() {
                    self.all_staff = staff.clone();
                    self.filtered_staff = staff;
                    self.filter_staff(); // Re-apply any active search filter
                }

                Ok(())
            }
            Err(e) => {
                self.set_error(format!("Database error: {}", e));
                Err(e)
            }
        }
    }

    /// Resets to staff selection state.
    fn back_to_selection(&mut self) {
        self.update_state = UpdateState::SelectingStaff;
        self.loaded = false;
        self.staff_id_input = String::new();
        self.editing = false;
        self.clear_error();
        self.clear_success();
    }

    /// Handles input events for the component.
    fn handle_input(&mut self, key: KeyEvent) -> Result<Option<SelectedApp>> {
        self.check_timeouts();

        // Handle confirmation dialog if it's shown
        if self.show_confirmation {
            match key.code {
                KeyCode::Left | KeyCode::Right => {
                    self.confirmation_selected = 1 - self.confirmation_selected;
                }
                KeyCode::Enter => {
                    if self.confirmation_selected == 0 {
                        if let Some(ConfirmAction::UpdateStaff) = self.confirmed_action.take() {
                            let _ = self.update_staff();
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

        // If we're editing, handle the input differently
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
                    // Cancel editing
                    self.editing = false;
                    self.update_input_value();
                }
                _ => {}
            }
            return Ok(None);
        }

        // Staff selection state
        if matches!(self.update_state, UpdateState::SelectingStaff) {
            match key.code {
                // Search mode handling
                KeyCode::Char(c) if self.is_searching => {
                    self.search_input.push(c);
                    self.filter_staff();
                    self.clear_error();
                }
                KeyCode::Backspace if self.is_searching => {
                    self.search_input.pop();
                    self.filter_staff();
                    self.clear_error();
                }
                KeyCode::Down if self.is_searching && !self.filtered_staff.is_empty() => {
                    // Move from search to results
                    self.is_searching = false;
                    self.table_state.select(Some(0));
                }
                KeyCode::Esc if self.is_searching => {
                    // Cancel search
                    self.is_searching = false;
                    self.search_input.clear();
                    self.filter_staff();
                }

                // Search activation keys
                KeyCode::Char('/') | KeyCode::Char('s') | KeyCode::Char('S')
                    if !self.is_searching =>
                {
                    self.is_searching = true;
                }

                // ID input handling (when not searching)
                KeyCode::Char(c) if !self.is_searching => {
                    self.staff_id_input.push(c);
                    self.input_value = self.staff_id_input.clone();
                    self.clear_error();
                }
                KeyCode::Backspace if !self.is_searching => {
                    self.staff_id_input.pop();
                    self.input_value = self.staff_id_input.clone();
                    self.clear_error();
                }

                // Navigation in results (when not searching)
                KeyCode::Up if !self.is_searching => {
                    let selected = self.table_state.selected().unwrap_or(0);
                    if selected > 0 {
                        self.table_state.select(Some(selected - 1));
                    }
                }
                KeyCode::Down if !self.is_searching => {
                    let selected = self.table_state.selected().unwrap_or(0);
                    if selected < self.filtered_staff.len().saturating_sub(1) {
                        self.table_state.select(Some(selected + 1));
                    }
                }

                // Selection handling
                KeyCode::Enter => {
                    if self.is_searching {
                        // If in search, pressing enter moves to results
                        if !self.filtered_staff.is_empty() {
                            self.is_searching = false;
                            self.table_state.select(Some(0));
                        }
                    } else {
                        // Try loading the staff from ID input or selected row
                        if !self.staff_id_input.is_empty() {
                            let _ = self.load_staff();
                        } else if !self.filtered_staff.is_empty() {
                            let _ = self.load_selected_staff();
                        }
                    }
                }

                // Exit
                KeyCode::Esc => {
                    return Ok(Some(SelectedApp::None));
                }
                _ => {}
            }
            return Ok(None);
        }

        // Staff editing state
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
                // Start editing the selected field
                self.editing = true;
            }
            KeyCode::Char('s') | KeyCode::Char('S')
                if key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                // Save (update) staff with confirmation
                self.show_confirmation(
                    "Are you sure you want to update this staff member? (y/n)".to_string(),
                    ConfirmAction::UpdateStaff,
                );
            }
            KeyCode::Char('e') | KeyCode::Char('E') => {
                // Edit the selected field
                self.editing = true;
            }
            KeyCode::Esc => {
                // Go back to staff selection
                self.back_to_selection();
                return Ok(None);
            }
            _ => {}
        }

        Ok(None)
    }

    /// Clears error message and timer.
    fn clear_error(&mut self) {
        self.error_message = None;
        self.error_timer = None;
    }

    /// Sets an error message with a timeout.
    fn set_error(&mut self, message: String) {
        self.error_message = Some(message);
        self.error_timer = Some(Instant::now());
    }

    /// Clears success message and timer.
    fn clear_success(&mut self) {
        self.success_message = None;
        self.success_timer = None;
    }

    /// Checks if the success message should be cleared due to timeout.
    fn check_success_timeout(&mut self) {
        if let Some(timer) = self.success_timer {
            if timer.elapsed() > Duration::from_secs(5) {
                self.clear_success();
            }
        }
    }

    /// Checks if the error message should be cleared due to timeout.
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

impl Default for UpdateStaff {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for UpdateStaff {
    /// Handles user input events for the `UpdateStaff` component.
    ///
    /// # Arguments
    ///
    /// * `event` - The key event to be handled.
    ///
    /// # Returns
    ///
    /// Returns `Result<Option<SelectedApp>>` indicating the outcome of the input handling:
    ///
    /// * `Ok(Some(SelectedApp::None))` if the component should switch back to the main app.
    /// * `Ok(None)` if the component should continue processing input.
    /// * `Err(_)` if an error occurred during input handling.
    fn handle_input(&mut self, event: KeyEvent) -> Result<Option<SelectedApp>> {
        match self.handle_input(event)? {
            Some(_) => Ok(Some(crate::app::SelectedApp::None)),
            None => Ok(None),
        }
    }
    /// Renders the `UpdateStaff` component onto the given frame.
    ///
    /// # Arguments
    ///
    /// * `frame` - The frame to render the component onto.
    fn render(&self, frame: &mut Frame) {
        let area = frame.area();
        frame.render_widget(
            Block::default().style(Style::default().bg(Color::Rgb(16, 16, 28))),
            area,
        );

        match self.update_state {
            UpdateState::SelectingStaff => self.render_staff_selection(frame, area),
            UpdateState::EditingStaff => self.render_staff_editing(frame, area),
        }

        // Render confirmation dialog if needed
        if self.show_confirmation {
            self.render_confirmation_dialog(frame, area);
        }
    }
}

impl UpdateStaff {
    /// Renders the staff selection screen with table of staff.
    fn render_staff_selection(&self, frame: &mut Frame, area: Rect) {
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Length(3), // Search input
                Constraint::Length(3), // Staff ID Input
                Constraint::Min(10),   // Staff selection table
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

        let title = Paragraph::new("✍️  SELECT STAFF TO UPDATE")
            .style(
                Style::default()
                    .fg(Color::Rgb(230, 230, 250))
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Rgb(16, 16, 28)),
            )
            .alignment(Alignment::Center);
        frame.render_widget(title, main_layout[0]);

        // Search input field
        let search_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(Span::styled(
                " Search Staff ",
                Style::default()
                    .fg(Color::Rgb(230, 230, 250))
                    .add_modifier(Modifier::BOLD),
            ))
            .border_style(if self.is_searching {
                Style::default().fg(Color::Rgb(250, 250, 110)) // Yellow when active
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

        // ID input field
        let id_input_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(Span::styled(
                " Staff ID ",
                Style::default()
                    .fg(Color::Rgb(230, 230, 250))
                    .add_modifier(Modifier::BOLD),
            ))
            .border_style(if !self.is_searching {
                Style::default().fg(Color::Rgb(250, 250, 110)) // Yellow when active
            } else {
                Style::default().fg(Color::Rgb(140, 140, 200))
            })
            .style(Style::default().bg(Color::Rgb(26, 26, 36)));

        let id_input_paragraph = Paragraph::new(self.staff_id_input.clone())
            .style(
                Style::default()
                    .fg(Color::Rgb(220, 220, 240))
                    .bg(Color::Rgb(26, 26, 36)),
            )
            .block(id_input_block);
        frame.render_widget(id_input_paragraph, main_layout[2]);

        // Staff selection table
        if self.filtered_staff.is_empty() {
            let no_staff = Paragraph::new(if self.search_input.is_empty() {
                "No staff found in database"
            } else {
                "No staff match your search criteria"
            })
            .style(Style::default().fg(Color::Rgb(220, 220, 240)))
            .alignment(Alignment::Center);
            frame.render_widget(no_staff, main_layout[3]);
        } else {
            let staff_rows: Vec<Row> = self
                .filtered_staff
                .iter()
                .map(|s| {
                    Row::new(vec![
                        s.id.to_string(),
                        s.name.clone(),
                        s.phone_number.clone(),
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

            let header = Row::new(vec!["ID", "Name", "Phone"])
                .style(
                    Style::default()
                        .fg(Color::Rgb(220, 220, 240))
                        .bg(Color::Rgb(80, 60, 130))
                        .add_modifier(Modifier::BOLD),
                )
                .height(1);

            let widths = [
                Constraint::Percentage(15),
                Constraint::Percentage(40),
                Constraint::Percentage(45),
            ];

            let staff_table = Table::new(staff_rows, widths)
                .header(header)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .title(format!(" Staff ({}) ", self.filtered_staff.len()))
                        .border_style(Style::default().fg(Color::Rgb(140, 140, 200)))
                        .style(Style::default().bg(Color::Rgb(26, 26, 36))),
                )
                .column_spacing(2)
                .row_highlight_style(selected_style)
                .highlight_symbol("► ");

            let mut table_state_copy = self.table_state.clone();
            frame.render_stateful_widget(staff_table, main_layout[3], &mut table_state_copy);
        }

        // Error or success message
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

        // Help text
        let help_text = if self.is_searching {
            "Type to search | ↓: To results | Esc: Cancel search"
        } else {
            "/ or s: Search | ↑/↓: Navigate staff | Enter: Select staff | Esc: Back"
        };

        let help_paragraph = Paragraph::new(help_text)
            .style(Style::default().fg(Color::Rgb(220, 220, 240)))
            .alignment(Alignment::Center);
        frame.render_widget(help_paragraph, main_layout[5]);
    }

    /// Renders the staff editing screen with data table and input field.
    fn render_staff_editing(&self, frame: &mut Frame, area: Rect) {
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(10),   // Table
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
            "✍️  EDITING STAFF"
        } else {
            "✍️  UPDATE STAFF"
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

        // Staff data table
        let id_str = self.staff.id.to_string();
        let role_str = match self.staff.role {
            StaffRole::Doctor => "Doctor",
            StaffRole::Nurse => "Nurse",
            StaffRole::Admin => "Admin",
            StaffRole::Technician => "Technician",
        };
        let email_str = self.staff.email.clone().unwrap_or_default();

        let table_items = vec![
            Row::new(vec!["ID", &id_str])
                .style(Style::default().fg(Color::Rgb(220, 220, 240)))
                .height(1)
                .bottom_margin(0),
            Row::new(vec!["Name", &self.staff.name])
                .style(Style::default().fg(Color::Rgb(220, 220, 240)))
                .height(1)
                .bottom_margin(0),
            Row::new(vec!["Role", &role_str])
                .style(Style::default().fg(Color::Rgb(220, 220, 240)))
                .height(1)
                .bottom_margin(0),
            Row::new(vec!["Phone", &self.staff.phone_number])
                .style(Style::default().fg(Color::Rgb(220, 220, 240)))
                .height(1)
                .bottom_margin(0),
            Row::new(vec!["Email", &email_str])
                .style(Style::default().fg(Color::Rgb(220, 220, 240)))
                .height(1)
                .bottom_margin(0),
            Row::new(vec!["Address", &self.staff.address])
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
                    .title(" Staff Data ")
                    .border_style(Style::default().fg(Color::Rgb(140, 140, 200)))
                    .style(Style::default().bg(Color::Rgb(26, 26, 36))),
            )
            .column_spacing(2)
            .row_highlight_style(selected_style)
            .highlight_symbol("► ");

        let mut edit_table_state_copy = self.edit_table_state.clone();
        frame.render_stateful_widget(table, main_layout[1], &mut edit_table_state_copy);

        // Input field for editing
        let input_label = match self.selected_field {
            Some(ID_INPUT) => "ID",
            Some(NAME_INPUT) => "Name",
            Some(ROLE_INPUT) => "Role",
            Some(PHONE_INPUT) => "Phone",
            Some(EMAIL_INPUT) => "Email",
            Some(ADDRESS_INPUT) => "Address",
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
                Style::default().fg(Color::Rgb(140, 219, 140)) // Green when editing
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

        // Error or success message
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

        // Help text
        let help_text = if self.editing {
            "Enter: Save Changes | Esc: Cancel Editing"
        } else {
            "↑/↓: Navigate | E: Edit Field | Ctrl+S: Save Staff | Esc: Back"
        };

        let help_paragraph = Paragraph::new(help_text)
            .style(Style::default().fg(Color::Rgb(220, 220, 240)))
            .alignment(Alignment::Center);
        frame.render_widget(help_paragraph, main_layout[4]);
    }

    /// Renders the confirmation dialog for actions like save.
    fn render_confirmation_dialog(&self, frame: &mut Frame, area: Rect) {
        let dialog_width = 50;
        let dialog_height = 8;
        let dialog_area = Rect::new(
            (area.width.saturating_sub(dialog_width)) / 2,
            (area.height.saturating_sub(dialog_height)) / 2,
            dialog_width,
            dialog_height,
        );

        // Clear just the dialog area
        frame.render_widget(Clear, dialog_area);

        let dialog_block = Block::default()
            .title(" Update Patient ")
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

        let message = Paragraph::new(self.confirmation_message.as_str()) // Use the message
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
