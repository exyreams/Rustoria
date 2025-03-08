//! `DeleteRecord` module for the Hospital application.
//!
//! This module provides the `DeleteRecord` component, which allows users to view, filter, select, and delete medical records from the database.  It integrates with the database for record retrieval and deletion, and presents a TUI (Text User Interface) for user interaction, including search functionality and confirmation dialogs. The primary type exposed is `DeleteRecord`.

use crate::app::SelectedApp;
use crate::components::Component;
use crate::db;
use crate::models::MedicalRecord;
use crate::tui::Frame;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, Cell, Clear, Paragraph, Row, Table, TableState},
};
use std::time::{Duration, Instant};

/// A component for managing the deletion of medical records in the hospital application.
///
/// This struct encapsulates the state and logic required to display a list of medical records,
/// allow users to select records for deletion, search and filter records based on user input,
/// and provide a confirmation dialog before performing the deletion. It interacts with the
/// database to fetch and delete records.  The component's lifecycle involves fetching records,
/// displaying them in a table, handling user input for selection and search, and managing
/// a confirmation dialog.
pub struct DeleteRecord {
    records: Vec<MedicalRecord>,          // All records
    filtered_records: Vec<MedicalRecord>, // Filtered records
    selected_record_ids: Vec<i64>,        // IDs of selected records
    search_input: String,                 // Search input
    is_searching: bool,                   // Search mode flag
    table_state: TableState,              // Table state
    show_confirmation: bool,              // Confirmation dialog flag
    confirmation_selected: usize,         // Confirmation dialog selection
    error_message: Option<String>,
    error_timer: Option<Instant>,
    success_message: Option<String>,
    success_timer: Option<Instant>,
}

impl DeleteRecord {
    /// Creates a new `DeleteRecord` component.
    ///
    /// This function initializes a new instance of the `DeleteRecord` component with default values.
    /// It sets up empty vectors for records and filtered records, clears the search input,
    /// initializes the table state, and sets the confirmation dialog flag to false.
    ///
    /// # Returns
    ///
    /// A new `DeleteRecord` instance.
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
            filtered_records: Vec::new(),
            selected_record_ids: Vec::new(),
            search_input: String::new(),
            is_searching: false,
            table_state: TableState::default(),
            show_confirmation: false,
            confirmation_selected: 1, // Default to "No"
            error_message: None,
            error_timer: None,
            success_message: None,
            success_timer: None,
        }
    }

    /// Fetches all medical records from the database and filters them.
    ///
    /// This method retrieves all medical records from the database using `db::get_all_medical_records()`.
    /// After successfully fetching the records, it calls `filter_records()` to apply any existing search
    /// filters. The table's selection state is then updated based on the presence of records.
    ///
    /// # Errors
    ///
    /// Returns an `anyhow::Result` which will be an error if there is a problem retrieving records
    /// from the database.
    pub fn fetch_records(&mut self) -> Result<()> {
        self.records = db::get_all_medical_records()?;
        self.filter_records();

        if !self.filtered_records.is_empty() {
            self.table_state.select(Some(0));
        } else {
            self.table_state.select(None);
        }

        Ok(())
    }

    /// Filters the medical records based on the current search input.
    ///
    /// This function filters the `records` field, populating the `filtered_records` field with the
    /// records that match the current `search_input`. If the search input is empty, all records are
    /// considered a match. The search is case-insensitive, and filters are applied to patient ID,
    /// doctor notes, and diagnosis fields of the `MedicalRecord` struct. It also updates the selection
    /// state of the table to prevent out-of-bounds selection, clearing it if the filtered list is empty.
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

        // Update selection state
        self.selected_record_ids.clear();
        if let Some(selected) = self.table_state.selected() {
            if selected >= self.filtered_records.len() && !self.filtered_records.is_empty() {
                self.table_state.select(Some(0));
            } else if self.filtered_records.is_empty() {
                self.table_state.select(None)
            }
        }
    }

    /// Handles key input events to manage the component's behavior.
    ///
    /// This method processes keyboard input, handling actions like searching, record selection,
    /// deletion confirmation, and navigation. It manages different states of the component
    /// including searching and confirmation dialogs.  It also interacts with the database to
    /// perform record deletion based on user confirmation and displays success or error
    /// messages to the user.
    ///
    /// # Arguments
    ///
    /// * `key`: A `KeyEvent` representing the user's keyboard input.
    ///
    /// # Returns
    ///
    /// Returns a `Result<Option<SelectedApp>>`.  `Some(SelectedApp::None)` is returned to
    /// indicate that no specific app selection change is needed.
    fn handle_input(&mut self, key: KeyEvent) -> Result<Option<SelectedApp>> {
        self.check_timeouts();

        if self.show_confirmation {
            match key.code {
                KeyCode::Left | KeyCode::Right => {
                    self.confirmation_selected = 1 - self.confirmation_selected;
                }
                KeyCode::Enter => {
                    if self.confirmation_selected == 0 {
                        // Yes
                        let mut deleted_count = 0;
                        let mut error_occurred = false;

                        for record_id in &self.selected_record_ids {
                            match db::delete_medical_record(*record_id) {
                                Ok(_) => deleted_count += 1,
                                Err(_) => {
                                    error_occurred = true;
                                    break;
                                }
                            }
                        }
                        self.selected_record_ids.clear();

                        if error_occurred {
                            self.set_error(format!(
                                "Error during deletion. {} records deleted successfully.",
                                deleted_count
                            ));
                        } else if deleted_count > 0 {
                            self.success_message = Some(format!(
                                "{} record{} deleted successfully!",
                                deleted_count,
                                if deleted_count == 1 { "" } else { "s" }
                            ));
                            self.success_timer = Some(Instant::now());
                        } else {
                            self.set_error("No records were selected for deletion.".to_string());
                        }

                        if let Ok(records) = db::get_all_medical_records() {
                            self.records = records;
                            self.filter_records();

                            if self.filtered_records.is_empty() {
                                self.table_state.select(None);
                            } else if let Some(selected) = self.table_state.selected() {
                                if selected >= self.filtered_records.len() {
                                    self.table_state.select(Some(
                                        self.filtered_records.len().saturating_sub(1),
                                    ));
                                }
                            }
                        }

                        self.show_confirmation = false;
                    } else {
                        // No
                        self.show_confirmation = false;
                    }
                }
                KeyCode::Esc => {
                    self.show_confirmation = false;
                }
                _ => {}
            }
        } else if self.is_searching {
            match key.code {
                KeyCode::Char(c) => {
                    self.search_input.push(c);
                    self.filter_records();
                }
                KeyCode::Backspace => {
                    self.search_input.pop();
                    self.filter_records();
                }
                KeyCode::Enter | KeyCode::Down => {
                    if !self.filtered_records.is_empty() {
                        self.is_searching = false;
                        self.table_state.select(Some(0));
                    }
                }
                KeyCode::Esc => {
                    self.is_searching = false;
                }
                _ => {}
            }
        } else {
            match key.code {
                KeyCode::Char('/') | KeyCode::Char('s') | KeyCode::Char('S') => {
                    self.is_searching = true;
                    return Ok(None);
                }
                KeyCode::Down => {
                    if !self.filtered_records.is_empty() {
                        let next = match self.table_state.selected() {
                            Some(i) => {
                                if i >= self.filtered_records.len() - 1 {
                                    0
                                } else {
                                    i + 1
                                }
                            }
                            None => 0,
                        };
                        self.table_state.select(Some(next));
                    }
                }
                KeyCode::Up => {
                    if !self.filtered_records.is_empty() {
                        let next = match self.table_state.selected() {
                            Some(i) => {
                                if i == 0 {
                                    self.filtered_records.len() - 1
                                } else {
                                    i - 1
                                }
                            }
                            None => 0,
                        };
                        self.table_state.select(Some(next));
                    }
                }
                KeyCode::Char(' ') => {
                    if let Some(selected) = self.table_state.selected() {
                        if selected < self.filtered_records.len() {
                            let record_id = self.filtered_records[selected].id;
                            if let Some(index) = self
                                .selected_record_ids
                                .iter()
                                .position(|&id| id == record_id)
                            {
                                self.selected_record_ids.remove(index);
                            } else {
                                self.selected_record_ids.push(record_id);
                            }
                        }
                    }
                }
                KeyCode::Char('b') => {
                    if !self.selected_record_ids.is_empty() {
                        self.show_confirmation = true;
                        self.confirmation_selected = 1;
                    } else {
                        self.set_error("No records selected for deletion.".to_string());
                    }
                }
                KeyCode::Enter => {
                    if let Some(selected) = self.table_state.selected() {
                        if selected < self.filtered_records.len() {
                            let record_id = self.filtered_records[selected].id;
                            self.selected_record_ids.push(record_id);
                            self.show_confirmation = true;
                            self.confirmation_selected = 1;
                        }
                    }
                }
                KeyCode::Char('a') => {
                    if self.selected_record_ids.len() == self.filtered_records.len() {
                        self.selected_record_ids.clear();
                    } else {
                        self.selected_record_ids =
                            self.filtered_records.iter().map(|r| r.id).collect();
                    }
                }
                KeyCode::Char('r') | KeyCode::Char('R') => {
                    if let Ok(records) = db::get_all_medical_records() {
                        self.records = records;
                        self.filter_records();
                    }
                }
                KeyCode::Esc => {
                    return Ok(Some(SelectedApp::None));
                }
                _ => {}
            }
        }
        Ok(None)
    }

    /// Clears the current error message and resets the error timer.
    ///
    /// This method is called to remove any existing error message from the display and reset the
    /// associated timer. This is typically done after the error has been displayed for a certain
    /// duration.
    fn clear_error(&mut self) {
        self.error_message = None;
        self.error_timer = None;
    }

    /// Sets an error message and starts the error display timer.
    ///
    /// This function sets the `error_message` field to the provided `message` and initializes
    /// the `error_timer` with the current instant. This is used to display an error to the user
    /// and automatically clear it after a predefined time.
    ///
    /// # Arguments
    ///
    /// * `message`: A `String` containing the error message to be displayed.
    fn set_error(&mut self, message: String) {
        self.error_message = Some(message);
        self.error_timer = Some(Instant::now());
    }

    /// Clears the success message and resets the success timer.
    ///
    /// This function clears the currently displayed success message and resets the associated timer.
    /// This is typically called after a successful operation, and the message should be removed from
    /// display after a certain period.
    fn clear_success(&mut self) {
        self.success_message = None;
        self.success_timer = None;
    }

    /// Checks if the success message timeout has elapsed and clears the message if it has.
    ///
    /// This method checks the `success_timer` to determine if the success message should be cleared.
    /// If the timer is active and the elapsed time exceeds 5 seconds, it calls `clear_success()`
    /// to remove the success message from display.
    fn check_success_timeout(&mut self) {
        if let Some(timer) = self.success_timer {
            if timer.elapsed() > Duration::from_secs(5) {
                self.clear_success();
            }
        }
    }

    /// Checks if the error message timeout has elapsed and clears the message if it has.
    ///
    /// This method checks the `error_timer` to determine if the error message should be cleared.
    /// If the timer is active and the elapsed time exceeds 5 seconds, it calls `clear_error()`
    /// to remove the error message from display.
    fn check_error_timeout(&mut self) {
        if let Some(timer) = self.error_timer {
            if timer.elapsed() > Duration::from_secs(5) {
                self.clear_error();
            }
        }
    }

    /// Checks the timeouts for both success and error messages.
    ///
    /// This method calls both `check_error_timeout()` and `check_success_timeout()` to manage
    /// the display durations of both error and success messages.
    fn check_timeouts(&mut self) {
        self.check_error_timeout();
        self.check_success_timeout();
    }
}

impl Default for DeleteRecord {
    /// Creates a new `DeleteRecord` component with default settings.
    fn default() -> Self {
        Self::new()
    }
}

impl Component for DeleteRecord {
    /// Handles key input events and directs them to the component's internal handler.
    ///
    /// This function is part of the `Component` trait implementation. It receives a `KeyEvent` and
    /// passes it to the `handle_input` method of the `DeleteRecord` struct. This allows the
    /// component to react to user input.
    fn handle_input(&mut self, event: KeyEvent) -> Result<Option<SelectedApp>> {
        self.handle_input(event)
    }

    /// Renders the `DeleteRecord` component to the terminal frame.
    ///
    /// This method is responsible for drawing the component's user interface. It constructs the
    /// layout, including the header, search input, record table, success/error messages, and
    /// confirmation dialog. The UI elements are styled and rendered using `ratatui` widgets.
    ///
    /// # Arguments
    ///
    /// * `frame`: A mutable reference to the `Frame` in which to render the component.
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
                Constraint::Min(5),
                Constraint::Length(2),
                Constraint::Length(3),
            ])
            .margin(1)
            .split(area);

        // Header
        let header_block = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(Color::Rgb(75, 75, 120)))
            .style(Style::default().bg(Color::Rgb(16, 16, 28)));
        frame.render_widget(header_block, layout[0]);

        let title = Paragraph::new("🗑️ RECORD DELETION MANAGER")
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
                Style::default().fg(Color::Rgb(250, 250, 110))
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

        // Record table
        let table_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(if !self.search_input.is_empty() {
                format!(
                    " Records ({} of {} matches) ",
                    self.filtered_records.len(),
                    self.records.len()
                )
            } else {
                format!(" Records ({}) ", self.records.len())
            })
            .title_style(
                Style::default()
                    .fg(Color::Rgb(230, 230, 250))
                    .add_modifier(Modifier::BOLD),
            )
            .border_style(Style::default().fg(Color::Rgb(140, 140, 200)))
            .style(Style::default().bg(Color::Rgb(26, 26, 36)));

        let selected_style = Style::default()
            .bg(Color::Rgb(45, 45, 60))
            .fg(Color::Rgb(250, 250, 250))
            .add_modifier(Modifier::BOLD);
        let normal_style = Style::default()
            .bg(Color::Rgb(26, 26, 36))
            .fg(Color::Rgb(220, 220, 240));

        let mut rows = Vec::new();
        for record in &self.filtered_records {
            let checkbox = if self.selected_record_ids.contains(&record.id) {
                "[✓]"
            } else {
                "[ ]"
            };

            rows.push(Row::new(vec![
                Cell::from(checkbox).style(normal_style),
                Cell::from(record.id.to_string()).style(normal_style),
                Cell::from(record.patient_id.to_string()).style(normal_style),
                Cell::from(record.diagnosis.clone()).style(normal_style),
            ]));
        }

        if self.filtered_records.is_empty() {
            let message = if self.search_input.is_empty() {
                "No records found in database"
            } else {
                "No records match your search criteria"
            };

            rows.push(Row::new(vec![
                Cell::from(""),
                Cell::from(""),
                Cell::from(message).style(Style::default().fg(Color::Rgb(180, 180, 200))),
                Cell::from(""),
            ]));
        }

        let table = Table::new(
            rows,
            [
                Constraint::Length(5),
                Constraint::Length(8),
                Constraint::Length(12),
                Constraint::Min(20),
            ],
        )
        .header(
            Row::new(vec![
                Cell::from(""),
                Cell::from("ID").style(Style::default().add_modifier(Modifier::BOLD)),
                Cell::from("Patient ID").style(Style::default().add_modifier(Modifier::BOLD)),
                Cell::from("Diagnosis").style(Style::default().add_modifier(Modifier::BOLD)),
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

        frame.render_stateful_widget(table, layout[2], &mut self.table_state.clone());

        // Success or error message
        if let Some(success) = &self.success_message {
            let success_paragraph = Paragraph::new(success.as_str())
                .style(
                    Style::default()
                        .fg(Color::Rgb(140, 219, 140))
                        .add_modifier(Modifier::BOLD),
                )
                .alignment(Alignment::Center);
            frame.render_widget(success_paragraph, layout[3]);
        } else if let Some(error) = &self.error_message {
            let error_paragraph = Paragraph::new(error.as_str())
                .style(
                    Style::default()
                        .fg(Color::Rgb(240, 100, 100))
                        .add_modifier(Modifier::BOLD),
                )
                .alignment(Alignment::Center);
            frame.render_widget(error_paragraph, layout[3]);
        }

        // Help text
        if self.is_searching {
            let help_text =
                Paragraph::new("Type to search | ↓/Enter: To results | Esc: Cancel search")
                    .style(Style::default().fg(Color::Rgb(180, 180, 200)))
                    .alignment(Alignment::Center);
            frame.render_widget(help_text, layout[4]);
        } else {
            let help_block = Block::default()
                .border_style(Style::default().fg(Color::Rgb(75, 75, 120)))
                .style(Style::default().bg(Color::Rgb(16, 16, 28)));

            let help_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(layout[4]);

            let help_text1 = Paragraph::new("/ or s: Search | ↑/↓: Navigate | Space: Toggle | A: Select/deselect all | R: Refresh")
                .style(Style::default().fg(Color::Rgb(180, 180, 200)))
                .alignment(Alignment::Center);

            let help_text2 = Paragraph::new("Enter: Delete selected | B: Bulk delete | Esc: Back")
                .style(Style::default().fg(Color::Rgb(180, 180, 200)))
                .alignment(Alignment::Center);

            frame.render_widget(help_block, layout[4]);
            frame.render_widget(help_text1, help_layout[0]);
            frame.render_widget(help_text2, help_layout[1]);
        }

        // Confirmation dialog
        if self.show_confirmation {
            let dialog_width = 50;
            let dialog_height = 8;
            let dialog_area = Rect::new(
                (area.width.saturating_sub(dialog_width)) / 2,
                (area.height.saturating_sub(dialog_height)) / 2,
                dialog_width,
                dialog_height,
            );

            frame.render_widget(Clear, dialog_area);

            let selected_count = self.selected_record_ids.len();
            let title = format!(
                " Delete {} Record{} ",
                selected_count,
                if selected_count == 1 { "" } else { "s" }
            );

            let dialog_block = Block::default()
                .title(title)
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

            let message = Paragraph::new("Are you sure you want to delete the selected record(s)?")
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
}
