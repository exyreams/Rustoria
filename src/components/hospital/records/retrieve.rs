use crate::app::SelectedApp;
use crate::components::Component;
use crate::db;
use crate::models::{MedicalRecord, Patient};
use crate::tui::Frame;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{prelude::*, widgets::*};
use std::collections::HashMap;

const SEARCH_FIELD: usize = 0;
const RECORD_LIST: usize = 1;
const BACK_BUTTON: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetrieveState {
    ViewingList,
    ViewingDetails,
}
pub struct RetrieveRecords {
    records: Vec<MedicalRecord>,
    filtered_records: Vec<MedicalRecord>,
    search_input: String,
    is_searching: bool,
    state: TableState,
    error_message: Option<String>,
    focus_index: usize,
    view_state: RetrieveState,
    patients: HashMap<i64, Patient>,
}

impl RetrieveRecords {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
            filtered_records: Vec::new(),
            search_input: String::new(),
            is_searching: false,
            state: TableState::default(),
            error_message: None,
            focus_index: RECORD_LIST,
            view_state: RetrieveState::ViewingList,
            patients: HashMap::new(),
        }
    }

    pub fn fetch_records(&mut self) -> Result<()> {
        match db::get_all_medical_records() {
            Ok(records) => {
                self.records = records;
                self.fetch_patients_data()?;
                self.filter_records();

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
                self.error_message = Some(format!("Failed to fetch records: {}", e));
                Ok(())
            }
        }
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
                self.error_message = Some(format!("Failed to fetch patient data: {}", e));
                Ok(())
            }
        }
    }

    fn filter_records(&mut self) {
        if self.search_input.is_empty() {
            self.filtered_records = self.records.clone();
        } else {
            let search_term = self.search_input.to_lowercase();
            self.filtered_records = self
                .records
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

        if let Some(selected) = self.state.selected() {
            if selected >= self.filtered_records.len() && !self.filtered_records.is_empty() {
                self.state.select(Some(0));
            } else if self.filtered_records.is_empty() {
                self.state.select(None);
            }
        }
    }

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

    fn view_record_details(&mut self) {
        if !self.filtered_records.is_empty() && self.state.selected().is_some() {
            self.view_state = RetrieveState::ViewingDetails;
        }
    }

    fn return_to_list(&mut self) {
        self.view_state = RetrieveState::ViewingList;
    }

    fn focus_next(&mut self) {
        self.focus_index = (self.focus_index + 1) % 3;
        self.is_searching = self.focus_index == SEARCH_FIELD;
    }

    fn focus_previous(&mut self) {
        self.focus_index = (self.focus_index + 2) % 3;
        self.is_searching = self.focus_index == SEARCH_FIELD;
    }

    pub fn handle_input(&mut self, key: KeyEvent) -> Result<Option<SelectedApp>> {
        match self.view_state {
            RetrieveState::ViewingList => {
                if self.is_searching {
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
                            self.view_record_details();
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
                        return Ok(Some(SelectedApp::None));
                    }
                    _ => {}
                }
            }
            RetrieveState::ViewingDetails => match key.code {
                KeyCode::Enter | KeyCode::Esc | KeyCode::Backspace => {
                    self.return_to_list();
                }
                KeyCode::Char('b') | KeyCode::Char('B') => {
                    self.return_to_list();
                }
                _ => {}
            },
        }
        Ok(None)
    }

    fn selected_record(&self) -> Option<&MedicalRecord> {
        self.state
            .selected()
            .and_then(|i| self.filtered_records.get(i))
    }

    fn get_patient(&self, patient_id: i64) -> Option<&Patient> {
        self.patients.get(&patient_id)
    }
}

impl Component for RetrieveRecords {
    fn handle_input(&mut self, event: KeyEvent) -> Result<Option<SelectedApp>> {
        self.handle_input(event)
    }

    fn render(&self, frame: &mut Frame) {
        frame.render_widget(
            Block::default().style(Style::default().bg(Color::Rgb(16, 16, 28))),
            frame.area(),
        );

        match self.view_state {
            RetrieveState::ViewingList => self.render_list_view(frame),
            RetrieveState::ViewingDetails => self.render_details_view(frame),
        }
    }
}

impl RetrieveRecords {
    fn render_list_view(&self, frame: &mut Frame) {
        let area = frame.area();

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .margin(1)
            .split(area);

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

        let header_cells = ["ID", "First Name", "Last Name", "Diagnosis"]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().fg(Color::Rgb(230, 230, 250))));
        let header = Row::new(header_cells)
            .style(Style::default().bg(Color::Rgb(80, 60, 130)))
            .height(1);

        let rows = self.filtered_records.iter().map(|record| {
            let (first_name, last_name) = match self.get_patient(record.patient_id) {
                Some(patient) => (patient.first_name.clone(), patient.last_name.clone()),
                None => ("Unknown".to_string(), "Patient".to_string()),
            };

            let cells = vec![
                Cell::from(record.id.to_string()),
                Cell::from(first_name),
                Cell::from(last_name),
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
                Constraint::Percentage(20),
                Constraint::Percentage(50),
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
                        .title(table_title.clone())
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

        if let Some(error) = &self.error_message {
            let error_paragraph = Paragraph::new(error.as_str())
                .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
                .alignment(Alignment::Center);
            frame.render_widget(error_paragraph, layout[3]);
        }

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

        let help_text = if self.is_searching {
            "Type to search | ↓/Enter: To results | Esc: Cancel search"
        } else {
            "/ or s: Search | ↑↓: Navigate | Enter: View Details | R: Refresh | Tab: Focus"
        };

        let help_paragraph = Paragraph::new(help_text)
            .style(Style::default().fg(Color::Rgb(140, 140, 170)))
            .alignment(Alignment::Center);
        frame.render_widget(help_paragraph, layout[6]);
    }

    fn render_details_view(&self, frame: &mut Frame) {
        let area = frame.area();

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(20),
                Constraint::Length(2),
            ])
            .margin(1)
            .split(area);

        let header_block = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(Color::Rgb(75, 75, 120)))
            .style(Style::default().bg(Color::Rgb(16, 16, 28)));
        frame.render_widget(header_block, layout[0]);

        let title = Paragraph::new("🏥 RECORD DETAILS")
            .style(
                Style::default()
                    .fg(Color::Rgb(230, 230, 250))
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Rgb(16, 16, 28)),
            )
            .alignment(Alignment::Center);
        frame.render_widget(title, layout[0]);

        if let Some(record) = self.selected_record() {
            let patient = self.get_patient(record.patient_id);
            let _patient_name = match patient {
                Some(p) => format!("{} {}", p.first_name, p.last_name),
                None => "Unknown Patient".to_string(),
            };

            let blocks_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Length(4),
                    Constraint::Length(4),
                    Constraint::Length(6),
                    Constraint::Length(6),
                ])
                .split(layout[1]);

            let record_info_text = format!("   Record Number: {}", record.id);
            let record_info_block = Block::default()
                .title(Span::styled(
                    " Patient Information ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::White))
                .style(Style::default().bg(Color::Rgb(22, 22, 35)));

            let record_info_widget = Paragraph::new(record_info_text)
                .style(Style::default().fg(Color::Rgb(220, 220, 240)))
                .block(record_info_block);

            frame.render_widget(record_info_widget, blocks_layout[0]);

            let diagnosis_block = Block::default()
                .title(Span::styled(
                    " Diagnosis ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::White))
                .style(Style::default().bg(Color::Rgb(22, 22, 35)));

            let diagnosis_widget = Paragraph::new(format!("   {}", record.diagnosis))
                .style(Style::default().fg(Color::Rgb(220, 220, 240)))
                .block(diagnosis_block)
                .wrap(Wrap { trim: true });

            frame.render_widget(diagnosis_widget, blocks_layout[1]);

            let prescription_text = record
                .prescription
                .clone()
                .unwrap_or_else(|| "None".to_string());
            let prescription_block = Block::default()
                .title(Span::styled(
                    " Prescription ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::White))
                .style(Style::default().bg(Color::Rgb(22, 22, 35)));

            let prescription_widget = Paragraph::new(format!("   {}", prescription_text))
                .style(Style::default().fg(Color::Rgb(220, 220, 240)))
                .block(prescription_block)
                .wrap(Wrap { trim: true });

            frame.render_widget(prescription_widget, blocks_layout[2]);

            let doctor_notes_block = Block::default()
                .title(Span::styled(
                    " Doctor's Notes ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::White))
                .style(Style::default().bg(Color::Rgb(22, 22, 35)));

            let doctor_notes_widget = Paragraph::new(format!("   {}", record.doctor_notes))
                .style(Style::default().fg(Color::Rgb(220, 220, 240)))
                .block(doctor_notes_block)
                .wrap(Wrap { trim: true });

            frame.render_widget(doctor_notes_widget, blocks_layout[3]);

            let nurse_notes_text = record
                .nurse_notes
                .clone()
                .unwrap_or_else(|| "None".to_string());
            let nurse_notes_block = Block::default()
                .title(Span::styled(
                    " Nurse's Notes ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::White))
                .style(Style::default().bg(Color::Rgb(22, 22, 35)));

            let nurse_notes_widget = Paragraph::new(format!("   {}", nurse_notes_text))
                .style(Style::default().fg(Color::Rgb(220, 220, 240)))
                .block(nurse_notes_block)
                .wrap(Wrap { trim: true });

            frame.render_widget(nurse_notes_widget, blocks_layout[4]);
        }

        let footer_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Length(1)])
            .split(layout[2]);

        let back_button = Paragraph::new("► Back ◄")
            .style(
                Style::default()
                    .fg(Color::Rgb(129, 199, 245))
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center);
        frame.render_widget(back_button, footer_layout[0]);

        let help_text = "Enter/Esc/Backspace: Return to list";
        let help_paragraph = Paragraph::new(help_text)
            .style(Style::default().fg(Color::Rgb(140, 140, 170)))
            .alignment(Alignment::Center);
        frame.render_widget(help_paragraph, footer_layout[1]);
    }
}

impl Default for RetrieveRecords {
    fn default() -> Self {
        Self::new()
    }
}
