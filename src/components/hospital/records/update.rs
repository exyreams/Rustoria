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

enum ConfirmAction {
    UpdateRecord,
}

enum UpdateState {
    SelectingRecord,
    EditingRecord,
}

pub struct UpdateRecord {
    all_records: Vec<MedicalRecord>,
    filtered_records: Vec<MedicalRecord>,
    patients: HashMap<i64, Patient>,
    search_input: String,
    is_searching: bool,
    table_state: TableState,
    update_state: UpdateState,
    record_id_input: String,
    record: MedicalRecord,
    loaded: bool,
    selected_field: Option<usize>,
    edit_table_state: TableState,
    input_value: String,
    editing: bool,
    error_message: Option<String>,
    error_timer: Option<Instant>,
    success_message: Option<String>,
    success_timer: Option<Instant>,
    show_confirmation: bool,
    confirmation_message: String,
    confirmed_action: Option<ConfirmAction>,
    confirmation_selected: usize,
}

const ID_INPUT: usize = 0;
const PATIENT_ID_INPUT: usize = 1;
const DOCTOR_NOTES_INPUT: usize = 2;
const NURSE_NOTES_INPUT: usize = 3;
const DIAGNOSIS_INPUT: usize = 4;
const PRESCRIPTION_INPUT: usize = 5;
const INPUT_FIELDS: usize = 5;

impl UpdateRecord {
    pub fn new() -> Self {
        let mut selection_state = TableState::default();
        selection_state.select(Some(0));

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
            confirmation_selected: 0,
        }
    }

    pub fn fetch_records(&mut self) -> Result<()> {
        self.all_records = db::get_all_medical_records()?;
        self.fetch_patients_data()?;
        self.filter_records();
        Ok(())
    }

    fn fetch_patients_data(&mut self) -> Result<()> {
        self.patients.clear();

        match db::get_all_patients() {
            Ok(all_patients) => {
                for patient in all_patients {
                    self.patients.insert(patient.id, patient);
                }
                Ok(())
            }
            Err(e) => {
                self.set_error(format!("Failed to fetch patient data: {}", e));
                Ok(())
            }
        }
    }

    fn get_patient(&self, patient_id: i64) -> Option<&Patient> {
        self.patients.get(&patient_id)
    }

    fn filter_records(&mut self) {
        if self.search_input.is_empty() {
            self.filtered_records = self.all_records.clone();
        } else {
            let search_term = self.search_input.to_lowercase();
            self.filtered_records = self
                .all_records
                .iter()
                .filter(|r| {
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

        if let Some(selected) = self.table_state.selected() {
            if selected >= self.filtered_records.len() && !self.filtered_records.is_empty() {
                self.table_state.select(Some(0));
            }
        }
    }

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
                        return;
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

    fn show_confirmation(&mut self, message: String, action: ConfirmAction) {
        self.show_confirmation = true;
        self.confirmation_message = message;
        self.confirmed_action = Some(action);
        self.confirmation_selected = 0;
    }

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

    fn back_to_selection(&mut self) {
        self.update_state = UpdateState::SelectingRecord;
        self.loaded = false;
        self.record_id_input = String::new();
        self.editing = false;
        self.clear_error();
        self.clear_success();
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

impl Default for UpdateRecord {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for UpdateRecord {
    fn handle_input(&mut self, event: KeyEvent) -> Result<Option<SelectedApp>> {
        match self.handle_input(event)? {
            Some(_) => Ok(Some(SelectedApp::None)),
            None => Ok(None),
        }
    }

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
    fn render_record_selection(&self, frame: &mut Frame, area: Rect) {
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(1),
                Constraint::Length(2),
            ])
            .margin(1)
            .split(area);

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
                Style::default().fg(Color::Rgb(250, 250, 110))
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

    fn render_record_editing(&self, frame: &mut Frame, area: Rect) {
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(14),
                Constraint::Length(3),
                Constraint::Length(1),
                Constraint::Length(2),
            ])
            .margin(1)
            .split(area);

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
                Style::default().fg(Color::Rgb(140, 219, 140))
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
