use crate::app::SelectedApp;
use crate::components::Component;
use crate::db;
use crate::models::{Invoice, Patient};
use crate::tui::Frame;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{prelude::*, widgets::*};
use std::collections::HashMap;
use std::time::{Duration, Instant};

enum ConfirmAction {
    UpdateInvoice,
}

enum UpdateState {
    SelectingInvoice,
    EditingInvoice,
}

pub struct UpdateInvoice {
    all_invoices: Vec<Invoice>,
    filtered_invoices: Vec<Invoice>,
    patients: HashMap<i64, Patient>,
    search_input: String,
    is_searching: bool,
    table_state: TableState,
    update_state: UpdateState,
    invoice_id_input: String,
    invoice: Invoice,
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
const ITEM_INPUT: usize = 2;
const QUANTITY_INPUT: usize = 3;
const COST_INPUT: usize = 4;
const INPUT_FIELDS: usize = 4;

impl UpdateInvoice {
    pub fn new() -> Self {
        let mut selection_state = TableState::default();
        selection_state.select(Some(0));

        let mut edit_table_state = TableState::default();
        edit_table_state.select(Some(0));

        Self {
            all_invoices: Vec::new(),
            filtered_invoices: Vec::new(),
            patients: HashMap::new(),
            search_input: String::new(),
            is_searching: false,
            table_state: selection_state,
            update_state: UpdateState::SelectingInvoice,
            invoice_id_input: String::new(),
            invoice: Invoice {
                id: 0,
                patient_id: 0,
                item: String::new(),
                quantity: 0,
                cost: 0.0,
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

    pub fn fetch_invoices(&mut self) -> Result<()> {
        self.all_invoices = db::get_all_invoices()?;
        self.fetch_patients_data()?;
        self.filter_invoices();
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

    fn get_patient(&self, patient_id: i64) -> Option<&Patient> {
        self.patients.get(&patient_id)
    }

    fn filter_invoices(&mut self) {
        if self.search_input.is_empty() {
            self.filtered_invoices = self.all_invoices.clone();
        } else {
            let search_term = self.search_input.to_lowercase();
            self.filtered_invoices = self
                .all_invoices
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

        if let Some(selected) = self.table_state.selected() {
            if selected >= self.filtered_invoices.len() && !self.filtered_invoices.is_empty() {
                self.table_state.select(Some(0));
            }
        }
    }

    fn load_invoice_by_id(&mut self, invoice_id: i64) -> Result<()> {
        match db::get_invoice(invoice_id) {
            Ok(invoice) => {
                self.invoice = invoice;
                self.loaded = true;
                self.update_state = UpdateState::EditingInvoice;
                self.update_input_value();
                Ok(())
            }
            Err(_) => {
                self.set_error(format!("Invoice with ID {} doesn't exist", invoice_id));
                Err(anyhow::anyhow!("Invoice not found"))
            }
        }
    }

    fn load_invoice(&mut self) -> Result<()> {
        if !self.loaded {
            if let Ok(invoice_id) = self.invoice_id_input.parse::<i64>() {
                self.load_invoice_by_id(invoice_id)
            } else {
                self.set_error("Invalid Invoice ID format.".to_string());
                Err(anyhow::anyhow!("Invalid Invoice ID format"))
            }
        } else {
            Ok(())
        }
    }

    fn load_selected_invoice(&mut self) -> Result<()> {
        if let Some(selected) = self.table_state.selected() {
            if selected < self.filtered_invoices.len() {
                let invoice_id = self.filtered_invoices[selected].id;
                self.invoice_id_input = invoice_id.to_string();
                return self.load_invoice_by_id(invoice_id);
            }
        }
        self.set_error("No invoice selected".to_string());
        Err(anyhow::anyhow!("No invoice selected"))
    }

    fn update_input_value(&mut self) {
        if !self.loaded {
            self.input_value = self.invoice_id_input.clone();
            return;
        }

        if let Some(field_index) = self.selected_field {
            self.input_value = match field_index {
                ID_INPUT => self.invoice.id.to_string(),
                PATIENT_ID_INPUT => self.invoice.patient_id.to_string(),
                ITEM_INPUT => self.invoice.item.clone(),
                QUANTITY_INPUT => self.invoice.quantity.to_string(),
                COST_INPUT => self.invoice.cost.to_string(),
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
                        self.invoice.patient_id = patient_id;
                    } else {
                        self.set_error("Invalid Patient ID format.".to_string());
                        return;
                    }
                }
                ITEM_INPUT => self.invoice.item = self.input_value.clone(),
                QUANTITY_INPUT => {
                    if let Ok(quantity) = self.input_value.parse::<i32>() {
                        self.invoice.quantity = quantity;
                    } else {
                        self.set_error("Invalid Quantity format.".to_string());
                        return;
                    }
                }
                COST_INPUT => {
                    if let Ok(cost) = self.input_value.parse::<f64>() {
                        self.invoice.cost = cost;
                    } else {
                        self.set_error("Invalid Cost format.".to_string());
                        return;
                    }
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

    fn update_invoice(&mut self) -> Result<()> {
        match db::update_invoice(&self.invoice) {
            Ok(_) => {
                self.success_message = Some("Invoice updated successfully!".to_string());
                self.success_timer = Some(Instant::now());

                if let Ok(invoices) = db::get_all_invoices() {
                    self.all_invoices = invoices.clone();
                    self.filtered_invoices = invoices;
                    self.filter_invoices();
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
        self.update_state = UpdateState::SelectingInvoice;
        self.loaded = false;
        self.invoice_id_input = String::new();
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
                        if let Some(ConfirmAction::UpdateInvoice) = self.confirmed_action.take() {
                            let _ = self.update_invoice();
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

        if matches!(self.update_state, UpdateState::SelectingInvoice) {
            match key.code {
                KeyCode::Char(c) if self.is_searching => {
                    self.search_input.push(c);
                    self.filter_invoices();
                    self.clear_error();
                }
                KeyCode::Backspace if self.is_searching => {
                    self.search_input.pop();
                    self.filter_invoices();
                    self.clear_error();
                }
                KeyCode::Down if self.is_searching && !self.filtered_invoices.is_empty() => {
                    self.is_searching = false;
                    self.table_state.select(Some(0));
                }
                KeyCode::Esc if self.is_searching => {
                    self.is_searching = false;
                    self.search_input.clear();
                    self.filter_invoices();
                }
                KeyCode::Char('/') | KeyCode::Char('s') | KeyCode::Char('S')
                    if !self.is_searching =>
                {
                    self.is_searching = true;
                }

                KeyCode::Char(c) if !self.is_searching => {
                    self.invoice_id_input.push(c);
                    self.input_value = self.invoice_id_input.clone();
                    self.clear_error();
                }
                KeyCode::Backspace if !self.is_searching => {
                    self.invoice_id_input.pop();
                    self.input_value = self.invoice_id_input.clone();
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
                    if selected < self.filtered_invoices.len().saturating_sub(1) {
                        self.table_state.select(Some(selected + 1));
                    }
                }

                KeyCode::Enter => {
                    if self.is_searching {
                        if !self.filtered_invoices.is_empty() {
                            self.is_searching = false;
                            self.table_state.select(Some(0));
                        }
                    } else {
                        if !self.invoice_id_input.is_empty() {
                            let _ = self.load_invoice();
                        } else if !self.filtered_invoices.is_empty() {
                            let _ = self.load_selected_invoice();
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
                    "Are you sure you want to update this invoice?".to_string(),
                    ConfirmAction::UpdateInvoice,
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

impl Default for UpdateInvoice {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for UpdateInvoice {
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
            UpdateState::SelectingInvoice => self.render_invoice_selection(frame, area),
            UpdateState::EditingInvoice => self.render_invoice_editing(frame, area),
        }

        if self.show_confirmation {
            self.render_confirmation_dialog(frame, area);
        }
    }
}

impl UpdateInvoice {
    fn render_invoice_selection(&self, frame: &mut Frame, area: Rect) {
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

        let title = Paragraph::new("✍️  SELECT INVOICE TO UPDATE")
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
                " Search Invoices ",
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
                " Invoice ID ",
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

        let id_input_paragraph = Paragraph::new(self.invoice_id_input.clone())
            .style(
                Style::default()
                    .fg(Color::Rgb(220, 220, 240))
                    .bg(Color::Rgb(26, 26, 36)),
            )
            .block(id_input_block);
        frame.render_widget(id_input_paragraph, main_layout[2]);

        if self.filtered_invoices.is_empty() {
            let no_invoices = Paragraph::new(if self.search_input.is_empty() {
                "No invoices found in database"
            } else {
                "No invoices match your search criteria"
            })
            .style(Style::default().fg(Color::Rgb(220, 220, 240)))
            .alignment(Alignment::Center);
            frame.render_widget(no_invoices, main_layout[3]);
        } else {
            let invoices_rows: Vec<Row> = self
                .filtered_invoices
                .iter()
                .map(|invoice| {
                    let patient_name = match self.get_patient(invoice.patient_id) {
                        Some(patient) => format!("{} {}", patient.first_name, patient.last_name),
                        None => "Unknown Patient".to_string(),
                    };

                    Row::new(vec![
                        invoice.id.to_string(),
                        patient_name,
                        invoice.item.clone(),
                        invoice.quantity.to_string(),
                        format!("{:.2}", invoice.cost),
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

            let header = Row::new(vec!["ID", "Patient", "Item", "Quantity", "Cost"])
                .style(
                    Style::default()
                        .fg(Color::Rgb(220, 220, 240))
                        .bg(Color::Rgb(80, 60, 130))
                        .add_modifier(Modifier::BOLD),
                )
                .height(1);

            let widths = [
                Constraint::Percentage(10),
                Constraint::Percentage(30),
                Constraint::Percentage(30),
                Constraint::Percentage(10),
                Constraint::Percentage(20),
            ];

            let invoices_table = Table::new(invoices_rows, widths)
                .header(header)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .title(format!(" Invoices ({}) ", self.filtered_invoices.len()))
                        .border_style(Style::default().fg(Color::Rgb(140, 140, 200)))
                        .style(Style::default().bg(Color::Rgb(26, 26, 36))),
                )
                .column_spacing(2)
                .row_highlight_style(selected_style)
                .highlight_symbol("► ");

            frame.render_stateful_widget(
                invoices_table,
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
            "/ or s: Search | ↑/↓: Navigate invoices | Enter: Select invoice | Esc: Back"
        };

        let help_paragraph = Paragraph::new(help_text)
            .style(Style::default().fg(Color::Rgb(220, 220, 240)))
            .alignment(Alignment::Center);
        frame.render_widget(help_paragraph, main_layout[5]);
    }

    fn render_invoice_editing(&self, frame: &mut Frame, area: Rect) {
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
            "✍️  EDITING INVOICE"
        } else {
            "✍️  UPDATE INVOICE"
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

        let id_str = self.invoice.id.to_string();
        let patient_id_str = self.invoice.patient_id.to_string();
        let quantity_str = self.invoice.quantity.to_string();
        let cost_str = format!("{:.2}", self.invoice.cost);

        let table_items = vec![
            Row::new(vec!["ID", &id_str])
                .style(Style::default().fg(Color::Rgb(220, 220, 240)))
                .height(1)
                .bottom_margin(0),
            Row::new(vec!["Patient ID", &patient_id_str])
                .style(Style::default().fg(Color::Rgb(220, 220, 240)))
                .height(1)
                .bottom_margin(0),
            Row::new(vec!["Item", &self.invoice.item])
                .style(Style::default().fg(Color::Rgb(220, 220, 240)))
                .height(1)
                .bottom_margin(0),
            Row::new(vec!["Quantity", &quantity_str])
                .style(Style::default().fg(Color::Rgb(220, 220, 240)))
                .height(1)
                .bottom_margin(0),
            Row::new(vec!["Cost", &cost_str])
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
                    .title(" Invoice Data ")
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
            Some(ITEM_INPUT) => "Item",
            Some(QUANTITY_INPUT) => "Quantity",
            Some(COST_INPUT) => "Cost",
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
            "↑/↓: Navigate | E: Edit Field | Ctrl+S: Save Invoice | Esc: Back"
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
            .title(" Update Invoice ")
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
