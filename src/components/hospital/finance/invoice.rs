use crate::app::SelectedApp;
use crate::components::Component;
use crate::db;
use crate::models::{Invoice, Patient};
use crate::tui::Frame;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{prelude::*, widgets::*};
use std::time::{Duration, Instant};
use time::OffsetDateTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvoiceState {
    SelectingPatient,
    EnteringDetails,
}

const PATIENT_SELECTION: usize = 0;
const INVOICE_DETAILS_FIELDS: usize = 3;
const SUBMIT_BUTTON: usize = 4;
const BACK_BUTTON: usize = 5;

pub struct InvoiceComponent {
    all_patients: Vec<Patient>,
    filtered_patients: Vec<Patient>,
    selected_patient: Option<Patient>,
    search_input: String,
    is_searching: bool,
    table_state: TableState,
    invoice_item: String,
    invoice_quantity: String,
    invoice_cost: String,
    focus_index: usize,
    state: InvoiceState,
    error_message: Option<String>,
    error_timer: Option<Instant>,
    success_message: Option<String>,
    success_timer: Option<Instant>,
}

impl Default for InvoiceComponent {
    fn default() -> Self {
        let mut table_state = TableState::default();
        table_state.select(Some(0));
        InvoiceComponent {
            all_patients: Vec::new(),
            filtered_patients: Vec::new(),
            selected_patient: None,
            search_input: String::new(),
            is_searching: false,
            table_state,
            invoice_item: String::new(),
            invoice_quantity: String::new(),
            invoice_cost: String::new(),
            focus_index: PATIENT_SELECTION,
            state: InvoiceState::SelectingPatient,
            error_message: None,
            error_timer: None,
            success_message: None,
            success_timer: None,
        }
    }
}

impl InvoiceComponent {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn load_patients(&mut self) -> Result<()> {
        self.all_patients = db::get_all_patients()?;
        self.filter_patients();
        Ok(())
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
                })
                .cloned()
                .collect();
        }

        if !self.filtered_patients.is_empty() {
            self.table_state.select(Some(0));
        } else {
            self.table_state.select(None);
        }
    }

    fn select_next_patient(&mut self) {
        if self.filtered_patients.is_empty() {
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => {
                if i >= self.filtered_patients.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    fn select_previous_patient(&mut self) {
        if self.filtered_patients.is_empty() {
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.filtered_patients.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
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
    pub fn check_error_timeout(&mut self) {
        if let Some(timer) = self.error_timer {
            if timer.elapsed() > Duration::from_secs(5) {
                self.clear_error();
            }
        }
    }

    pub fn check_success_timeout(&mut self) {
        if let Some(timer) = self.success_timer {
            if timer.elapsed() > Duration::from_secs(5) {
                self.clear_success();
            }
        }
    }
    pub fn handle_input(&mut self, key: KeyEvent) -> Result<Option<SelectedApp>> {
        self.check_error_timeout();
        self.check_success_timeout();

        match self.state {
            InvoiceState::SelectingPatient => {
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
                    KeyCode::Up => self.select_previous_patient(),
                    KeyCode::Down => self.select_next_patient(),
                    KeyCode::Tab => {
                        self.focus_index = if self.focus_index == PATIENT_SELECTION {
                            BACK_BUTTON
                        } else {
                            PATIENT_SELECTION
                        };
                    }
                    KeyCode::Char(' ') => {
                        if let Some(selected) = self.table_state.selected() {
                            if selected < self.filtered_patients.len() {
                                if let Some(patient) = &self.selected_patient {
                                    if patient.id == self.filtered_patients[selected].id {
                                        self.selected_patient = None;
                                    } else {
                                        self.selected_patient =
                                            Some(self.filtered_patients[selected].clone());
                                    }
                                } else {
                                    self.selected_patient =
                                        Some(self.filtered_patients[selected].clone());
                                }
                            }
                        }
                    }
                    KeyCode::Enter => {
                        if self.focus_index == BACK_BUTTON {
                            return Ok(Some(SelectedApp::None));
                        }
                        if self.is_searching {
                            if !self.filtered_patients.is_empty() {
                                self.is_searching = false;
                                self.table_state.select(Some(0));
                            }
                        } else {
                            if let Some(selected) = self.table_state.selected() {
                                if selected < self.filtered_patients.len() {
                                    if let Some(patient) = &self.selected_patient {
                                        if patient.id == self.filtered_patients[selected].id {
                                            self.state = InvoiceState::EnteringDetails;
                                            self.focus_index = 0;
                                            return Ok(None);
                                        } else {
                                            self.set_error(
                                                "Please Select Patient with Spacebar".to_string(),
                                            );
                                            return Ok(None);
                                        }
                                    } else {
                                        self.set_error(
                                            "Please Select Patient with Spacebar".to_string(),
                                        );
                                        return Ok(None);
                                    }
                                }
                            }
                        }
                    }
                    KeyCode::Esc => return Ok(Some(SelectedApp::None)),
                    _ => {}
                }
                return Ok(None);
            }
            InvoiceState::EnteringDetails => match key.code {
                KeyCode::Char(c) => match self.focus_index {
                    0 => self.invoice_item.push(c),
                    1 => self.invoice_quantity.push(c),
                    2 => self.invoice_cost.push(c),
                    _ => {}
                },
                KeyCode::Backspace => match self.focus_index {
                    0 => {
                        self.invoice_item.pop();
                    }
                    1 => {
                        self.invoice_quantity.pop();
                    }
                    2 => {
                        self.invoice_cost.pop();
                    }
                    _ => {}
                },
                KeyCode::Tab => {
                    self.focus_index = (self.focus_index + 1) % (INVOICE_DETAILS_FIELDS + 2);
                }
                KeyCode::Down => {
                    self.focus_index = (self.focus_index + 1) % (INVOICE_DETAILS_FIELDS + 2);
                }
                KeyCode::Up => {
                    self.focus_index = (self.focus_index + (INVOICE_DETAILS_FIELDS + 1))
                        % (INVOICE_DETAILS_FIELDS + 2);
                }
                KeyCode::Enter if self.focus_index == BACK_BUTTON => {
                    self.state = InvoiceState::SelectingPatient;
                    self.focus_index = PATIENT_SELECTION;
                    return Ok(None);
                }
                KeyCode::Enter if self.focus_index == SUBMIT_BUTTON => {
                    if self.invoice_item.is_empty() {
                        self.set_error("Invoice Item cannot be empty".to_string());
                        return Ok(None);
                    }
                    if self.invoice_quantity.is_empty() {
                        self.set_error("Invoice Quantity cannot be empty".to_string());
                        return Ok(None);
                    }
                    if self.invoice_cost.is_empty() {
                        self.set_error("Invoice Cost cannot be empty".to_string());
                        return Ok(None);
                    }
                    if self.invoice_quantity.parse::<i32>().is_err() {
                        self.set_error(
                            "Invalid quantity. Please enter a valid number.".to_string(),
                        );
                        return Ok(None);
                    }
                    if self.invoice_cost.parse::<f64>().is_err() {
                        self.set_error("Invalid cost. Please enter a valid number.".to_string());
                        return Ok(None);
                    }
                    if let Some(patient) = &self.selected_patient {
                        let new_invoice = Invoice {
                            id: 0,
                            patient_id: patient.id,
                            item: self.invoice_item.clone(),
                            quantity: self.invoice_quantity.parse::<i32>().unwrap(),
                            cost: self.invoice_cost.parse::<f64>().unwrap(),
                        };
                        match db::create_invoice(&new_invoice) {
                            Ok(_) => {
                                self.success_message =
                                    Some("Invoice created successfully!".to_string());
                                self.success_timer = Some(Instant::now());
                            }
                            Err(e) => {
                                self.set_error(format!("Database error: {}", e));
                                return Ok(None);
                            }
                        }
                        self.invoice_item.clear();
                        self.invoice_quantity.clear();
                        self.invoice_cost.clear();
                        self.state = InvoiceState::SelectingPatient;
                        self.focus_index = PATIENT_SELECTION;
                        self.selected_patient = None;
                        self.clear_error();
                        return Ok(None);
                    } else {
                        self.set_error("Please select a patient first.".to_string());
                        return Ok(None);
                    }
                }
                KeyCode::Enter => {}
                KeyCode::Esc => {
                    self.state = InvoiceState::SelectingPatient;
                    self.focus_index = PATIENT_SELECTION;
                    return Ok(None);
                }
                _ => {}
            },
        }
        Ok(None)
    }
}

impl Component for InvoiceComponent {
    fn handle_input(&mut self, event: KeyEvent) -> Result<Option<SelectedApp>> {
        self.handle_input(event)
    }
    fn render(&self, frame: &mut Frame) {
        frame.render_widget(
            Block::default().style(Style::default().bg(Color::Rgb(16, 16, 28))),
            frame.area(),
        );
        match self.state {
            InvoiceState::SelectingPatient => {
                self.render_patient_selection_page(frame);
            }
            InvoiceState::EnteringDetails => {
                self.render_invoice_details_page(frame);
            }
        }
    }
}
impl InvoiceComponent {
    fn render_patient_selection_page(&self, frame: &mut Frame) {
        let area = frame.area();
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(7),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .margin(1)
            .split(area);
        let header = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(Color::Rgb(75, 75, 120)))
            .style(Style::default().bg(Color::Rgb(16, 16, 28)));
        frame.render_widget(header, layout[0]);
        let title = Paragraph::new("🧾 SELECT PATIENT FOR INVOICE")
            .style(
                Style::default()
                    .fg(Color::Rgb(230, 230, 250))
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Rgb(16, 16, 28)),
            )
            .alignment(Alignment::Center);
        frame.render_widget(title, layout[0]);
        self.render_patient_selection_content(frame, layout[1]);
        self.render_status_message(frame, layout[2]);
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
        frame.render_widget(
            Paragraph::new(back_text)
                .style(back_style)
                .alignment(Alignment::Center),
            layout[3],
        );
        frame.render_widget(
            Paragraph::new(
                "/ or s: Search, ↑/↓: Navigate | Spacebar: Select | Enter: Confirm | Tab: Back | Esc: Exit"
            )
            .style(Style::default().fg(Color::Rgb(180, 180, 200)))
            .alignment(Alignment::Center),
            layout[6],
        );
    }

    fn render_patient_selection_content(&self, frame: &mut Frame, area: Rect) {
        let content_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(4)])
            .split(area);
        let search_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(Span::styled(
                " Search Patients ",
                Style::default()
                    .fg(Color::Rgb(230, 230, 250))
                    .add_modifier(Modifier::BOLD),
            ))
            .border_style(
                if self.is_searching && self.focus_index == PATIENT_SELECTION {
                    Style::default().fg(Color::Rgb(250, 250, 110))
                } else {
                    Style::default().fg(Color::Rgb(75, 75, 120))
                },
            )
            .style(Style::default().bg(Color::Rgb(22, 22, 35)));
        let search_paragraph = Paragraph::new(self.search_input.clone())
            .style(Style::default().fg(Color::Rgb(220, 220, 240)))
            .block(search_block);
        frame.render_widget(search_paragraph, content_layout[0]);
        let table_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(if !self.search_input.is_empty() {
                format!(
                    " Select Patient ({} of {} matches) ",
                    self.filtered_patients.len(),
                    self.all_patients.len()
                )
            } else {
                format!(" Select Patient ({}) ", self.all_patients.len())
            })
            .title_style(
                Style::default()
                    .fg(Color::Rgb(230, 230, 250))
                    .add_modifier(Modifier::BOLD),
            )
            .border_style(
                if self.focus_index == PATIENT_SELECTION && !self.is_searching {
                    Style::default().fg(Color::Rgb(250, 250, 110))
                } else {
                    Style::default().fg(Color::Rgb(140, 140, 200))
                },
            )
            .style(Style::default().bg(Color::Rgb(26, 26, 36)));
        let selected_style = Style::default()
            .bg(Color::Rgb(45, 45, 60))
            .fg(Color::Rgb(250, 250, 110))
            .add_modifier(Modifier::BOLD);
        let normal_style = Style::default()
            .bg(Color::Rgb(26, 26, 36))
            .fg(Color::Rgb(220, 220, 240));
        let mut rows = Vec::new();
        for patient in &self.filtered_patients {
            let selected_indicator = if let Some(selected) = &self.selected_patient {
                if selected.id == patient.id {
                    "✓"
                } else {
                    ""
                }
            } else {
                ""
            };
            rows.push(Row::new(vec![
                Cell::from(selected_indicator.to_string()).style(normal_style),
                Cell::from(patient.id.to_string()).style(normal_style),
                Cell::from(patient.first_name.clone()).style(normal_style),
                Cell::from(patient.last_name.clone()).style(normal_style),
                Cell::from(patient.phone_number.clone()).style(normal_style),
            ]));
        }
        if self.filtered_patients.is_empty() {
            let message = if self.search_input.is_empty() {
                "No patients found in database"
            } else {
                "No patients match your search criteria"
            };
            rows.push(Row::new(vec![
                Cell::from(""),
                Cell::from(""),
                Cell::from(message).style(Style::default().fg(Color::Rgb(180, 180, 200))),
                Cell::from(""),
                Cell::from(""),
            ]));
        }
        let table = Table::new(
            rows,
            [
                Constraint::Length(3),
                Constraint::Length(8),
                Constraint::Length(15),
                Constraint::Length(15),
                Constraint::Min(15),
            ],
        )
        .header(
            Row::new(vec![
                Cell::from(""),
                Cell::from("ID").style(Style::default().add_modifier(Modifier::BOLD)),
                Cell::from("First Name").style(Style::default().add_modifier(Modifier::BOLD)),
                Cell::from("Last Name").style(Style::default().add_modifier(Modifier::BOLD)),
                Cell::from("Phone").style(Style::default().add_modifier(Modifier::BOLD)),
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
        frame.render_stateful_widget(table, content_layout[1], &mut self.table_state.clone());
    }

    fn render_invoice_details_page(&self, frame: &mut Frame) {
        let area = frame.area();
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(12),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .margin(1)
            .split(area);
        let header = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(Color::Rgb(75, 75, 120)))
            .style(Style::default().bg(Color::Rgb(16, 16, 28)));
        frame.render_widget(header, layout[0]);
        let title = Paragraph::new("🧾 ADD INVOICE DETAILS")
            .style(
                Style::default()
                    .fg(Color::Rgb(230, 230, 250))
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Rgb(16, 16, 28)),
            )
            .alignment(Alignment::Center);
        frame.render_widget(title, layout[0]);
        self.render_invoice_form_fields(frame, layout[1]);
        self.render_status_message(frame, layout[3]);
        let submit_text = if self.focus_index == SUBMIT_BUTTON {
            "► Add Invoice ◄"
        } else {
            "  Add Invoice  "
        };
        let submit_style = if self.focus_index == SUBMIT_BUTTON {
            Style::default()
                .fg(Color::Rgb(140, 219, 140))
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Rgb(180, 180, 200))
        };
        frame.render_widget(
            Paragraph::new(submit_text)
                .style(submit_style)
                .alignment(Alignment::Center),
            layout[4],
        );
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
        frame.render_widget(
            Paragraph::new(back_text)
                .style(back_style)
                .alignment(Alignment::Center),
            layout[6],
        );
        frame.render_widget(
            Paragraph::new("Tab: Switch Focus, ↑/↓: Navigate | Enter: Submit | Esc: Back")
                .style(Style::default().fg(Color::Rgb(180, 180, 200)))
                .alignment(Alignment::Center),
            layout[9],
        );
    }

    fn render_invoice_form_fields(&self, frame: &mut Frame, area: Rect) {
        let form_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
            ])
            .horizontal_margin(3)
            .split(area);
        let required_style = Style::default().fg(Color::Rgb(230, 230, 250));
        let invoice_item_input = Paragraph::new(self.invoice_item.clone())
            .style(if self.focus_index == 0 {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Rgb(220, 220, 240))
            })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(Span::styled(" Item Description* ", required_style))
                    .border_style(if self.focus_index == 0 {
                        Style::default().fg(Color::Rgb(250, 250, 110))
                    } else {
                        Style::default().fg(Color::Rgb(140, 140, 200))
                    })
                    .style(Style::default().bg(Color::Rgb(26, 26, 36))),
            );
        frame.render_widget(invoice_item_input, form_layout[0]);
        let invoice_quantity_input = Paragraph::new(self.invoice_quantity.clone())
            .style(if self.focus_index == 1 {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Rgb(220, 220, 240))
            })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(Span::styled(" Quantity* ", required_style))
                    .border_style(if self.focus_index == 1 {
                        Style::default().fg(Color::Rgb(250, 250, 110))
                    } else {
                        Style::default().fg(Color::Rgb(140, 140, 200))
                    })
                    .style(Style::default().bg(Color::Rgb(26, 26, 36))),
            );
        frame.render_widget(invoice_quantity_input, form_layout[1]);
        let invoice_cost_input = Paragraph::new(self.invoice_cost.clone())
            .style(if self.focus_index == 2 {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Rgb(220, 220, 240))
            })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(Span::styled(" Cost* ", required_style))
                    .border_style(if self.focus_index == 2 {
                        Style::default().fg(Color::Rgb(250, 250, 110))
                    } else {
                        Style::default().fg(Color::Rgb(140, 140, 200))
                    })
                    .style(Style::default().bg(Color::Rgb(26, 26, 36))),
            );
        frame.render_widget(invoice_cost_input, form_layout[2]);
        let now = OffsetDateTime::now_utc();
        let formatted_date = format!(
            "  {}",
            now.format(&time::format_description::parse("[year]-[month]-[day]").unwrap())
                .unwrap()
        );
        let time_date_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(Span::styled(
                " Time & Date ",
                Style::default().fg(Color::Rgb(230, 230, 250)),
            ))
            .border_style(Style::default().fg(Color::Rgb(140, 140, 200)))
            .style(Style::default().bg(Color::Rgb(26, 26, 36)));
        let time_date_paragraph = Paragraph::new(formatted_date)
            .style(Style::default().fg(Color::Rgb(220, 220, 240)))
            .block(time_date_block);
        frame.render_widget(time_date_paragraph, form_layout[3]);
    }

    fn render_status_message(&self, frame: &mut Frame, area: Rect) {
        let status_message = if let Some(success) = &self.success_message {
            Paragraph::new(format!("✓ {}", success))
                .style(
                    Style::default()
                        .fg(Color::Rgb(140, 219, 140))
                        .add_modifier(Modifier::BOLD)
                        .bg(Color::Rgb(16, 16, 28)),
                )
                .alignment(Alignment::Center)
        } else if let Some(error) = &self.error_message {
            Paragraph::new(format!("⚠️ {}", error))
                .style(
                    Style::default()
                        .fg(Color::Rgb(255, 100, 100))
                        .add_modifier(Modifier::BOLD)
                        .bg(Color::Rgb(16, 16, 28)),
                )
                .alignment(Alignment::Center)
        } else {
            Paragraph::new("").style(Style::default().bg(Color::Rgb(16, 16, 28)))
        };
        frame.render_widget(status_message, area);
    }
}
