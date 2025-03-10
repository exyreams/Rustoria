use crate::app::SelectedApp;
use crate::components::Component;
use crate::db;
use crate::models::{Invoice, Patient};
use crate::tui::Frame;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{prelude::*, widgets::*};
use std::collections::HashMap;
const SEARCH_FIELD: usize = 0;
const PATIENT_LIST: usize = 1;
const BACK_BUTTON: usize = 2;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewState {
    ViewingList,
    ViewingDetails,
}
pub struct ViewInvoices {
    invoices: Vec<Invoice>,
    filtered_invoices: Vec<Invoice>,
    patients: HashMap<i64, Patient>,
    search_input: String,
    is_searching: bool,
    state: TableState,
    error_message: Option<String>,
    focus_index: usize,
    view_state: ViewState,
    selected_patient_id: Option<i64>,
}
impl ViewInvoices {
    pub fn new() -> Self {
        Self {
            invoices: Vec::new(),
            filtered_invoices: Vec::new(),
            patients: HashMap::new(),
            search_input: String::new(),
            is_searching: false,
            state: TableState::default(),
            error_message: None,
            focus_index: PATIENT_LIST,
            view_state: ViewState::ViewingList,
            selected_patient_id: None,
        }
    }
    pub fn fetch_invoices(&mut self) -> Result<()> {
        self.invoices = db::get_all_invoices()?;
        self.fetch_patients_data()?;
        self.filter_invoices();
        if self.invoices.is_empty() {
            self.state.select(None);
        } else {
            self.state.select(Some(0));
        }
        self.error_message = None;
        Ok(())
    }
    fn fetch_patients_data(&mut self) -> Result<()> {
        self.patients.clear();
        let all_patients = db::get_all_patients()?;
        for patient in all_patients {
            self.patients.insert(patient.id, patient);
        }
        Ok(())
    }
    fn filter_invoices(&mut self) {
        if self.search_input.is_empty() {
            self.filtered_invoices = self.invoices.clone();
        } else {
            let search_term = self.search_input.to_lowercase();
            self.filtered_invoices = self
                .invoices
                .iter()
                .filter(|invoice| {
                    let patient_name_match =
                        if let Some(patient) = self.patients.get(&invoice.patient_id) {
                            patient.first_name.to_lowercase().contains(&search_term)
                                || patient.last_name.to_lowercase().contains(&search_term)
                        } else {
                            false
                        };
                    invoice.patient_id.to_string().contains(&search_term)
                        || invoice.item.to_lowercase().contains(&search_term)
                        || patient_name_match
                })
                .cloned()
                .collect();
        }
    }
    fn select_next(&mut self) {
        let mut aggregated_invoices: HashMap<i64, f64> = HashMap::new();
        for invoice in &self.filtered_invoices {
            *aggregated_invoices.entry(invoice.patient_id).or_insert(0.0) += invoice.cost;
        }
        let mut sorted_patients: Vec<i64> = aggregated_invoices.keys().cloned().collect();
        sorted_patients.sort();
        if sorted_patients.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i >= sorted_patients.len() - 1 {
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
        let mut aggregated_invoices: HashMap<i64, f64> = HashMap::new();
        for invoice in &self.filtered_invoices {
            *aggregated_invoices.entry(invoice.patient_id).or_insert(0.0) += invoice.cost;
        }
        let mut sorted_patients: Vec<i64> = aggregated_invoices.keys().cloned().collect();
        sorted_patients.sort();
        if sorted_patients.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    sorted_patients.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
    fn view_invoice_details(&mut self) {
        if let Some(selected_index) = self.state.selected() {
            let mut aggregated_invoices: HashMap<i64, f64> = HashMap::new();
            for invoice in &self.filtered_invoices {
                *aggregated_invoices.entry(invoice.patient_id).or_insert(0.0) += invoice.cost;
            }
            let mut patient_ids: Vec<_> = aggregated_invoices.keys().cloned().collect();
            patient_ids.sort();
            if let Some(patient_id) = patient_ids.get(selected_index) {
                self.selected_patient_id = Some(*patient_id);
                self.view_state = ViewState::ViewingDetails;
            }
        }
    }
    fn return_to_list(&mut self) {
        self.view_state = ViewState::ViewingList;
        self.selected_patient_id = None;
    }
    fn focus_next(&mut self) {
        self.focus_index = (self.focus_index + 1) % 3;
        self.is_searching = self.focus_index == SEARCH_FIELD;
    }
    fn focus_previous(&mut self) {
        self.focus_index = (self.focus_index + 2) % 3;
        self.is_searching = self.focus_index == SEARCH_FIELD;
    }
    fn get_patient(&self, patient_id: i64) -> Option<&Patient> {
        self.patients.get(&patient_id)
    }
    pub fn handle_input(&mut self, key: KeyEvent) -> Result<Option<SelectedApp>> {
        match self.view_state {
            ViewState::ViewingList => {
                if self.is_searching {
                    match key.code {
                        KeyCode::Char(c) => {
                            self.search_input.push(c);
                            self.filter_invoices();
                        }
                        KeyCode::Backspace => {
                            self.search_input.pop();
                            self.filter_invoices();
                        }
                        KeyCode::Enter | KeyCode::Down | KeyCode::Tab => {
                            if !self.filtered_invoices.is_empty() {
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
                            return Ok(Some(SelectedApp::None));
                        } else if self.focus_index == PATIENT_LIST {
                            self.view_invoice_details();
                        } else if self.focus_index == SEARCH_FIELD {
                            self.is_searching = true;
                        }
                    }
                    KeyCode::Char('b') | KeyCode::Char('B') => {
                        return Ok(Some(SelectedApp::None));
                    }
                    KeyCode::Char('r') | KeyCode::Char('R') => {
                        self.fetch_invoices()?;
                    }
                    KeyCode::Esc => {
                        return Ok(Some(SelectedApp::None));
                    }
                    _ => {}
                }
            }
            ViewState::ViewingDetails => match key.code {
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
}
impl Component for ViewInvoices {
    fn handle_input(&mut self, event: KeyEvent) -> Result<Option<SelectedApp>> {
        self.handle_input(event)
    }
    fn render(&self, frame: &mut Frame) {
        frame.render_widget(
            Block::default().style(Style::default().bg(Color::Rgb(16, 16, 28))),
            frame.area(),
        );
        match self.view_state {
            ViewState::ViewingList => self.render_list_view(frame),
            ViewState::ViewingDetails => self.render_details_view(frame),
        }
    }
}
impl ViewInvoices {
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
        let title = Paragraph::new("🧾 Invoices")
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
                " Search Patients ",
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
            .style(Style::default().fg(Color::Rgb(220, 220, 240)))
            .block(search_block);
        frame.render_widget(search_paragraph, layout[1]);
        let mut aggregated_invoices: HashMap<i64, (String, f64)> = HashMap::new();
        for invoice in &self.filtered_invoices {
            if let Some(patient) = self.get_patient(invoice.patient_id) {
                let full_name = format!("{} {}", patient.first_name, patient.last_name);
                let entry = aggregated_invoices
                    .entry(patient.id)
                    .or_insert((full_name, 0.0));
                entry.1 += invoice.cost;
            }
        }
        let mut sorted_invoices: Vec<_> = aggregated_invoices.into_iter().collect();
        sorted_invoices.sort_by_key(|&(patient_id, _)| patient_id);
        let rows = sorted_invoices
            .iter()
            .map(|(patient_id, (patient_name, total_cost))| {
                let cells = vec![
                    Cell::from(patient_id.to_string()),
                    Cell::from(patient_name.clone()),
                    Cell::from(format!("${:.2}", total_cost)),
                ];
                Row::new(cells)
                    .height(1)
                    .bottom_margin(0)
                    .style(Style::default().fg(Color::Rgb(220, 220, 240)))
            });
        let header_cells = ["Patient ID", "Patient Name", "Total Cost"]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().fg(Color::Rgb(230, 230, 250))));
        let header = Row::new(header_cells)
            .style(Style::default().bg(Color::Rgb(80, 60, 130)))
            .height(1);
        let selected_style = Style::default()
            .fg(Color::Rgb(250, 250, 110))
            .bg(Color::Rgb(40, 40, 60))
            .add_modifier(Modifier::BOLD);
        let table_title = if !self.search_input.is_empty() {
            format!(
                " Patients with Invoices ({} of {} patients with invoices) ",
                sorted_invoices.len(),
                self.patients.len()
            )
        } else {
            format!(" Patients with Invoices ({}) ", self.patients.len())
        };
        let table = Table::new(
            rows,
            [
                Constraint::Percentage(20),
                Constraint::Percentage(50),
                Constraint::Percentage(30),
            ],
        )
        .header(header)
        .block(
            Block::default()
                .title(table_title)
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Rgb(75, 75, 120)))
                .style(Style::default().bg(Color::Rgb(22, 22, 35))),
        )
        .row_highlight_style(selected_style)
        .highlight_symbol(if self.focus_index == PATIENT_LIST {
            "► "
        } else {
            "  "
        });
        frame.render_stateful_widget(table, layout[2], &mut self.state.clone());
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
            "/ or s: Search | ↑↓: Navigate | Enter: View Details | r: Refresh | Tab: Focus"
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
                Constraint::Min(10),
                Constraint::Length(3),
                Constraint::Length(1),
            ])
            .margin(1)
            .split(area);
        let header_block = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(Color::Rgb(75, 75, 120)))
            .style(Style::default().bg(Color::Rgb(16, 16, 28)));
        frame.render_widget(header_block, layout[0]);
        let title = Paragraph::new("🧾 INVOICE DETAILS")
            .style(
                Style::default()
                    .fg(Color::Rgb(230, 230, 250))
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Rgb(16, 16, 28)),
            )
            .alignment(Alignment::Center);
        frame.render_widget(title, layout[0]);
        if let Some(patient_id) = self.selected_patient_id {
            let patient_name = self
                .get_patient(patient_id)
                .map(|p| format!("{} {}", p.first_name, p.last_name))
                .unwrap_or_else(|| "Unknown Patient".to_string());
            let invoices_for_patient: Vec<&Invoice> = self
                .invoices
                .iter()
                .filter(|inv| inv.patient_id == patient_id)
                .collect();
            let header_cells = ["Item", "Quantity", "Cost"].iter().map(|h| {
                Cell::from(format!("  {}", h)).style(Style::default().fg(Color::Rgb(230, 230, 250)))
            });
            let header = Row::new(header_cells)
                .style(Style::default().bg(Color::Rgb(80, 60, 130)))
                .height(1);
            let rows = invoices_for_patient.iter().map(|invoice| {
                let cells = vec![
                    Cell::from(format!("  {}", invoice.item))
                        .style(Style::default().fg(Color::Rgb(220, 220, 240))),
                    Cell::from(format!("  {}", invoice.quantity))
                        .style(Style::default().fg(Color::Rgb(220, 220, 240))),
                    Cell::from(format!("  ${:.2}", invoice.cost))
                        .style(Style::default().fg(Color::Rgb(220, 220, 240))),
                ];
                Row::new(cells).height(1).bottom_margin(0)
            });
            let title_text = vec![
                Span::styled(
                    "Invoices for Patient: ",
                    Style::default().fg(Color::Rgb(230, 230, 250)),
                ),
                Span::styled(
                    patient_name,
                    Style::default()
                        .fg(Color::Rgb(250, 250, 110))
                        .add_modifier(Modifier::BOLD),
                ),
            ];
            let title = Line::from(title_text);
            let table = Table::new(
                rows,
                [
                    Constraint::Percentage(50),
                    Constraint::Percentage(20),
                    Constraint::Percentage(30),
                ],
            )
            .header(header)
            .block(
                Block::default()
                    .title(title)
                    .title_alignment(Alignment::Center)
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Rgb(75, 75, 120)))
                    .style(Style::default().bg(Color::Rgb(22, 22, 35))),
            )
            .row_highlight_style(
                Style::default()
                    .fg(Color::Rgb(250, 250, 110))
                    .bg(Color::Rgb(40, 40, 60))
                    .add_modifier(Modifier::BOLD),
            );
            frame.render_widget(table, layout[1]);
            let total_cost: f64 = invoices_for_patient
                .iter()
                .map(|invoice| invoice.cost)
                .sum();
            let total_cost_paragraph = Paragraph::new(format!("Total Cost: ${:.2}", total_cost))
                .style(
                    Style::default()
                        .fg(Color::Rgb(140, 219, 140))
                        .add_modifier(Modifier::BOLD),
                )
                .alignment(Alignment::Center);
            frame.render_widget(total_cost_paragraph, layout[2]);
        } else {
            frame.render_widget(
                Paragraph::new("No patient selected.")
                    .style(Style::default().fg(Color::Red))
                    .alignment(Alignment::Center),
                layout[1],
            );
        }
        let help_text = "Enter/Esc/Backspace: Return to list";
        let help_paragraph = Paragraph::new(help_text)
            .style(Style::default().fg(Color::Rgb(140, 140, 170)))
            .alignment(Alignment::Center);
        frame.render_widget(help_paragraph, layout[3]);
    }
}
impl Default for ViewInvoices {
    fn default() -> Self {
        Self::new()
    }
}
