use crate::app::SelectedApp;
use crate::components::Component;
use crate::db;
use crate::models::StaffMember;
use crate::tui::Frame;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, Cell, Clear, Paragraph, Row, Table, TableState},
};
use std::time::{Duration, Instant};

pub struct DeleteStaff {
    staff: Vec<StaffMember>,
    filtered_staff: Vec<StaffMember>,
    selected_staff_ids: Vec<i64>,
    search_input: String,
    is_searching: bool,
    table_state: TableState,
    show_confirmation: bool,
    confirmation_selected: usize,
    error_message: Option<String>,
    error_timer: Option<Instant>,
    success_message: Option<String>,
    success_timer: Option<Instant>,
}

impl DeleteStaff {
    pub fn new() -> Self {
        Self {
            staff: Vec::new(),
            filtered_staff: Vec::new(),
            selected_staff_ids: Vec::new(),
            search_input: String::new(),
            is_searching: false,
            table_state: TableState::default(),
            show_confirmation: false,
            confirmation_selected: 1,
            error_message: None,
            error_timer: None,
            success_message: None,
            success_timer: None,
        }
    }

    pub fn fetch_staff(&mut self) -> Result<()> {
        self.staff = db::get_all_staff()?;
        self.filter_staff();

        if !self.filtered_staff.is_empty() {
            self.table_state.select(Some(0));
        } else {
            self.table_state.select(None);
        }

        Ok(())
    }

    fn filter_staff(&mut self) {
        if self.search_input.is_empty() {
            self.filtered_staff = self.staff.clone();
        } else {
            let search_term = self.search_input.to_lowercase();
            self.filtered_staff = self
                .staff
                .iter()
                .filter(|s| {
                    s.name.to_lowercase().contains(&search_term)
                        || s.id.to_string().contains(&search_term)
                        || s.phone_number.to_lowercase().contains(&search_term)
                        || s.address.to_lowercase().contains(&search_term)
                })
                .cloned()
                .collect();
        }

        self.selected_staff_ids.clear();
        if let Some(selected) = self.table_state.selected() {
            if selected >= self.filtered_staff.len() && !self.filtered_staff.is_empty() {
                self.table_state.select(Some(0));
            } else if self.filtered_staff.is_empty() {
                self.table_state.select(None)
            }
        }
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
                        let mut deleted_count = 0;
                        let mut error_occurred = false;

                        for staff_id in &self.selected_staff_ids {
                            match db::delete_staff_member(*staff_id) {
                                Ok(_) => deleted_count += 1,
                                Err(_) => {
                                    error_occurred = true;
                                    break;
                                }
                            }
                        }
                        self.selected_staff_ids.clear();

                        if error_occurred {
                            self.set_error(format!(
                                "Error during deletion. {} staff deleted successfully.",
                                deleted_count
                            ));
                        } else if deleted_count > 0 {
                            self.success_message = Some(format!(
                                "{} staff member{} deleted successfully!",
                                deleted_count,
                                if deleted_count == 1 { "" } else { "s" }
                            ));
                            self.success_timer = Some(Instant::now());
                        } else {
                            self.set_error("No staff were selected for deletion.".to_string());
                        }

                        if let Ok(staff) = db::get_all_staff() {
                            self.staff = staff;
                            self.filter_staff();

                            if self.filtered_staff.is_empty() {
                                self.table_state.select(None);
                            } else if let Some(selected) = self.table_state.selected() {
                                if selected >= self.filtered_staff.len() {
                                    self.table_state
                                        .select(Some(self.filtered_staff.len().saturating_sub(1)));
                                }
                            }
                        }

                        self.show_confirmation = false;
                    } else {
                        self.show_confirmation = false;
                    }
                }
                KeyCode::Esc => {
                    self.show_confirmation = false;
                }
                _ => {}
            }
        } else if self.is_searching {
            match key.code {
                KeyCode::Char(c) => {
                    self.search_input.push(c);
                    self.filter_staff();
                }
                KeyCode::Backspace => {
                    self.search_input.pop();
                    self.filter_staff();
                }
                KeyCode::Enter | KeyCode::Down => {
                    if !self.filtered_staff.is_empty() {
                        self.is_searching = false;
                        self.table_state.select(Some(0));
                    }
                }
                KeyCode::Esc => {
                    self.is_searching = false;
                }
                _ => {}
            }
        } else {
            match key.code {
                KeyCode::Char('/') | KeyCode::Char('s') | KeyCode::Char('S') => {
                    self.is_searching = true;
                    return Ok(None);
                }
                KeyCode::Down => {
                    if !self.filtered_staff.is_empty() {
                        let next = match self.table_state.selected() {
                            Some(i) => {
                                if i >= self.filtered_staff.len() - 1 {
                                    0
                                } else {
                                    i + 1
                                }
                            }
                            None => 0,
                        };
                        self.table_state.select(Some(next));
                    }
                }
                KeyCode::Up => {
                    if !self.filtered_staff.is_empty() {
                        let next = match self.table_state.selected() {
                            Some(i) => {
                                if i == 0 {
                                    self.filtered_staff.len() - 1
                                } else {
                                    i - 1
                                }
                            }
                            None => 0,
                        };
                        self.table_state.select(Some(next));
                    }
                }
                KeyCode::Char(' ') => {
                    if let Some(selected) = self.table_state.selected() {
                        if selected < self.filtered_staff.len() {
                            let staff_id = self.filtered_staff[selected].id;
                            if let Some(index) = self
                                .selected_staff_ids
                                .iter()
                                .position(|&id| id == staff_id)
                            {
                                self.selected_staff_ids.remove(index);
                            } else {
                                self.selected_staff_ids.push(staff_id);
                            }
                        }
                    }
                }
                KeyCode::Char('b') => {
                    if !self.selected_staff_ids.is_empty() {
                        self.show_confirmation = true;
                        self.confirmation_selected = 1;
                    } else {
                        self.set_error("No staff selected for deletion.".to_string());
                    }
                }
                KeyCode::Enter => {
                    if let Some(selected) = self.table_state.selected() {
                        if selected < self.filtered_staff.len() {
                            let staff_id = self.filtered_staff[selected].id;
                            self.selected_staff_ids.push(staff_id);
                            self.show_confirmation = true;
                            self.confirmation_selected = 1;
                        }
                    }
                }
                KeyCode::Char('a') => {
                    if self.selected_staff_ids.len() == self.filtered_staff.len() {
                        self.selected_staff_ids.clear();
                    } else {
                        self.selected_staff_ids =
                            self.filtered_staff.iter().map(|s| s.id).collect();
                    }
                }
                KeyCode::Char('r') | KeyCode::Char('R') => {
                    if let Ok(staff) = db::get_all_staff() {
                        self.staff = staff;
                        self.filter_staff();
                    }
                }
                KeyCode::Esc => {
                    return Ok(Some(SelectedApp::None));
                }
                _ => {}
            }
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

impl Default for DeleteStaff {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for DeleteStaff {
    fn handle_input(&mut self, event: KeyEvent) -> Result<Option<SelectedApp>> {
        self.handle_input(event)
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
                Constraint::Min(5),
                Constraint::Length(2),
                Constraint::Length(3),
            ])
            .margin(1)
            .split(area);

        let header_block = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(Color::Rgb(75, 75, 120)))
            .style(Style::default().bg(Color::Rgb(16, 16, 28)));
        frame.render_widget(header_block, layout[0]);

        let title = Paragraph::new("🗑️ STAFF DELETION MANAGER")
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
                " Search Staff ",
                Style::default()
                    .fg(Color::Rgb(230, 230, 250))
                    .add_modifier(Modifier::BOLD),
            ))
            .border_style(if self.is_searching {
                Style::default().fg(Color::Rgb(250, 250, 110)) // Yellow when active
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

        let table_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(if !self.search_input.is_empty() {
                format!(
                    " Staff ({} of {} matches) ",
                    self.filtered_staff.len(),
                    self.staff.len()
                )
            } else {
                format!(" Staff ({}) ", self.staff.len())
            })
            .title_style(
                Style::default()
                    .fg(Color::Rgb(230, 230, 250))
                    .add_modifier(Modifier::BOLD),
            )
            .border_style(Style::default().fg(Color::Rgb(140, 140, 200)))
            .style(Style::default().bg(Color::Rgb(26, 26, 36)));

        let selected_style = Style::default()
            .bg(Color::Rgb(45, 45, 60))
            .fg(Color::Rgb(250, 250, 110))
            .add_modifier(Modifier::BOLD);
        let normal_style = Style::default()
            .bg(Color::Rgb(26, 26, 36))
            .fg(Color::Rgb(220, 220, 240));

        let mut rows = Vec::new();
        for staff_member in &self.filtered_staff {
            let checkbox = if self.selected_staff_ids.contains(&staff_member.id) {
                "[✓]"
            } else {
                "[ ]"
            };

            rows.push(Row::new(vec![
                Cell::from(checkbox).style(normal_style),
                Cell::from(staff_member.id.to_string()).style(normal_style),
                Cell::from(staff_member.name.clone()).style(normal_style),
                Cell::from(match staff_member.role {
                    crate::models::StaffRole::Doctor => "Doctor",
                    crate::models::StaffRole::Nurse => "Nurse",
                    crate::models::StaffRole::Admin => "Admin",
                    crate::models::StaffRole::Technician => "Technician",
                })
                .style(normal_style),
                Cell::from(staff_member.phone_number.clone()).style(normal_style),
                Cell::from(staff_member.address.clone()).style(normal_style),
            ]));
        }

        if self.filtered_staff.is_empty() {
            let message = if self.search_input.is_empty() {
                "No staff found in database"
            } else {
                "No staff match your search criteria"
            };

            rows.push(Row::new(vec![
                Cell::from(""),
                Cell::from(""),
                Cell::from(message).style(Style::default().fg(Color::Rgb(180, 180, 200))),
                Cell::from(""),
                Cell::from(""),
                Cell::from(""),
            ]));
        }

        let table = Table::new(
            rows,
            [
                Constraint::Length(5),
                Constraint::Length(8),
                Constraint::Length(20),
                Constraint::Length(12),
                Constraint::Length(15),
                Constraint::Min(20),
            ],
        )
        .header(
            Row::new(vec![
                Cell::from(""),
                Cell::from("ID").style(Style::default().add_modifier(Modifier::BOLD)),
                Cell::from("Name").style(Style::default().add_modifier(Modifier::BOLD)),
                Cell::from("Role").style(Style::default().add_modifier(Modifier::BOLD)),
                Cell::from("Phone").style(Style::default().add_modifier(Modifier::BOLD)),
                Cell::from("Address").style(Style::default().add_modifier(Modifier::BOLD)),
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

        let mut table_state = self.table_state.clone();
        frame.render_stateful_widget(table, layout[2], &mut table_state);

        if let Some(success) = &self.success_message {
            let success_paragraph = Paragraph::new(success.as_str())
                .style(
                    Style::default()
                        .fg(Color::Rgb(140, 219, 140))
                        .add_modifier(Modifier::BOLD),
                )
                .alignment(Alignment::Center);
            frame.render_widget(success_paragraph, layout[3]);
        } else if let Some(error) = &self.error_message {
            let error_paragraph = Paragraph::new(error.as_str())
                .style(
                    Style::default()
                        .fg(Color::Rgb(240, 100, 100))
                        .add_modifier(Modifier::BOLD),
                )
                .alignment(Alignment::Center);
            frame.render_widget(error_paragraph, layout[3]);
        }

        if self.is_searching {
            let help_text =
                Paragraph::new("Type to search | ↓/Enter: To results | Esc: Cancel search")
                    .style(Style::default().fg(Color::Rgb(180, 180, 200)))
                    .alignment(Alignment::Center);
            frame.render_widget(help_text, layout[4]);
        } else {
            let help_block = Block::default()
                .border_style(Style::default().fg(Color::Rgb(75, 75, 120)))
                .style(Style::default().bg(Color::Rgb(16, 16, 28)));

            let help_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(layout[4]);

            let help_text1 = Paragraph::new("/ or s: Search | ↑/↓: Navigate | Space: Toggle | A: Select/deselect all | R: Refresh")
                .style(Style::default().fg(Color::Rgb(180, 180, 200)))
                .alignment(Alignment::Center);

            let help_text2 = Paragraph::new("Enter: Delete selected | B: Bulk delete | Esc: Back")
                .style(Style::default().fg(Color::Rgb(180, 180, 200)))
                .alignment(Alignment::Center);

            frame.render_widget(help_block, layout[4]);
            frame.render_widget(help_text1, help_layout[0]);
            frame.render_widget(help_text2, help_layout[1]);
        }

        if self.show_confirmation {
            let dialog_width = 50;
            let dialog_height = 8;
            let dialog_area = Rect::new(
                (area.width.saturating_sub(dialog_width)) / 2,
                (area.height.saturating_sub(dialog_height)) / 2,
                dialog_width,
                dialog_height,
            );

            frame.render_widget(Clear, dialog_area);

            let selected_count = self.selected_staff_ids.len();
            let title = format!(
                " Delete {} Staff Member{} ",
                selected_count,
                if selected_count == 1 { "" } else { "s" }
            );

            let dialog_block = Block::default()
                .title(title)
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

            let message =
                Paragraph::new("Are you sure you want to delete the selected staff member(s)?")
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
}
