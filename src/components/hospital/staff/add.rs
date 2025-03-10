use crate::app::SelectedApp;
use crate::components::Component;
use crate::db;
use crate::models::{StaffMember, StaffRole};
use crate::tui::Frame;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{prelude::*, widgets::*};
use std::time::{Duration, Instant};

pub struct AddStaff {
    name: String,
    role: StaffRole,
    phone: String,
    email: Option<String>,
    address: String,
    focus_index: usize,
    error_message: Option<String>,
    error_timer: Option<Instant>,
    success_message: Option<String>,
    success_timer: Option<Instant>,
}

const INPUT_FIELDS: usize = 5;
const SUBMIT_BUTTON: usize = 5;
const BACK_BUTTON: usize = 6;

impl AddStaff {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            role: StaffRole::Doctor,
            phone: String::new(),
            email: None,
            address: String::new(),
            focus_index: 0,
            error_message: None,
            error_timer: None,
            success_message: None,
            success_timer: None,
        }
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

    fn process_input(&mut self, key: KeyEvent) -> Result<Option<SelectedApp>> {
        self.check_timeouts();
        match key.code {
            KeyCode::Char(c) => {
                match self.focus_index {
                    0 => self.name.push(c),
                    1 => {
                        if c.to_ascii_lowercase() == 'd' {
                            self.role = StaffRole::Doctor;
                        } else if c.to_ascii_lowercase() == 'n' {
                            self.role = StaffRole::Nurse;
                        } else if c.to_ascii_lowercase() == 'a' {
                            self.role = StaffRole::Admin;
                        } else if c.to_ascii_lowercase() == 't' {
                            self.role = StaffRole::Technician;
                        }
                    }
                    2 => self.phone.push(c),
                    3 => {
                        if let Some(ref mut email) = self.email {
                            email.push(c);
                        } else {
                            self.email = Some(c.to_string());
                        }
                    }
                    4 => self.address.push(c),
                    _ => {}
                }
                self.clear_error();
            }
            KeyCode::Backspace => {
                match self.focus_index {
                    0 => {
                        self.name.pop();
                    }
                    1 => { /* Role handled separately */ }
                    2 => {
                        self.phone.pop();
                    }
                    3 => {
                        if let Some(email) = self.email.as_mut() {
                            email.pop();
                        }
                    }
                    4 => {
                        self.address.pop();
                    }
                    _ => {}
                }
                self.clear_error();
            }
            KeyCode::Tab => {
                if self.focus_index <= INPUT_FIELDS - 1 {
                    self.focus_index = SUBMIT_BUTTON;
                } else if self.focus_index == SUBMIT_BUTTON {
                    self.focus_index = BACK_BUTTON;
                } else {
                    self.focus_index = 0;
                }
            }
            KeyCode::Down => {
                self.focus_index = (self.focus_index + 1) % (BACK_BUTTON + 1);
            }
            KeyCode::Up => {
                self.focus_index = (self.focus_index + BACK_BUTTON) % (BACK_BUTTON + 1);
            }
            KeyCode::Left => {}
            KeyCode::Right => {}
            KeyCode::Enter => {
                if self.focus_index == BACK_BUTTON {
                    return Ok(Some(SelectedApp::None));
                }
                if self.focus_index == SUBMIT_BUTTON {
                    if self.name.is_empty() {
                        self.set_error("Name cannot be empty".to_string());
                        return Ok(None);
                    }
                    if self.phone.is_empty() {
                        self.set_error("Phone number cannot be empty".to_string());
                        return Ok(None);
                    }
                    if self.address.is_empty() {
                        self.set_error("Address cannot be empty".to_string());
                        return Ok(None);
                    }

                    let new_staff_member = StaffMember {
                        id: 0,
                        name: self.name.clone(),
                        role: self.role.clone(),
                        phone_number: self.phone.clone(),
                        email: self.email.clone(),
                        address: self.address.clone(),
                    };

                    match db::create_staff_member(&new_staff_member) {
                        Ok(_) => {
                            self.success_message =
                                Some("Staff member added successfully!".to_string());
                            self.success_timer = Some(Instant::now());

                            self.name.clear();
                            self.role = StaffRole::Doctor;
                            self.phone.clear();
                            self.email = None;
                            self.address.clear();
                            self.focus_index = 0;
                            self.clear_error();
                        }
                        Err(e) => {
                            self.set_error(format!("Database error: {}", e));
                        }
                    }
                }
            }
            KeyCode::Esc => {
                return Ok(Some(SelectedApp::None));
            }
            _ => {}
        }

        Ok(None)
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

impl Default for AddStaff {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for AddStaff {
    fn handle_input(&mut self, event: KeyEvent) -> Result<Option<SelectedApp>> {
        self.check_timeouts();

        if let Some(selected_app) = self.process_input(event)? {
            return Ok(Some(selected_app));
        }
        Ok(None)
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

        let title = Paragraph::new("👥 ADD STAFF")
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
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
            ])
            .margin(1)
            .split(body_inner);

        let name_input = Paragraph::new(self.name.clone())
            .style(
                Style::default()
                    .fg(Color::Rgb(220, 220, 240))
                    .bg(Color::Rgb(26, 26, 36)),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(Span::styled(
                        " Full Name ",
                        Style::default().fg(if self.focus_index == 0 {
                            Color::Rgb(250, 250, 110)
                        } else {
                            Color::Rgb(140, 140, 200)
                        }),
                    ))
                    .border_style(if self.focus_index == 0 {
                        Style::default().fg(Color::Rgb(250, 250, 110))
                    } else {
                        Style::default().fg(Color::Rgb(140, 140, 200))
                    })
                    .style(Style::default().bg(Color::Rgb(26, 26, 36))),
            );
        frame.render_widget(name_input, body_layout[0]);

        let role_text = match self.role {
            StaffRole::Doctor => "Doctor",
            StaffRole::Nurse => "Nurse",
            StaffRole::Admin => "Admin",
            StaffRole::Technician => "Technician",
        };

        let role_input = Paragraph::new(role_text)
            .style(
                Style::default()
                    .fg(Color::Rgb(220, 220, 240))
                    .bg(Color::Rgb(26, 26, 36)),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(Span::styled(
                        " Role (D/N/A/T) ",
                        Style::default().fg(if self.focus_index == 1 {
                            Color::Rgb(250, 250, 110)
                        } else {
                            Color::Rgb(140, 140, 200)
                        }),
                    ))
                    .border_style(if self.focus_index == 1 {
                        Style::default().fg(Color::Rgb(250, 250, 110))
                    } else {
                        Style::default().fg(Color::Rgb(140, 140, 200))
                    })
                    .style(Style::default().bg(Color::Rgb(26, 26, 36))),
            );
        frame.render_widget(role_input, body_layout[1]);

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
                    .title(Span::styled(
                        " Phone ",
                        Style::default().fg(if self.focus_index == 2 {
                            Color::Rgb(250, 250, 110)
                        } else {
                            Color::Rgb(140, 140, 200)
                        }),
                    ))
                    .border_style(if self.focus_index == 2 {
                        Style::default().fg(Color::Rgb(250, 250, 110))
                    } else {
                        Style::default().fg(Color::Rgb(140, 140, 200))
                    })
                    .style(Style::default().bg(Color::Rgb(26, 26, 36))),
            );
        frame.render_widget(phone_input, body_layout[2]);

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
                    .title(Span::styled(
                        " Email ",
                        Style::default().fg(if self.focus_index == 3 {
                            Color::Rgb(250, 250, 110)
                        } else {
                            Color::Rgb(140, 140, 200)
                        }),
                    ))
                    .border_style(if self.focus_index == 3 {
                        Style::default().fg(Color::Rgb(250, 250, 110))
                    } else {
                        Style::default().fg(Color::Rgb(140, 140, 200))
                    })
                    .style(Style::default().bg(Color::Rgb(26, 26, 36))),
            );
        frame.render_widget(email_input, body_layout[3]);

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
                    .title(Span::styled(
                        " Address ",
                        Style::default().fg(if self.focus_index == 4 {
                            Color::Rgb(250, 250, 110)
                        } else {
                            Color::Rgb(140, 140, 200)
                        }),
                    ))
                    .border_style(if self.focus_index == 4 {
                        Style::default().fg(Color::Rgb(250, 250, 110))
                    } else {
                        Style::default().fg(Color::Rgb(140, 140, 200))
                    })
                    .style(Style::default().bg(Color::Rgb(26, 26, 36))),
            );
        frame.render_widget(address_input, body_layout[4]);

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

        let submit_text = if self.focus_index == SUBMIT_BUTTON {
            "► Submit ◄"
        } else {
            "  Submit  "
        };

        let submit_style = if self.focus_index == SUBMIT_BUTTON {
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
        frame.render_widget(back_button, footer_layout[1]);

        let help_text = "Tab: Switch Focus | Arrow Keys: Switch Fields | Enter: Submit | Esc: Back\nFor Role: Type 'D' for Doctor, 'N' for Nurse, 'A' for Admin, 'T' for Technician";
        let help_paragraph = Paragraph::new(help_text)
            .style(
                Style::default()
                    .fg(Color::Rgb(140, 140, 170))
                    .bg(Color::Rgb(16, 16, 28)),
            )
            .alignment(Alignment::Center);
        frame.render_widget(help_paragraph, footer_layout[2]);
    }
}
