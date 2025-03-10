use crate::app::SelectedApp;
use crate::components::hospital::patients::PatientAction;
use crate::components::Component;
use crate::db;
use crate::models::{Gender, Patient};
use crate::tui::Frame;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{prelude::*, widgets::*};
use std::time::{Duration, Instant};

enum ConfirmAction {
    UpdatePatient,
}

enum UpdateState {
    SelectingPatient,
    EditingPatient,
}

pub struct UpdatePatient {
    all_patients: Vec<Patient>,
    filtered_patients: Vec<Patient>,
    search_input: String,
    is_searching: bool,
    table_state: TableState,
    update_state: UpdateState,
    patient_id_input: String,
    patient: Patient,
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
const FIRST_NAME_INPUT: usize = 1;
const LAST_NAME_INPUT: usize = 2;
const DOB_INPUT: usize = 3;
const GENDER_INPUT: usize = 4;
const ADDRESS_INPUT: usize = 5;
const PHONE_INPUT: usize = 6;
const EMAIL_INPUT: usize = 7;
const MEDICAL_HISTORY_INPUT: usize = 8;
const ALLERGIES_INPUT: usize = 9;
const MEDICATIONS_INPUT: usize = 10;
const INPUT_FIELDS: usize = 10;

impl UpdatePatient {
    pub fn new() -> Self {
        let mut selection_state = TableState::default();
        selection_state.select(Some(0));

        let mut edit_table_state = TableState::default();
        edit_table_state.select(Some(0));

        let all_patients = match db::get_all_patients() {
            Ok(patients) => patients,
            Err(_) => Vec::new(),
        };

        Self {
            all_patients: all_patients.clone(),
            filtered_patients: all_patients,
            search_input: String::new(),
            is_searching: false,
            table_state: selection_state,
            update_state: UpdateState::SelectingPatient,
            patient_id_input: String::new(),
            patient: Patient {
                id: 0,
                first_name: String::new(),
                last_name: String::new(),
                date_of_birth: String::new(),
                gender: Gender::Male,
                address: String::new(),
                phone_number: String::new(),
                email: None,
                medical_history: None,
                allergies: None,
                current_medications: None,
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
                        || p.phone_number.to_lowercase().contains(&search_term)
                })
                .cloned()
                .collect();
        }

        if let Some(selected) = self.table_state.selected() {
            if selected >= self.filtered_patients.len() && !self.filtered_patients.is_empty() {
                self.table_state.select(Some(0));
            }
        }
    }

    fn load_patient_by_id(&mut self, patient_id: i64) -> Result<()> {
        match db::get_patient(patient_id) {
            Ok(patient) => {
                self.patient = patient;
                self.loaded = true;
                self.update_state = UpdateState::EditingPatient;
                self.update_input_value();
                Ok(())
            }
            Err(_) => {
                self.set_error(format!("Patient with ID {} doesn't exist", patient_id));
                Err(anyhow::anyhow!("Patient not found"))
            }
        }
    }

    fn load_patient(&mut self) -> Result<()> {
        if !self.loaded {
            if let Ok(patient_id) = self.patient_id_input.parse::<i64>() {
                match self.load_patient_by_id(patient_id) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e),
                }
            } else {
                self.set_error("Invalid Patient ID format.".to_string());
                Err(anyhow::anyhow!("Invalid Patient ID format"))
            }
        } else {
            Ok(())
        }
    }

    fn load_selected_patient(&mut self) -> Result<()> {
        if let Some(selected) = self.table_state.selected() {
            if selected < self.filtered_patients.len() {
                let patient_id = self.filtered_patients[selected].id;
                self.patient_id_input = patient_id.to_string();
                return self.load_patient_by_id(patient_id);
            }
        }
        self.set_error("No patient selected".to_string());
        Err(anyhow::anyhow!("No patient selected"))
    }

    fn update_input_value(&mut self) {
        if !self.loaded {
            self.input_value = self.patient_id_input.clone();
            return;
        }

        if let Some(field_index) = self.selected_field {
            self.input_value = match field_index {
                ID_INPUT => self.patient.id.to_string(),
                FIRST_NAME_INPUT => self.patient.first_name.clone(),
                LAST_NAME_INPUT => self.patient.last_name.clone(),
                DOB_INPUT => self.patient.date_of_birth.clone(),
                GENDER_INPUT => match self.patient.gender {
                    Gender::Male => "Male".to_string(),
                    Gender::Female => "Female".to_string(),
                    Gender::Other => "Other".to_string(),
                },
                ADDRESS_INPUT => self.patient.address.clone(),
                PHONE_INPUT => self.patient.phone_number.clone(),
                EMAIL_INPUT => self.patient.email.clone().unwrap_or_default(),
                MEDICAL_HISTORY_INPUT => self.patient.medical_history.clone().unwrap_or_default(),
                ALLERGIES_INPUT => self.patient.allergies.clone().unwrap_or_default(),
                MEDICATIONS_INPUT => self.patient.current_medications.clone().unwrap_or_default(),
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
                FIRST_NAME_INPUT => self.patient.first_name = self.input_value.clone(),
                LAST_NAME_INPUT => self.patient.last_name = self.input_value.clone(),
                DOB_INPUT => self.patient.date_of_birth = self.input_value.clone(),
                GENDER_INPUT => {
                    self.patient.gender = match self.input_value.to_lowercase().as_str() {
                        "f" | "female" => Gender::Female,
                        "m" | "male" => Gender::Male,
                        _ => Gender::Other,
                    }
                }
                ADDRESS_INPUT => self.patient.address = self.input_value.clone(),
                PHONE_INPUT => self.patient.phone_number = self.input_value.clone(),
                EMAIL_INPUT => self.patient.email = Some(self.input_value.clone()),
                MEDICAL_HISTORY_INPUT => {
                    self.patient.medical_history = Some(self.input_value.clone())
                }
                ALLERGIES_INPUT => self.patient.allergies = Some(self.input_value.clone()),
                MEDICATIONS_INPUT => {
                    self.patient.current_medications = Some(self.input_value.clone())
                }
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

    fn update_patient(&mut self) -> Result<()> {
        match db::update_patient(&self.patient) {
            Ok(_) => {
                self.success_message = Some("Patient updated successfully!".to_string());
                self.success_timer = Some(Instant::now());

                if let Ok(patients) = db::get_all_patients() {
                    self.all_patients = patients.clone();
                    self.filtered_patients = patients;
                    self.filter_patients();
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
        self.update_state = UpdateState::SelectingPatient;
        self.loaded = false;
        self.patient_id_input = String::new();
        self.editing = false;
        self.clear_error();
        self.clear_success();
    }

    fn handle_input(&mut self, key: KeyEvent) -> Result<Option<PatientAction>> {
        self.check_timeouts();

        if self.show_confirmation {
            match key.code {
                KeyCode::Left | KeyCode::Right => {
                    self.confirmation_selected = 1 - self.confirmation_selected;
                }
                KeyCode::Enter => {
                    if self.confirmation_selected == 0 {
                        if let Some(ConfirmAction::UpdatePatient) = self.confirmed_action.take() {
                            let _ = self.update_patient();
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

        if matches!(self.update_state, UpdateState::SelectingPatient) {
            match key.code {
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
                    self.table_state.select(Some(0));
                }
                KeyCode::Esc if self.is_searching => {
                    self.is_searching = false;
                    self.search_input.clear();
                    self.filter_patients();
                }

                KeyCode::Char('/') | KeyCode::Char('s') | KeyCode::Char('S')
                    if !self.is_searching =>
                {
                    self.is_searching = true;
                }

                KeyCode::Char(c) if !self.is_searching => {
                    self.patient_id_input.push(c);
                    self.input_value = self.patient_id_input.clone();
                    self.clear_error();
                }
                KeyCode::Backspace if !self.is_searching => {
                    self.patient_id_input.pop();
                    self.input_value = self.patient_id_input.clone();
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
                    if selected < self.filtered_patients.len().saturating_sub(1) {
                        self.table_state.select(Some(selected + 1));
                    }
                }

                KeyCode::Enter => {
                    if self.is_searching {
                        if !self.filtered_patients.is_empty() {
                            self.is_searching = false;
                            self.table_state.select(Some(0));
                        }
                    } else {
                        if !self.patient_id_input.is_empty() {
                            let _ = self.load_patient();
                        } else if !self.filtered_patients.is_empty() {
                            let _ = self.load_selected_patient();
                        }
                    }
                }

                KeyCode::Esc if !self.is_searching => {
                    return Ok(Some(PatientAction::BackToHome));
                }
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
                    "Are you sure you want to update this patient?".to_string(),
                    ConfirmAction::UpdatePatient,
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

impl Default for UpdatePatient {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for UpdatePatient {
    fn handle_input(&mut self, event: KeyEvent) -> Result<Option<SelectedApp>> {
        match self.handle_input(event)? {
            Some(PatientAction::BackToHome) => Ok(Some(crate::app::SelectedApp::None)),
            Some(PatientAction::BackToList) => Ok(None),
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
            UpdateState::SelectingPatient => self.render_patient_selection(frame, area),
            UpdateState::EditingPatient => self.render_patient_editing(frame, area),
        }

        if self.show_confirmation {
            self.render_confirmation_dialog(frame, area);
        }
    }
}

impl UpdatePatient {
    fn render_patient_selection(&self, frame: &mut Frame, area: Rect) {
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

        let title = Paragraph::new("✍️  SELECT PATIENT TO UPDATE")
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
                " Search Patients ",
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
                " Patient ID ",
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

        let id_input_paragraph = Paragraph::new(self.patient_id_input.clone())
            .style(
                Style::default()
                    .fg(Color::Rgb(220, 220, 240))
                    .bg(Color::Rgb(26, 26, 36)),
            )
            .block(id_input_block);
        frame.render_widget(id_input_paragraph, main_layout[2]);

        if self.filtered_patients.is_empty() {
            let no_patients = Paragraph::new(if self.search_input.is_empty() {
                "No patients found in database"
            } else {
                "No patients match your search criteria"
            })
            .style(Style::default().fg(Color::Rgb(220, 220, 240)))
            .alignment(Alignment::Center);
            frame.render_widget(no_patients, main_layout[3]);
        } else {
            let patients_rows: Vec<Row> = self
                .filtered_patients
                .iter()
                .map(|p| {
                    Row::new(vec![
                        p.id.to_string(),
                        p.first_name.clone(),
                        p.last_name.clone(),
                        p.phone_number.clone(),
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

            let header = Row::new(vec!["ID", "First Name", "Last Name", "Phone"])
                .style(
                    Style::default()
                        .fg(Color::Rgb(220, 220, 240))
                        .bg(Color::Rgb(80, 60, 130))
                        .add_modifier(Modifier::BOLD),
                )
                .height(1);

            let widths = [
                Constraint::Percentage(15),
                Constraint::Percentage(30),
                Constraint::Percentage(30),
                Constraint::Percentage(25),
            ];

            let patients_table = Table::new(patients_rows, widths)
                .header(header)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .title(format!(" Patients ({}) ", self.filtered_patients.len()))
                        .border_style(Style::default().fg(Color::Rgb(140, 140, 200)))
                        .style(Style::default().bg(Color::Rgb(26, 26, 36))),
                )
                .column_spacing(2)
                .row_highlight_style(selected_style)
                .highlight_symbol("► ");

            frame.render_stateful_widget(
                patients_table,
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
            "/ or s: Search | ↑/↓: Navigate patients | Enter: Select patient | Esc: Back"
        };

        let help_paragraph = Paragraph::new(help_text)
            .style(Style::default().fg(Color::Rgb(220, 220, 240)))
            .alignment(Alignment::Center);
        frame.render_widget(help_paragraph, main_layout[5]);
    }

    fn render_patient_editing(&self, frame: &mut Frame, area: Rect) {
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
            "✍️  EDITING PATIENT"
        } else {
            "✍️  UPDATE PATIENT"
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

        let id_str = self.patient.id.to_string();
        let gender_str = match self.patient.gender {
            Gender::Male => "Male",
            Gender::Female => "Female",
            Gender::Other => "Other",
        };
        let email_str = self.patient.email.clone().unwrap_or_default();
        let medical_history_str = self.patient.medical_history.clone().unwrap_or_default();
        let allergies_str = self.patient.allergies.clone().unwrap_or_default();
        let medications_str = self.patient.current_medications.clone().unwrap_or_default();

        let table_items = vec![
            Row::new(vec!["ID", &id_str])
                .style(Style::default().fg(Color::Rgb(220, 220, 240)))
                .height(1)
                .bottom_margin(0),
            Row::new(vec!["First Name", &self.patient.first_name])
                .style(Style::default().fg(Color::Rgb(220, 220, 240)))
                .height(1)
                .bottom_margin(0),
            Row::new(vec!["Last Name", &self.patient.last_name])
                .style(Style::default().fg(Color::Rgb(220, 220, 240)))
                .height(1)
                .bottom_margin(0),
            Row::new(vec!["Date of Birth", &self.patient.date_of_birth])
                .style(Style::default().fg(Color::Rgb(220, 220, 240)))
                .height(1)
                .bottom_margin(0),
            Row::new(vec!["Gender", &gender_str])
                .style(Style::default().fg(Color::Rgb(220, 220, 240)))
                .height(1)
                .bottom_margin(0),
            Row::new(vec!["Phone", &self.patient.phone_number])
                .style(Style::default().fg(Color::Rgb(220, 220, 240)))
                .height(1)
                .bottom_margin(0),
            Row::new(vec!["Address", &self.patient.address])
                .style(Style::default().fg(Color::Rgb(220, 220, 240)))
                .height(1)
                .bottom_margin(0),
            Row::new(vec!["Email", &email_str])
                .style(Style::default().fg(Color::Rgb(220, 220, 240)))
                .height(1)
                .bottom_margin(0),
            Row::new(vec!["Medical History", &medical_history_str])
                .style(Style::default().fg(Color::Rgb(220, 220, 240)))
                .height(1)
                .bottom_margin(0),
            Row::new(vec!["Allergies", &allergies_str])
                .style(Style::default().fg(Color::Rgb(220, 220, 240)))
                .height(1)
                .bottom_margin(0),
            Row::new(vec!["Medications", &medications_str])
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
                    .title(" Patient Data ")
                    .border_style(Style::default().fg(Color::Rgb(140, 140, 200)))
                    .style(Style::default().bg(Color::Rgb(26, 26, 36))),
            )
            .column_spacing(2)
            .row_highlight_style(selected_style)
            .highlight_symbol("► ");

        frame.render_stateful_widget(table, main_layout[1], &mut self.edit_table_state.clone());

        let input_label = match self.selected_field {
            Some(ID_INPUT) => "ID",
            Some(FIRST_NAME_INPUT) => "First Name",
            Some(LAST_NAME_INPUT) => "Last Name",
            Some(DOB_INPUT) => "Date of Birth",
            Some(GENDER_INPUT) => "Gender",
            Some(ADDRESS_INPUT) => "Address",
            Some(PHONE_INPUT) => "Phone",
            Some(EMAIL_INPUT) => "Email",
            Some(MEDICAL_HISTORY_INPUT) => "Medical History",
            Some(ALLERGIES_INPUT) => "Allergies",
            Some(MEDICATIONS_INPUT) => "Medications",
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
            "↑/↓: Navigate | E: Edit Field | Ctrl+S: Save Patient | Esc: Back"
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
