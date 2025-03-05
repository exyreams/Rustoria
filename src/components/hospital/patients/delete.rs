//! Delete Patient component for the Hospital application.

use crate::app::SelectedApp;
use crate::components::Component;
use crate::db;
use crate::models::Patient;
use crate::tui::Frame;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, Cell, Clear, Paragraph, Row, Table, TableState},
};
use std::time::{Duration, Instant};

/// Component to delete patients with bulk delete functionality
pub struct DeletePatient {
    patients: Vec<Patient>,
    selected_patients: Vec<bool>,
    table_state: TableState,
    show_confirmation: bool,
    confirmation_selected: usize, // 0 for Yes, 1 for No
    error_message: Option<String>,
    success_message: Option<String>,
    error_timer: Option<Instant>,
    success_timer: Option<Instant>,
}

impl DeletePatient {
    /// Creates a new `DeletePatient` component.
    pub fn new() -> Self {
        let patients = match db::get_all_patients() {
            Ok(p) => p,
            Err(_) => Vec::new(),
        };

        let mut table_state = TableState::default();
        if !patients.is_empty() {
            table_state.select(Some(0));
        }

        Self {
            selected_patients: vec![false; patients.len()],
            patients,
            table_state,
            show_confirmation: false,
            confirmation_selected: 1, // Default to "No"
            error_message: None,
            success_message: None,
            error_timer: None,
            success_timer: None,
        }
    }

    fn handle_input(&mut self, key: KeyEvent) -> Result<Option<SelectedApp>> {
        self.check_timeouts();

        if self.show_confirmation {
            match key.code {
                KeyCode::Left | KeyCode::Right => {
                    self.confirmation_selected = 1 - self.confirmation_selected;
                }
                KeyCode::Enter => {
                    if self.confirmation_selected == 0 {
                        // Yes - delete selected patients
                        let mut deleted_count = 0;
                        let mut error_occurred = false;

                        for (index, checked) in self.selected_patients.iter().enumerate() {
                            if *checked {
                                let patient_id = self.patients[index].id;
                                match db::delete_patient(patient_id) {
                                    Ok(_) => deleted_count += 1,
                                    Err(_) => {
                                        error_occurred = true;
                                        break;
                                    }
                                }
                            }
                        }

                        if error_occurred {
                            self.set_error(format!(
                                "Error during deletion. {} patients deleted successfully.",
                                deleted_count
                            ));
                        } else if deleted_count > 0 {
                            self.success_message = Some(format!(
                                "{} patient{} deleted successfully!",
                                deleted_count,
                                if deleted_count == 1 { "" } else { "s" }
                            ));
                            self.success_timer = Some(Instant::now());
                        } else {
                            self.set_error("No patients were selected for deletion.".to_string());
                        }

                        // Refresh patient list
                        if let Ok(patients) = db::get_all_patients() {
                            self.patients = patients;
                            self.selected_patients = vec![false; self.patients.len()];
                            if self.patients.is_empty() {
                                self.table_state.select(None);
                            } else if let Some(selected) = self.table_state.selected() {
                                if selected >= self.patients.len() {
                                    self.table_state.select(Some(self.patients.len() - 1));
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
        } else {
            match key.code {
                KeyCode::Down => {
                    if !self.patients.is_empty() {
                        let next = match self.table_state.selected() {
                            Some(i) => {
                                if i >= self.patients.len() - 1 {
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
                    if !self.patients.is_empty() {
                        let next = match self.table_state.selected() {
                            Some(i) => {
                                if i == 0 {
                                    self.patients.len() - 1
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
                    // Toggle selected state of current row
                    if let Some(selected) = self.table_state.selected() {
                        if selected < self.selected_patients.len() {
                            self.selected_patients[selected] = !self.selected_patients[selected];
                        }
                    }
                }
                KeyCode::Char('b') => {
                    // Bulk delete
                    if self.selected_patients.iter().any(|&x| x) {
                        self.show_confirmation = true;
                        self.confirmation_selected = 1; // Default to "No" for safety
                    } else {
                        self.set_error("No patients selected for deletion.".to_string());
                    }
                }
                KeyCode::Enter => {
                    // Single delete - toggle current row and initiate confirmation
                    if let Some(selected) = self.table_state.selected() {
                        if selected < self.selected_patients.len() {
                            self.selected_patients[selected] = true;
                            self.show_confirmation = true;
                            self.confirmation_selected = 1; // Default to "No" for safety
                        }
                    }
                }
                KeyCode::Char('a') => {
                    // Select/deselect all
                    let all_selected = self.selected_patients.iter().all(|&x| x);
                    for i in 0..self.selected_patients.len() {
                        self.selected_patients[i] = !all_selected;
                    }
                }
                KeyCode::Esc => {
                    // Go back to the main menu
                    return Ok(Some(SelectedApp::None));
                }
                _ => {}
            }
        }
        Ok(None)
    }

    fn clear_error(&mut self) {
        self.error_message = None;
        self.error_timer = None;
    }

    fn set_error(&mut self, message: String) {
        self.error_message = Some(message);
        self.error_timer = Some(Instant::now());
    }

    fn clear_success(&mut self) {
        self.success_message = None;
        self.success_timer = None;
    }

    fn check_success_timeout(&mut self) {
        if let Some(timer) = self.success_timer {
            if timer.elapsed() > Duration::from_secs(5) {
                self.clear_success();
            }
        }
    }

    fn check_error_timeout(&mut self) {
        if let Some(timer) = self.error_timer {
            if timer.elapsed() > Duration::from_secs(5) {
                self.clear_error();
            }
        }
    }

    fn check_timeouts(&mut self) {
        self.check_error_timeout();
        self.check_success_timeout();
    }
}

impl Default for DeletePatient {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for DeletePatient {
    fn handle_input(&mut self, event: KeyEvent) -> Result<Option<SelectedApp>> {
        self.handle_input(event)
    }

    fn render(&self, frame: &mut Frame) {
        let area = frame.area();
        frame.render_widget(
            Block::default().style(Style::default().bg(Color::Rgb(16, 16, 28))),
            area,
        );

        // Main delete page rendering with table (always render this first)
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Length(3), // Instructions
                Constraint::Min(5),    // Table
                Constraint::Length(2), // Message area
            ])
            .margin(1)
            .split(area);

        let header_block = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(Color::Rgb(75, 75, 120)))
            .style(Style::default().bg(Color::Rgb(16, 16, 28)));
        frame.render_widget(header_block, layout[0]);

        let title = Paragraph::new("🗑️ PATIENT DELETION MANAGER")
            .style(
                Style::default()
                    .fg(Color::Rgb(230, 230, 250))
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Rgb(16, 16, 28)),
            )
            .alignment(Alignment::Center);
        frame.render_widget(title, layout[0]);

        // Instructions
        let instructions = Paragraph::new(
            "↑/↓: Navigate   Space: Toggle selection   B: Bulk delete   A: Select/deselect all   Enter: Delete selected   Esc: Back"
        )
        .style(Style::default().fg(Color::Rgb(180, 180, 200)))
        .alignment(Alignment::Center);
        frame.render_widget(instructions, layout[1]);

        // Patient table with checkboxes
        let table_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Patients ")
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

        // Create table rows
        let mut rows = Vec::new();
        for (i, patient) in self.patients.iter().enumerate() {
            let checkbox = if self.selected_patients[i] {
                "[✓]"
            } else {
                "[ ]"
            };

            rows.push(Row::new(vec![
                Cell::from(checkbox).style(normal_style),
                Cell::from(patient.id.to_string()).style(normal_style),
                Cell::from(patient.first_name.clone()).style(normal_style),
                Cell::from(patient.last_name.clone()).style(normal_style),
                Cell::from(patient.date_of_birth.clone()).style(normal_style),
                Cell::from(match patient.gender {
                    crate::models::Gender::Male => "Male",
                    crate::models::Gender::Female => "Female",
                    crate::models::Gender::Other => "Other",
                })
                .style(normal_style),
                Cell::from(patient.phone_number.clone()).style(normal_style),
                Cell::from(patient.address.clone()).style(normal_style),
            ]));
        }

        if self.patients.is_empty() {
            rows.push(Row::new(vec![
                Cell::from(""),
                Cell::from(""),
                Cell::from("No patients found in database")
                    .style(Style::default().fg(Color::Rgb(180, 180, 200))),
                Cell::from(""),
                Cell::from(""),
                Cell::from(""),
                Cell::from(""),
                Cell::from(""),
            ]));
        }

        let table = Table::new(
            rows,
            [
                Constraint::Length(5),  // Checkbox
                Constraint::Length(8),  // ID
                Constraint::Length(15), // First Name
                Constraint::Length(15), // Last Name
                Constraint::Length(12), // Date of Birth
                Constraint::Length(8),  // Gender
                Constraint::Length(15), // Phone Number
                Constraint::Min(20),    // Address
            ],
        )
        .header(
            Row::new(vec![
                Cell::from(""),
                Cell::from("ID").style(Style::default().add_modifier(Modifier::BOLD)),
                Cell::from("First Name").style(Style::default().add_modifier(Modifier::BOLD)),
                Cell::from("Last Name").style(Style::default().add_modifier(Modifier::BOLD)),
                Cell::from("DOB").style(Style::default().add_modifier(Modifier::BOLD)),
                Cell::from("Gender").style(Style::default().add_modifier(Modifier::BOLD)),
                Cell::from("Phone").style(Style::default().add_modifier(Modifier::BOLD)),
                Cell::from("Address").style(Style::default().add_modifier(Modifier::BOLD)),
            ])
            .style(Style::default().fg(Color::Rgb(180, 180, 250)))
            .height(1),
        )
        .block(table_block)
        .row_highlight_style(selected_style)
        .highlight_symbol("► ");

        // Need to cast to mutable for stateful widget with immutable &self
        let mut table_state_copy = self.table_state.clone();
        frame.render_stateful_widget(table, layout[2], &mut table_state_copy);

        // Display success or error message
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

        // Confirmation dialog as overlay (without dimming)
        if self.show_confirmation {
            // Confirmation dialog rendering
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

            let selected_count = self.selected_patients.iter().filter(|&&x| x).count();
            let title = format!(
                " Delete {} Patient{} ",
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

            let message =
                Paragraph::new("Are you sure you want to delete the selected patient(s)?")
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
