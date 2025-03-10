use crate::components::hospital::patients::PatientAction;
use crate::components::Component;
use crate::db;
use crate::models::{Gender, Patient};
use crate::tui::Frame;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{prelude::*, widgets::*};
use std::time::{Duration, Instant};

pub struct AddPatient {
    first_name: String,
    last_name: String,
    dob: String,
    gender: Gender,
    address: String,
    phone: String,
    email: Option<String>,
    medical_history: Option<String>,
    allergies: Option<String>,
    medications: Option<String>,
    focus_index: usize,
    error_message: Option<String>,
    error_timer: Option<Instant>,
    success_message: Option<String>,
    success_timer: Option<Instant>,
}

const INPUT_FIELDS: usize = 10;

impl Default for AddPatient {
    fn default() -> Self {
        AddPatient {
            first_name: String::new(),
            last_name: String::new(),
            dob: String::new(),
            gender: Gender::Male,
            address: String::new(),
            phone: String::new(),
            email: None,
            medical_history: None,
            allergies: None,
            medications: None,
            focus_index: 0,
            error_message: None,
            error_timer: None,
            success_message: None,
            success_timer: None,
        }
    }
}

impl AddPatient {
    pub fn new() -> Self {
        Self::default()
    }

    fn clear_error(&mut self) {
        self.error_message = None;
        self.error_timer = None;
    }

    fn set_error(&mut self, message: String) {
        self.error_message = Some(message);
        self.error_timer = Some(Instant::now());
    }

    pub fn check_error_timeout(&mut self) {
        if let Some(timer) = self.error_timer {
            if timer.elapsed() > Duration::from_secs(5) {
                self.clear_error();
            }
        }
    }

    fn clear_success(&mut self) {
        self.success_message = None;
        self.success_timer = None;
    }

    #[allow(dead_code)]
    fn set_success(&mut self, message: String) {
        self.success_message = Some(message);
        self.success_timer = Some(Instant::now());
    }

    pub fn check_success_timeout(&mut self) {
        if let Some(timer) = self.success_timer {
            if timer.elapsed() > Duration::from_secs(5) {
                self.clear_success();
            }
        }
    }

    pub fn handle_input(&mut self, key: KeyEvent) -> Result<Option<PatientAction>> {
        self.check_error_timeout();
        self.check_success_timeout();
        match key.code {
            KeyCode::Char(c) => {
                match self.focus_index {
                    0 => self.first_name.push(c),
                    1 => self.last_name.push(c),
                    2 => self.dob.push(c),
                    3 => {
                        if c.to_ascii_lowercase() == 'f' {
                            self.gender = Gender::Female;
                        } else if c.to_ascii_lowercase() == 'm' {
                            self.gender = Gender::Male;
                        } else if c.to_ascii_lowercase() == 'o' {
                            self.gender = Gender::Other;
                        }
                    }
                    4 => self.address.push(c),
                    5 => self.phone.push(c),
                    6 => {
                        if let Some(ref mut email) = self.email {
                            email.push(c);
                        } else {
                            self.email = Some(c.to_string());
                        }
                    }
                    7 => {
                        if let Some(ref mut history) = self.medical_history {
                            history.push(c);
                        } else {
                            self.medical_history = Some(c.to_string());
                        }
                    }
                    8 => {
                        if let Some(ref mut allergies) = self.allergies {
                            allergies.push(c);
                        } else {
                            self.allergies = Some(c.to_string());
                        }
                    }
                    9 => {
                        if let Some(ref mut medications) = self.medications {
                            medications.push(c);
                        } else {
                            self.medications = Some(c.to_string());
                        }
                    }
                    _ => {}
                }
                self.clear_error();
            }
            KeyCode::Backspace => {
                match self.focus_index {
                    0 => self.first_name.pop(),
                    1 => self.last_name.pop(),
                    2 => self.dob.pop(),
                    3 => None,
                    4 => self.address.pop(),
                    5 => self.phone.pop(),
                    6 => self.email.as_mut().and_then(|email| email.pop()),
                    7 => {
                        if let Some(ref mut history) = self.medical_history {
                            history.pop()
                        } else {
                            None
                        }
                    }
                    8 => {
                        if let Some(ref mut allergies) = self.allergies {
                            allergies.pop()
                        } else {
                            None
                        }
                    }
                    9 => {
                        if let Some(ref mut medications) = self.medications {
                            medications.pop()
                        } else {
                            None
                        }
                    }
                    _ => None,
                };
                self.clear_error();
            }
            KeyCode::Tab => {
                if self.focus_index <= 9 {
                    self.focus_index = INPUT_FIELDS;
                } else if self.focus_index == INPUT_FIELDS {
                    self.focus_index = INPUT_FIELDS + 1;
                } else {
                    self.focus_index = 0;
                }
            }
            KeyCode::Down => {
                self.focus_index = (self.focus_index + 1) % (INPUT_FIELDS + 2);
            }
            KeyCode::Up => {
                self.focus_index = (self.focus_index + INPUT_FIELDS + 1) % (INPUT_FIELDS + 2);
            }
            KeyCode::Left => {
                if self.focus_index >= 6 && self.focus_index <= 9 {
                    self.focus_index -= 6;
                }
            }
            KeyCode::Right => {
                if self.focus_index <= 5 {
                    self.focus_index = std::cmp::min(self.focus_index + 6, 9);
                }
            }
            KeyCode::Esc => {
                return Ok(Some(PatientAction::BackToHome));
            }
            KeyCode::Enter => {
                if self.focus_index == INPUT_FIELDS + 1 {
                    return Ok(Some(PatientAction::BackToHome));
                } else if self.focus_index == INPUT_FIELDS {
                    if self.first_name.is_empty() {
                        self.set_error("First Name cannot be empty".to_string());
                        return Ok(None);
                    }
                    if self.last_name.is_empty() {
                        self.set_error("Last Name cannot be empty".to_string());
                        return Ok(None);
                    }
                    if self.dob.is_empty() {
                        self.set_error("Date of Birth cannot be empty".to_string());
                        return Ok(None);
                    }
                    if self.address.is_empty() {
                        self.set_error("Address cannot be empty".to_string());
                        return Ok(None);
                    }
                    if self.phone.is_empty() {
                        self.set_error("Phone Number cannot be empty".to_string());
                        return Ok(None);
                    }

                    let new_patient = Patient {
                        id: 0,
                        first_name: self.first_name.clone(),
                        last_name: self.last_name.clone(),
                        date_of_birth: self.dob.clone(),
                        gender: self.gender.clone(),
                        address: self.address.clone(),
                        phone_number: self.phone.clone(),
                        email: self.email.clone(),
                        medical_history: self.medical_history.clone(),
                        allergies: self.allergies.clone(),
                        current_medications: self.medications.clone(),
                    };

                    match db::create_patient(&new_patient) {
                        Ok(_) => {
                            self.first_name.clear();
                            self.last_name.clear();
                            self.dob.clear();
                            self.gender = Gender::Male;
                            self.address.clear();
                            self.phone.clear();
                            self.email = None;
                            self.medical_history = None;
                            self.allergies = None;
                            self.medications = None;
                            self.focus_index = 0;

                            self.success_message = Some("Patient added successfully!".to_string());

                            self.clear_error();
                        }
                        Err(e) => {
                            self.set_error(format!("Database error: {}", e));
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(None)
    }
}

impl Component for AddPatient {
    fn handle_input(&mut self, event: KeyEvent) -> Result<Option<crate::app::SelectedApp>> {
        match self.handle_input(event)? {
            Some(PatientAction::BackToHome) | Some(PatientAction::BackToList) => {
                Ok(Some(crate::app::SelectedApp::None))
            }
            None => Ok(None),
        }
    }

    fn render(&self, frame: &mut Frame) {
        let area = frame.area();
        frame.render_widget(
            Block::default().style(Style::default().bg(Color::Rgb(16, 16, 28))),
            area,
        );

        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(22),
                Constraint::Length(0),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(6),
            ])
            .margin(1)
            .split(frame.area());

        let header = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(Color::Rgb(75, 75, 120)))
            .style(Style::default().bg(Color::Rgb(16, 16, 28)));
        frame.render_widget(header, main_layout[0]);

        let title = Paragraph::new("🏥 PATIENT REGISTRATION")
            .style(
                Style::default()
                    .fg(Color::Rgb(230, 230, 250))
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Rgb(16, 16, 28)),
            )
            .alignment(Alignment::Center);
        frame.render_widget(title, main_layout[0]);

        let body_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Rgb(75, 75, 120)))
            .style(Style::default().bg(Color::Rgb(22, 22, 35)));

        frame.render_widget(body_block.clone(), main_layout[1]);
        let body_inner = body_block.inner(main_layout[1]);

        let body_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(body_inner);

        let left_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
            ])
            .margin(1)
            .split(body_layout[0]);

        let right_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Length(5),
                Constraint::Length(5),
                Constraint::Length(5),
            ])
            .margin(1)
            .split(body_layout[1]);

        let primary_title = Paragraph::new("● REQUIRED INFORMATION").style(
            Style::default()
                .fg(Color::Rgb(250, 250, 110))
                .add_modifier(Modifier::BOLD)
                .bg(Color::Rgb(22, 22, 35)),
        );
        frame.render_widget(primary_title, left_layout[0]);

        let secondary_title = Paragraph::new("○ OPTIONAL INFORMATION").style(
            Style::default()
                .fg(Color::Rgb(250, 250, 110))
                .add_modifier(Modifier::BOLD)
                .bg(Color::Rgb(22, 22, 35)),
        );
        frame.render_widget(secondary_title, right_layout[0]);

        let required_style = Style::default().fg(Color::Rgb(230, 230, 250));

        let first_name_input = Paragraph::new(self.first_name.clone())
            .style(
                Style::default()
                    .fg(Color::Rgb(220, 220, 240))
                    .bg(Color::Rgb(26, 26, 36)),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(Span::styled(" First Name* ", required_style))
                    .border_style(if self.focus_index == 0 {
                        Style::default().fg(Color::Rgb(250, 250, 110))
                    } else {
                        Style::default().fg(Color::Rgb(140, 140, 200))
                    })
                    .style(Style::default().bg(Color::Rgb(26, 26, 36))),
            );
        frame.render_widget(first_name_input, left_layout[1]);

        let last_name_input = Paragraph::new(self.last_name.clone())
            .style(
                Style::default()
                    .fg(Color::Rgb(220, 220, 240))
                    .bg(Color::Rgb(26, 26, 36)),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(Span::styled(" Last Name* ", required_style))
                    .border_style(if self.focus_index == 1 {
                        Style::default().fg(Color::Rgb(250, 250, 110))
                    } else {
                        Style::default().fg(Color::Rgb(140, 140, 200))
                    })
                    .style(Style::default().bg(Color::Rgb(26, 26, 36))),
            );
        frame.render_widget(last_name_input, left_layout[2]);

        let dob_input = Paragraph::new(self.dob.clone())
            .style(
                Style::default()
                    .fg(Color::Rgb(220, 220, 240))
                    .bg(Color::Rgb(26, 26, 36)),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(Span::styled(" Date of Birth* ", required_style))
                    .title_alignment(Alignment::Left)
                    .border_style(if self.focus_index == 2 {
                        Style::default().fg(Color::Rgb(250, 250, 110))
                    } else {
                        Style::default().fg(Color::Rgb(140, 140, 200))
                    })
                    .style(Style::default().bg(Color::Rgb(26, 26, 36))),
            );
        frame.render_widget(dob_input, left_layout[3]);

        let gender_text = match self.gender {
            Gender::Male => "Male",
            Gender::Female => "Female",
            Gender::Other => "Other",
        };

        let gender_input = Paragraph::new(gender_text)
            .style(
                Style::default()
                    .fg(Color::Rgb(220, 220, 240))
                    .bg(Color::Rgb(26, 26, 36)),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(Span::styled(" Gender* ", required_style))
                    .border_style(if self.focus_index == 3 {
                        Style::default().fg(Color::Rgb(250, 250, 110))
                    } else {
                        Style::default().fg(Color::Rgb(140, 140, 200))
                    })
                    .style(Style::default().bg(Color::Rgb(26, 26, 36))),
            );
        frame.render_widget(gender_input, left_layout[4]);

        let address_input = Paragraph::new(self.address.clone())
            .style(
                Style::default()
                    .fg(Color::Rgb(220, 220, 240))
                    .bg(Color::Rgb(26, 26, 36)),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(Span::styled(" Address* ", required_style))
                    .border_style(if self.focus_index == 4 {
                        Style::default().fg(Color::Rgb(250, 250, 110))
                    } else {
                        Style::default().fg(Color::Rgb(140, 140, 200))
                    })
                    .style(Style::default().bg(Color::Rgb(26, 26, 36))),
            );
        frame.render_widget(address_input, left_layout[5]);

        let phone_input = Paragraph::new(self.phone.clone())
            .style(
                Style::default()
                    .fg(Color::Rgb(220, 220, 240))
                    .bg(Color::Rgb(26, 26, 36)),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(Span::styled(" Phone* ", required_style))
                    .border_style(if self.focus_index == 5 {
                        Style::default().fg(Color::Rgb(250, 250, 110))
                    } else {
                        Style::default().fg(Color::Rgb(140, 140, 200))
                    })
                    .style(Style::default().bg(Color::Rgb(26, 26, 36))),
            );
        frame.render_widget(phone_input, left_layout[6]);

        let optional_style = Style::default().fg(Color::Rgb(180, 180, 200));

        let email_input = Paragraph::new(self.email.clone().unwrap_or_default())
            .style(
                Style::default()
                    .fg(Color::Rgb(220, 220, 240))
                    .bg(Color::Rgb(26, 26, 36)),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(Span::styled(" Email (optional) ", optional_style))
                    .border_style(if self.focus_index == 6 {
                        Style::default().fg(Color::Rgb(250, 250, 110))
                    } else {
                        Style::default().fg(Color::Rgb(140, 140, 200))
                    })
                    .style(Style::default().bg(Color::Rgb(26, 26, 36))),
            );
        frame.render_widget(email_input, right_layout[1]);

        let history_input = Paragraph::new(self.medical_history.clone().unwrap_or_default())
            .style(
                Style::default()
                    .fg(Color::Rgb(220, 220, 240))
                    .bg(Color::Rgb(26, 26, 36)),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(Span::styled(" Medical History (optional) ", optional_style))
                    .border_style(if self.focus_index == 7 {
                        Style::default().fg(Color::Rgb(250, 250, 110))
                    } else {
                        Style::default().fg(Color::Rgb(140, 140, 200))
                    })
                    .style(Style::default().bg(Color::Rgb(26, 26, 36))),
            )
            .wrap(Wrap { trim: true });
        frame.render_widget(history_input, right_layout[2]);

        let allergies_input = Paragraph::new(self.allergies.clone().unwrap_or_default())
            .style(
                Style::default()
                    .fg(Color::Rgb(220, 220, 240))
                    .bg(Color::Rgb(26, 26, 36)),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(Span::styled(" Allergies (optional) ", optional_style))
                    .border_style(if self.focus_index == 8 {
                        Style::default().fg(Color::Rgb(250, 250, 110))
                    } else {
                        Style::default().fg(Color::Rgb(140, 140, 200))
                    })
                    .style(Style::default().bg(Color::Rgb(26, 26, 36))),
            )
            .wrap(Wrap { trim: true });
        frame.render_widget(allergies_input, right_layout[3]);

        let medications_input = Paragraph::new(self.medications.clone().unwrap_or_default())
            .style(
                Style::default()
                    .fg(Color::Rgb(220, 220, 240))
                    .bg(Color::Rgb(26, 26, 36)),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(Span::styled(" Medications (optional) ", optional_style))
                    .border_style(if self.focus_index == 9 {
                        Style::default().fg(Color::Rgb(250, 250, 110))
                    } else {
                        Style::default().fg(Color::Rgb(140, 140, 200))
                    })
                    .style(Style::default().bg(Color::Rgb(26, 26, 36))),
            )
            .wrap(Wrap { trim: true });
        frame.render_widget(medications_input, right_layout[4]);

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
        frame.render_widget(status_message, main_layout[3]);

        let footer_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),
                Constraint::Length(2),
                Constraint::Min(2),
            ])
            .split(main_layout[5]);

        let submit_text = if self.focus_index == INPUT_FIELDS {
            "► Submit ◄"
        } else {
            "  Submit  "
        };

        let submit_style = if self.focus_index == INPUT_FIELDS {
            Style::default()
                .fg(Color::Rgb(140, 219, 140))
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Rgb(180, 180, 200))
        };

        let submit_button = Paragraph::new(submit_text)
            .style(submit_style)
            .alignment(Alignment::Center);
        frame.render_widget(submit_button, footer_layout[0]);

        let back_text = if self.focus_index == INPUT_FIELDS + 1 {
            "► Back ◄"
        } else {
            "  Back  "
        };

        let back_style = if self.focus_index == INPUT_FIELDS + 1 {
            Style::default()
                .fg(Color::Rgb(129, 199, 245))
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Rgb(180, 180, 200))
        };

        let back_button = Paragraph::new(back_text)
            .style(back_style)
            .alignment(Alignment::Center);
        frame.render_widget(back_button, footer_layout[1]);

        let help_text = Paragraph::new("Tab: Switch Focus | Arrow Keys: Switch Fields | Enter: Submit | Esc: Back\nFor Gender: Type 'M' for Male, 'F' for Female, 'O' for Others")
.style(Style::default().fg(Color::Rgb(140, 140, 170)).bg(Color::Rgb(16, 16, 28)))
.alignment(Alignment::Center);
        frame.render_widget(help_text, footer_layout[2]);
    }
}
