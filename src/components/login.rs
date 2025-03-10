use crate::app::SelectedApp;
use crate::components::Component;
use crate::tui::Frame;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};
use std::time::{Duration, Instant};

#[derive(Debug, Default)]
pub struct Login {
    pub username: String,
    pub password: String,
    #[allow(dead_code)]
    pub focus_username: bool,
    pub error_message: Option<String>,
    pub success_message: Option<String>,
    pub selected_index: usize,
    pub show_exit_dialog: bool,
    pub exit_dialog_selected: usize,
    error_message_time: Option<std::time::Instant>,
    success_message_time: Option<std::time::Instant>,
}

impl Login {
    pub fn new() -> Self {
        Self {
            focus_username: true,
            selected_index: 0,
            show_exit_dialog: false,
            exit_dialog_selected: 0,
            error_message_time: None,
            ..Default::default()
        }
    }

    fn handle_exit_dialog_input(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Left | KeyCode::Right => {
                self.exit_dialog_selected = 1 - self.exit_dialog_selected;
            }
            KeyCode::Enter => {
                if self.exit_dialog_selected == 0 {
                    return Ok(true);
                } else {
                    self.show_exit_dialog = false;
                }
            }
            KeyCode::Esc => {
                self.show_exit_dialog = false;
            }
            _ => {}
        }
        Ok(false)
    }

    fn handle_login_input(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Char(c) => {
                if self.selected_index == 0 {
                    self.username.push(c);
                } else if self.selected_index == 1 {
                    self.password.push(c);
                }
                self.clear_error_message();
            }
            KeyCode::Backspace => {
                if self.selected_index == 0 {
                    self.username.pop();
                } else if self.selected_index == 1 {
                    self.password.pop();
                }
                self.clear_error_message();
            }
            KeyCode::Tab | KeyCode::Down => {
                self.selected_index = (self.selected_index + 1) % 4;
            }
            KeyCode::Up => {
                self.selected_index = (self.selected_index + 3) % 4;
            }
            KeyCode::Enter => match self.selected_index {
                0 | 1 => {
                    if self.username.is_empty() {
                        self.set_error_message("⚠️ Username cannot be empty.".to_string());
                        return Ok(false);
                    }

                    if self.password.is_empty() {
                        self.set_error_message("⚠️ Password cannot be empty.".to_string());
                        return Ok(false);
                    }

                    return Ok(true);
                }
                2 => {
                    return Ok(true);
                }
                3 => {
                    self.show_exit_dialog = true;
                    return Ok(false);
                }
                _ => {}
            },
            KeyCode::Esc => {
                self.show_exit_dialog = !self.show_exit_dialog;
            }
            _ => {}
        }
        Ok(false)
    }

    fn clear_error_message(&mut self) {
        self.error_message = None;
        self.error_message_time = None;
    }

    fn clear_success_message(&mut self) {
        self.success_message = None;
        self.success_message_time = None;
    }

    fn set_error_message(&mut self, message: String) {
        self.error_message = Some(message);
        self.error_message_time = Some(Instant::now());
    }

    pub fn set_success_message(&mut self, message: String) {
        self.success_message = Some(message);
        self.success_message_time = Some(Instant::now());
    }

    pub fn check_error_timeout(&mut self) {
        if let Some(time) = self.error_message_time {
            if time.elapsed() >= Duration::from_secs(5) {
                self.clear_error_message();
            }
        }
    }

    pub fn check_message_timeouts(&mut self) {
        if let Some(time) = self.error_message_time {
            if time.elapsed() >= Duration::from_secs(5) {
                self.clear_error_message();
            }
        }

        if let Some(time) = self.success_message_time {
            if time.elapsed() >= Duration::from_secs(5) {
                self.clear_success_message();
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

impl Component for Login {
    fn handle_input(&mut self, event: KeyEvent) -> Result<Option<crate::app::SelectedApp>> {
        self.check_message_timeouts();

        if self.show_exit_dialog {
            if self.handle_exit_dialog_input(event)? {
                return Ok(Some(SelectedApp::Quit));
            }
        } else {
            if self.handle_login_input(event)? {
                if self.selected_index == 2 {
                    return Ok(Some(SelectedApp::Hospital));
                } else {
                    return Ok(Some(SelectedApp::None));
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

        let vertical_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(7),
                    Constraint::Length(1),
                    Constraint::Length(2),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(2),
                    Constraint::Length(1),
                    Constraint::Length(2),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(3),
                    Constraint::Length(1),
                    Constraint::Min(0),
                ]
                .as_ref(),
            )
            .margin(1)
            .split(frame.area());

        let title = Paragraph::new(Text::from(vec![
            Line::from("██████╗░██╗░░░██╗░██████╗████████╗░█████╗░██████╗░██╗░█████╗░".to_string()),
            Line::from("██╔══██╗██║░░░██║██╔════╝╚══██╔══╝██╔══██╗██╔══██╗██║██╔══██╗".to_string()),
            Line::from("██████╔╝██║░░░██║╚█████╗░░░░██║░░░██║░░██║██████╔╝██║███████║".to_string()),
            Line::from("██╔══██╗██║░░░██║░╚═══██╗░░░██║░░░██║░░██║██╔══██╗██║██╔══██║".to_string()),
            Line::from("██║░░██║╚██████╔╝██████╔╝░░░██║░░░╚█████╔╝██║░░██║██║██║░░██║".to_string()),
            Line::from("╚═╝░░╚═╝░╚═════╝░╚═════╝░░░░╚═╝░░░░╚════╝░╚═╝░░╚═╝╚═╝╚═╝░░╚═╝".to_string()),
        ]))
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Rgb(129, 199, 245)));
        frame.render_widget(title, vertical_layout[0]);

        let title_block = Block::default().borders(Borders::NONE);
        let title = Paragraph::new(Text::from(vec![Line::from(Span::styled(
            "Seamless Hospital & Pharmacy Operations",
            Style::default()
                .fg(Color::Rgb(140, 219, 140))
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::ITALIC),
        ))]))
        .block(title_block)
        .alignment(Alignment::Center);
        frame.render_widget(title, vertical_layout[1]);

        let login_area = centered_rect(70, 48, frame.area());
        let login_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Rgb(75, 75, 120)))
            .style(Style::default().bg(Color::Rgb(22, 22, 35)));
        frame.render_widget(login_block.clone(), login_area);

        let subtitle = Paragraph::new(Span::styled(
            "Login to Rustoria",
            Style::default()
                .fg(Color::Rgb(230, 230, 250))
                .add_modifier(Modifier::BOLD),
        ))
        .alignment(Alignment::Center);
        frame.render_widget(subtitle, vertical_layout[3]);

        let username_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Username ")
            .title_style(
                Style::default()
                    .fg(Color::Rgb(230, 230, 250))
                    .add_modifier(Modifier::BOLD),
            )
            .border_style(if self.selected_index == 0 {
                Style::default().fg(Color::Rgb(250, 250, 110))
            } else {
                Style::default().fg(Color::Rgb(140, 140, 200))
            })
            .style(Style::default().bg(Color::Rgb(26, 26, 36)));

        let username_area = centered_rect(60, 100, vertical_layout[5]);
        let username_input = Paragraph::new(self.username.clone())
            .block(username_block)
            .style(Style::default().fg(Color::Rgb(220, 220, 240)))
            .alignment(Alignment::Left);
        frame.render_widget(username_input, username_area);

        let password_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Password ")
            .title_style(
                Style::default()
                    .fg(Color::Rgb(230, 230, 250))
                    .add_modifier(Modifier::BOLD),
            )
            .border_style(if self.selected_index == 1 {
                Style::default().fg(Color::Rgb(250, 250, 110))
            } else {
                Style::default().fg(Color::Rgb(140, 140, 200))
            })
            .style(Style::default().bg(Color::Rgb(26, 26, 36)));

        let password_area = centered_rect(60, 100, vertical_layout[6]);
        let password_input = Paragraph::new("•".repeat(self.password.len()))
            .block(password_block)
            .style(Style::default().fg(Color::Rgb(220, 220, 240)))
            .alignment(Alignment::Left);
        frame.render_widget(password_input, password_area);

        if let Some(error) = &self.error_message {
            let error_message = Paragraph::new(Span::styled(
                error,
                Style::default()
                    .fg(Color::Rgb(255, 100, 100))
                    .add_modifier(Modifier::BOLD),
            ))
            .alignment(Alignment::Center);
            frame.render_widget(error_message, vertical_layout[7]);
        } else if let Some(success) = &self.success_message {
            let success_message = Paragraph::new(Span::styled(
                success,
                Style::default()
                    .fg(Color::Rgb(140, 219, 140))
                    .add_modifier(Modifier::BOLD),
            ))
            .alignment(Alignment::Center);
            frame.render_widget(success_message, vertical_layout[7]);
        }

        let create_account_style = if self.selected_index == 2 {
            Style::default()
                .fg(Color::Rgb(250, 250, 110))
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Rgb(180, 180, 200))
        };

        let create_account_text = Paragraph::new(if self.selected_index == 2 {
            "► Create Account ◄"
        } else {
            "  Create Account  "
        })
        .style(create_account_style)
        .alignment(Alignment::Center);
        frame.render_widget(create_account_text, vertical_layout[9]);

        let exit_style = if self.selected_index == 3 {
            Style::default()
                .fg(Color::Rgb(255, 100, 100))
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Rgb(180, 180, 200))
        };

        let exit_text = Paragraph::new(if self.selected_index == 3 {
            "► Exit ◄"
        } else {
            "  Exit  "
        })
        .style(exit_style)
        .alignment(Alignment::Center);
        frame.render_widget(exit_text, vertical_layout[11]);

        let help_text = Paragraph::new(
            "TAB/Arrow Keys: Navigate | ENTER: Login/Select | ESC: Toggle Exit Dialog",
        )
        .style(Style::default().fg(Color::Rgb(140, 140, 170)))
        .alignment(Alignment::Center);
        frame.render_widget(help_text, vertical_layout[13]);

        if self.show_exit_dialog {
            let dialog_width = 40;
            let dialog_height = 8;

            let dialog_area = Rect::new(
                (frame.area().width.saturating_sub(dialog_width)) / 2,
                (frame.area().height.saturating_sub(dialog_height)) / 2,
                dialog_width,
                dialog_height,
            );

            frame.render_widget(Clear, dialog_area);

            let dialog_block = Block::default()
                .title(" Confirm Exit ")
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

            let message = Paragraph::new("Are you sure you want to exit?")
                .style(Style::default().fg(Color::Rgb(220, 220, 240)))
                .add_modifier(Modifier::BOLD)
                .alignment(Alignment::Center);

            frame.render_widget(message, content_layout[0]);

            let buttons_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(content_layout[1]);

            let yes_style = if self.exit_dialog_selected == 0 {
                Style::default()
                    .fg(Color::Rgb(140, 219, 140))
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Rgb(180, 180, 200))
            };

            let no_style = if self.exit_dialog_selected == 1 {
                Style::default()
                    .fg(Color::Rgb(255, 100, 100))
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Rgb(180, 180, 200))
            };

            let yes_text = if self.exit_dialog_selected == 0 {
                "► Yes ◄"
            } else {
                "  Yes  "
            };

            let no_text = if self.exit_dialog_selected == 1 {
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
}
