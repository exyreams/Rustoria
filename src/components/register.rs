use crate::app::SelectedApp;
use crate::components::Component;
use crate::tui::Frame;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, Paragraph},
};
use std::time::{Duration, Instant};

#[derive(Debug, Default)]
pub struct Register {
    pub username: String,
    pub password: String,
    pub confirm_password: String,
    focus_index: usize,
    pub error_message: Option<String>,
    error_message_time: Option<Instant>,
    pub registration_success: bool,
}

impl Register {
    pub fn new() -> Self {
        Self::default()
    }

    fn handle_register_input(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Char(c) => {
                match self.focus_index {
                    0 => self.username.push(c),
                    1 => self.password.push(c),
                    2 => self.confirm_password.push(c),
                    _ => {}
                }
                self.clear_error_message();
                self.registration_success = false;
            }
            KeyCode::Backspace => {
                match self.focus_index {
                    0 => self.username.pop(),
                    1 => self.password.pop(),
                    2 => self.confirm_password.pop(),
                    _ => None,
                };
                self.clear_error_message();
                self.registration_success = false;
            }
            KeyCode::Tab | KeyCode::Down => {
                self.focus_index = (self.focus_index + 1) % 4;
            }
            KeyCode::Up => {
                self.focus_index = (self.focus_index + 3) % 4;
            }
            KeyCode::Enter => {
                if self.focus_index == 3 {
                    return Ok(true);
                }

                return Ok(true);
            }
            KeyCode::Esc => {
                return Ok(true);
            }
            _ => {}
        }
        Ok(false)
    }

    fn clear_error_message(&mut self) {
        self.error_message = None;
        self.error_message_time = None;
    }

    fn set_error_message(&mut self, message: String) {
        self.error_message = Some(message);
        self.error_message_time = Some(Instant::now());
    }

    pub fn check_error_timeout(&mut self) {
        if let Some(time) = self.error_message_time {
            if time.elapsed() >= Duration::from_secs(5) {
                self.clear_error_message();
            }
        }
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

impl Component for Register {
    fn handle_input(&mut self, event: KeyEvent) -> Result<Option<crate::app::SelectedApp>> {
        self.check_error_timeout();

        if self.handle_register_input(event)? {
            if self.focus_index == 3 || event.code == KeyCode::Esc {
                return Ok(Some(SelectedApp::None));
            } else {
                if self.username.is_empty() {
                    self.set_error_message("Username cannot be empty.".to_string());
                    return Ok(None);
                }
                if self.password.is_empty() {
                    self.set_error_message("Password cannot be empty.".to_string());
                    return Ok(None);
                }
                if self.password != self.confirm_password {
                    self.set_error_message("Passwords do not match.".to_string());
                    return Ok(None);
                }

                match crate::db::create_user(&self.username, &self.password) {
                    Ok(_) => {
                        self.registration_success = true;

                        self.username.clear();
                        self.password.clear();
                        self.confirm_password.clear();
                        return Ok(Some(SelectedApp::None));
                    }
                    Err(err) => {
                        self.set_error_message(format!("{}", err));
                        return Ok(None);
                    }
                }
            }
        }
        Ok(None)
    }

    fn render(&self, frame: &mut Frame) {
        frame.render_widget(
            Block::default().style(Style::default().bg(Color::Rgb(16, 16, 28))),
            frame.area(),
        );

        let form_container = centered_rect(70, 70, frame.area());
        let container_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Rgb(75, 75, 120)))
            .style(Style::default().bg(Color::Rgb(22, 22, 35)));

        frame.render_widget(container_block.clone(), form_container);

        let vertical_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(2),
                    Constraint::Length(1),
                    Constraint::Length(3),
                    Constraint::Length(1),
                    Constraint::Length(3),
                    Constraint::Length(1),
                    Constraint::Length(3),
                    Constraint::Length(2),
                    Constraint::Length(2),
                    Constraint::Length(1),
                    Constraint::Min(0),
                ]
                .as_ref(),
            )
            .margin(2)
            .split(container_block.inner(form_container));

        let title = Paragraph::new("Create Account")
            .style(
                Style::default()
                    .fg(Color::Rgb(230, 230, 250))
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center);
        frame.render_widget(title, vertical_layout[0]);

        let username_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Username ")
            .title_style(
                Style::default()
                    .fg(Color::Rgb(230, 230, 250))
                    .add_modifier(Modifier::BOLD),
            )
            .border_style(if self.focus_index == 0 {
                Style::default().fg(Color::Rgb(250, 250, 110))
            } else {
                Style::default().fg(Color::Rgb(140, 140, 200))
            })
            .style(Style::default().bg(Color::Rgb(26, 26, 36)));

        let username_input = Paragraph::new(self.username.clone())
            .block(username_block)
            .style(Style::default().fg(Color::Rgb(220, 220, 240)))
            .alignment(Alignment::Left);
        frame.render_widget(username_input, vertical_layout[2]);

        let password_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Password ")
            .title_style(
                Style::default()
                    .fg(Color::Rgb(230, 230, 250))
                    .add_modifier(Modifier::BOLD),
            )
            .border_style(if self.focus_index == 1 {
                Style::default().fg(Color::Rgb(250, 250, 110))
            } else {
                Style::default().fg(Color::Rgb(140, 140, 200))
            })
            .style(Style::default().bg(Color::Rgb(26, 26, 36)));

        let password_input = Paragraph::new("•".repeat(self.password.len()))
            .block(password_block)
            .style(Style::default().fg(Color::Rgb(220, 220, 240)))
            .alignment(Alignment::Left);
        frame.render_widget(password_input, vertical_layout[4]);

        let confirm_password_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Confirm Password ")
            .title_style(
                Style::default()
                    .fg(Color::Rgb(230, 230, 250))
                    .add_modifier(Modifier::BOLD),
            )
            .border_style(if self.focus_index == 2 {
                Style::default().fg(Color::Rgb(250, 250, 110))
            } else {
                Style::default().fg(Color::Rgb(140, 140, 200))
            })
            .style(Style::default().bg(Color::Rgb(26, 26, 36)));

        let confirm_password_input = Paragraph::new("•".repeat(self.confirm_password.len()))
            .block(confirm_password_block)
            .style(Style::default().fg(Color::Rgb(220, 220, 240)))
            .alignment(Alignment::Left);
        frame.render_widget(confirm_password_input, vertical_layout[6]);

        if let Some(error) = &self.error_message {
            let error_paragraph = Paragraph::new(error.as_str())
                .style(
                    Style::default()
                        .fg(Color::Rgb(255, 100, 100))
                        .add_modifier(Modifier::BOLD),
                )
                .alignment(Alignment::Center);
            frame.render_widget(error_paragraph, vertical_layout[7]);
        } else if self.registration_success {
            let success_paragraph =
                Paragraph::new("✓ Registration successful! You can now log in.")
                    .style(
                        Style::default()
                            .fg(Color::Rgb(140, 219, 140))
                            .add_modifier(Modifier::BOLD),
                    )
                    .alignment(Alignment::Center);
            frame.render_widget(success_paragraph, vertical_layout[7]);
        }

        let back_style = if self.focus_index == 3 {
            Style::default()
                .fg(Color::Rgb(129, 199, 245))
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Rgb(180, 180, 200))
        };

        let back_to_login_text = Paragraph::new(if self.focus_index == 3 {
            "► Back to Login ◄"
        } else {
            "  Back to Login  "
        })
        .style(back_style)
        .alignment(Alignment::Center);
        frame.render_widget(back_to_login_text, vertical_layout[8]);

        let help_text =
            Paragraph::new("TAB/Arrow Keys: Navigate | ENTER: Select | ESC: Back to Login")
                .style(Style::default().fg(Color::Rgb(140, 140, 170)))
                .alignment(Alignment::Center);
        frame.render_widget(help_text, vertical_layout[9]);
    }
}
