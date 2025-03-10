use crate::components::hospital::patients::PatientAction;
use crate::components::Component;
use crate::db;
use crate::models::Patient;
use crate::tui::Frame;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{prelude::*, widgets::*};

const SEARCH_FIELD: usize = 0;
const PATIENT_LIST: usize = 1;
const BACK_BUTTON: usize = 2;

pub struct ListPatients {
    patients: Vec<Patient>,
    filtered_patients: Vec<Patient>,
    search_input: String,
    is_searching: bool,
    state: TableState,
    error_message: Option<String>,
    show_details: bool,
    focus_index: usize,
}

impl ListPatients {
    pub fn new() -> Self {
        Self {
            patients: Vec::new(),
            filtered_patients: Vec::new(),
            search_input: String::new(),
            is_searching: false,
            state: TableState::default(),
            error_message: None,
            show_details: false,
            focus_index: PATIENT_LIST,
        }
    }

    pub fn fetch_patients(&mut self) -> Result<()> {
        match db::get_all_patients() {
            Ok(patients) => {
                self.patients = patients;
                self.filter_patients();

                if self.filtered_patients.is_empty() {
                    self.state.select(None);
                } else {
                    let selection = self
                        .state
                        .selected()
                        .unwrap_or(0)
                        .min(self.filtered_patients.len() - 1);
                    self.state.select(Some(selection));
                }
                self.error_message = None;
                Ok(())
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to fetch patients: {}", e));
                Ok(())
            }
        }
    }

    fn filter_patients(&mut self) {
        if self.search_input.is_empty() {
            self.filtered_patients = self.patients.clone();
        } else {
            let search_term = self.search_input.to_lowercase();
            self.filtered_patients = self
                .patients
                .iter()
                .filter(|p| {
                    p.first_name.to_lowercase().contains(&search_term)
                        || p.last_name.to_lowercase().contains(&search_term)
                        || p.id.to_string().contains(&search_term)
                        || p.phone_number.to_lowercase().contains(&search_term)
                        || p.address.to_lowercase().contains(&search_term)
                })
                .cloned()
                .collect();
        }

        if let Some(selected) = self.state.selected() {
            if selected >= self.filtered_patients.len() && !self.filtered_patients.is_empty() {
                self.state.select(Some(0));
            } else if self.filtered_patients.is_empty() {
                self.state.select(None);
            }
        }
    }

    fn select_next(&mut self) {
        if self.filtered_patients.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.filtered_patients.len() - 1 {
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
        if self.filtered_patients.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.filtered_patients.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn toggle_details(&mut self) {
        if !self.filtered_patients.is_empty() && self.state.selected().is_some() {
            self.show_details = !self.show_details;
        }
    }

    fn focus_next(&mut self) {
        self.focus_index = (self.focus_index + 1) % 3;
        if self.focus_index == SEARCH_FIELD {
            self.is_searching = true;
        } else {
            self.is_searching = false;
        }
    }

    fn focus_previous(&mut self) {
        self.focus_index = (self.focus_index + 2) % 3;
        if self.focus_index == SEARCH_FIELD {
            self.is_searching = true;
        } else {
            self.is_searching = false;
        }
    }

    pub fn handle_input(&mut self, key: KeyEvent) -> Result<Option<PatientAction>> {
        if self.is_searching {
            match key.code {
                KeyCode::Char(c) => {
                    self.search_input.push(c);
                    self.filter_patients();
                }
                KeyCode::Backspace => {
                    self.search_input.pop();
                    self.filter_patients();
                }
                KeyCode::Enter | KeyCode::Down | KeyCode::Tab => {
                    if !self.filtered_patients.is_empty() {
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
                    return Ok(Some(PatientAction::BackToHome));
                } else if self.focus_index == PATIENT_LIST {
                    self.toggle_details();
                } else if self.focus_index == SEARCH_FIELD {
                    self.is_searching = true;
                }
            }
            KeyCode::Char('b') | KeyCode::Char('B') => {
                return Ok(Some(PatientAction::BackToHome));
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                self.fetch_patients()?;
            }
            KeyCode::Esc => {
                if self.show_details {
                    self.show_details = false;
                } else {
                    return Ok(Some(PatientAction::BackToHome));
                }
            }
            _ => {}
        }
        Ok(None)
    }

    fn selected_patient(&self) -> Option<&Patient> {
        self.state
            .selected()
            .and_then(|i| self.filtered_patients.get(i))
    }
}

impl Component for ListPatients {
    fn handle_input(&mut self, event: KeyEvent) -> Result<Option<crate::app::SelectedApp>> {
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

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(3),
                Constraint::Length(2),
                Constraint::Length(1),
            ])
            .margin(1)
            .split(area);

        let header_block = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(Color::Rgb(75, 75, 120)))
            .style(Style::default().bg(Color::Rgb(16, 16, 28)));
        frame.render_widget(header_block, layout[0]);

        let title = Paragraph::new("🏥 PATIENT LIST")
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
            .style(
                Style::default()
                    .fg(Color::Rgb(220, 220, 240))
                    .bg(Color::Rgb(22, 22, 35)),
            )
            .block(search_block);
        frame.render_widget(search_paragraph, layout[1]);

        let header_cells = [
            "ID",
            "First Name",
            "Last Name",
            "Date of Birth",
            "Gender",
            "Phone",
            "Address",
        ]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Rgb(230, 230, 250))));
        let header = Row::new(header_cells)
            .style(Style::default().bg(Color::Rgb(80, 60, 130)))
            .height(1);

        let rows = self.filtered_patients.iter().map(|patient| {
            let cells = vec![
                Cell::from(patient.id.to_string()),
                Cell::from(patient.first_name.clone()),
                Cell::from(patient.last_name.clone()),
                Cell::from(patient.date_of_birth.clone()),
                Cell::from(match patient.gender {
                    crate::models::Gender::Male => "Male",
                    crate::models::Gender::Female => "Female",
                    crate::models::Gender::Other => "Other",
                }),
                Cell::from(patient.phone_number.clone()),
                Cell::from(patient.address.clone()),
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
                " Patients ({} of {} matches) ",
                self.filtered_patients.len(),
                self.patients.len()
            )
        } else {
            format!(" Patients ({}) ", self.patients.len())
        };

        let table = Table::new(
            rows,
            [
                Constraint::Percentage(5),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(10),
                Constraint::Percentage(10),
                Constraint::Percentage(15),
                Constraint::Percentage(30),
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
        .highlight_symbol(if self.focus_index == PATIENT_LIST {
            "► "
        } else {
            "  "
        });

        if self.filtered_patients.is_empty() {
            let message = if self.search_input.is_empty() {
                "No patients found in database"
            } else {
                "No patients match your search criteria"
            };

            let no_patients = Paragraph::new(message)
                .style(Style::default().fg(Color::Rgb(220, 220, 240)))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .title(table_title)
                        .title_alignment(Alignment::Center)
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(Color::Rgb(75, 75, 120)))
                        .style(Style::default().bg(Color::Rgb(22, 22, 35))),
                );
            frame.render_widget(no_patients, layout[2]);
        } else {
            frame.render_stateful_widget(table, layout[2], &mut self.state.clone());
        }

        if self.show_details && self.state.selected().is_some() {
            if let Some(patient) = self.selected_patient() {
                let details = format!(
                    "Details for {} {}: Born on {}, Gender: {}, Phone: {}, Address: {}",
                    patient.first_name,
                    patient.last_name,
                    patient.date_of_birth,
                    match patient.gender {
                        crate::models::Gender::Male => "Male",
                        crate::models::Gender::Female => "Female",
                        crate::models::Gender::Other => "Other",
                    },
                    patient.phone_number,
                    patient.address
                );

                let details_widget = Paragraph::new(details)
                    .style(Style::default().fg(Color::Rgb(200, 200, 220)))
                    .block(
                        Block::default()
                            .title(" Patient Details ")
                            .borders(Borders::ALL)
                            .border_type(BorderType::Rounded)
                            .border_style(Style::default().fg(Color::Rgb(75, 75, 120))),
                    )
                    .wrap(Wrap { trim: true });

                frame.render_widget(details_widget, layout[3]);
            }
        } else {
            let help_text = if self.is_searching {
                "Type to search | ↓/Enter: To results | Esc: Cancel search"
            } else {
                "/ or s: Search | ↑↓: Navigate | Enter: View Details | R: Refresh | Tab: Focus"
            };

            let help_paragraph = Paragraph::new(help_text)
                .style(Style::default().fg(Color::Rgb(140, 140, 170)))
                .alignment(Alignment::Center);
            frame.render_widget(help_paragraph, layout[3]);
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

        if let Some(error) = &self.error_message {
            let error_paragraph = Paragraph::new(error.as_str())
                .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
                .alignment(Alignment::Center);
            frame.render_widget(error_paragraph, layout[5]);
        }
    }
}

impl Default for ListPatients {
    fn default() -> Self {
        Self::new()
    }
}
