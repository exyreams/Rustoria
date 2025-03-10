use crate::components::hospital::staff::StaffAction;
use crate::components::Component;
use crate::db;
use crate::models::StaffMember;
use crate::tui::Frame;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{prelude::*, widgets::*};

const SEARCH_FIELD: usize = 0;
const STAFF_LIST: usize = 1;
const BACK_BUTTON: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StaffViewState {
    ViewingList,
    ViewingDetails,
}

pub struct ListStaff {
    staff: Vec<StaffMember>,
    filtered_staff: Vec<StaffMember>,
    search_input: String,
    is_searching: bool,
    state: TableState,
    error_message: Option<String>,
    view_state: StaffViewState,
    focus_index: usize,
}

impl ListStaff {
    pub fn new() -> Self {
        Self {
            staff: Vec::new(),
            filtered_staff: Vec::new(),
            search_input: String::new(),
            is_searching: false,
            state: TableState::default(),
            error_message: None,
            view_state: StaffViewState::ViewingList,
            focus_index: STAFF_LIST,
        }
    }

    pub fn fetch_staff(&mut self) -> Result<()> {
        match db::get_all_staff() {
            Ok(staff) => {
                self.staff = staff;
                self.filter_staff();

                if self.filtered_staff.is_empty() {
                    self.state.select(None);
                } else {
                    let selection = self
                        .state
                        .selected()
                        .unwrap_or(0)
                        .min(self.filtered_staff.len() - 1);
                    self.state.select(Some(selection));
                }
                self.error_message = None;
                Ok(())
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to fetch staff: {}", e));
                Ok(())
            }
        }
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

        if let Some(selected) = self.state.selected() {
            if selected >= self.filtered_staff.len() && !self.filtered_staff.is_empty() {
                self.state.select(Some(0));
            } else if self.filtered_staff.is_empty() {
                self.state.select(None);
            }
        }
    }

    fn select_next(&mut self) {
        if self.filtered_staff.is_empty() {
            return;
        }

        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.filtered_staff.len() - 1 {
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
        if self.filtered_staff.is_empty() {
            return;
        }

        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.filtered_staff.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn view_staff_details(&mut self) {
        if !self.filtered_staff.is_empty() && self.state.selected().is_some() {
            self.view_state = StaffViewState::ViewingDetails;
        }
    }

    fn return_to_list(&mut self) {
        self.view_state = StaffViewState::ViewingList;
    }

    fn focus_next(&mut self) {
        self.focus_index = (self.focus_index + 1) % 3;
        self.is_searching = self.focus_index == SEARCH_FIELD;
    }

    fn focus_previous(&mut self) {
        self.focus_index = (self.focus_index + 2) % 3;
        self.is_searching = self.focus_index == SEARCH_FIELD;
    }

    fn activate_search(&mut self) {
        self.is_searching = true;
        self.focus_index = SEARCH_FIELD;
    }

    pub fn process_input(&mut self, key: KeyEvent) -> Result<Option<StaffAction>> {
        if self.is_searching {
            match key.code {
                KeyCode::Char(c) => {
                    self.search_input.push(c);
                    self.filter_staff();
                }
                KeyCode::Backspace => {
                    self.search_input.pop();
                    self.filter_staff();
                }
                KeyCode::Enter | KeyCode::Down | KeyCode::Tab => {
                    if !self.filtered_staff.is_empty() {
                        self.is_searching = false;
                        self.focus_index = STAFF_LIST;
                        self.state.select(Some(0));
                    }
                }
                KeyCode::Esc => {
                    self.is_searching = false;
                    self.focus_index = STAFF_LIST;
                }
                _ => {}
            }
            return Ok(None);
        }

        if matches!(self.view_state, StaffViewState::ViewingDetails) {
            match key.code {
                KeyCode::Esc | KeyCode::Enter | KeyCode::Backspace => {
                    self.return_to_list();
                }
                KeyCode::Char('b') | KeyCode::Char('B') => {
                    self.return_to_list();
                }
                _ => {}
            }
            return Ok(None);
        }

        match key.code {
            KeyCode::Char(c) if c == '/' || c == 's' || c == 'S' => {
                self.activate_search();
                return Ok(None);
            }
            KeyCode::Tab => self.focus_next(),
            KeyCode::BackTab => self.focus_previous(),
            KeyCode::Down => {
                if self.focus_index == STAFF_LIST {
                    self.select_next();
                } else {
                    self.focus_next();
                }
            }
            KeyCode::Up => {
                if self.focus_index == STAFF_LIST {
                    self.select_previous();
                } else {
                    self.focus_previous();
                }
            }
            KeyCode::Right => {
                if self.focus_index != STAFF_LIST {
                    self.focus_next();
                }
            }
            KeyCode::Left => {
                if self.focus_index != STAFF_LIST {
                    self.focus_previous();
                }
            }
            KeyCode::Enter => {
                if self.focus_index == BACK_BUTTON {
                    return Ok(Some(StaffAction::BackToHome));
                } else if self.focus_index == STAFF_LIST {
                    self.view_staff_details();
                } else if self.focus_index == SEARCH_FIELD {
                    self.activate_search();
                }
            }
            KeyCode::Char('b') | KeyCode::Char('B') => {
                return Ok(Some(StaffAction::BackToHome));
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                self.fetch_staff()?;
            }
            KeyCode::Esc => {
                return Ok(Some(StaffAction::BackToHome));
            }
            _ => {}
        }
        Ok(None)
    }

    fn selected_staff(&self) -> Option<&StaffMember> {
        self.state
            .selected()
            .and_then(|i| self.filtered_staff.get(i))
    }
}

impl Component for ListStaff {
    fn handle_input(&mut self, event: KeyEvent) -> Result<Option<crate::app::SelectedApp>> {
        match self.process_input(event)? {
            Some(StaffAction::BackToHome) => Ok(Some(crate::app::SelectedApp::None)),
            Some(StaffAction::BackToList) => Ok(None),
            None => Ok(None),
        }
    }

    fn render(&self, frame: &mut Frame) {
        let area = frame.area();
        frame.render_widget(
            Block::default().style(Style::default().bg(Color::Rgb(16, 16, 28))),
            area,
        );

        match self.view_state {
            StaffViewState::ViewingList => self.render_list_view(frame),
            StaffViewState::ViewingDetails => self.render_details_view(frame),
        }
    }
}

impl ListStaff {
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
            ])
            .margin(1)
            .split(area);

        let header_block = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(Color::Rgb(75, 75, 120)))
            .style(Style::default().bg(Color::Rgb(16, 16, 28)));
        frame.render_widget(header_block, layout[0]);

        let title = Paragraph::new("👥 STAFF LIST")
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

        let header_cells = ["ID", "Name", "Role", "Phone", "Address"]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().fg(Color::Rgb(230, 230, 250))));
        let header = Row::new(header_cells)
            .style(Style::default().bg(Color::Rgb(80, 60, 130)))
            .height(1);

        let rows = self.filtered_staff.iter().map(|staff_member| {
            let cells = vec![
                Cell::from(staff_member.id.to_string()),
                Cell::from(staff_member.name.clone()),
                Cell::from(match staff_member.role {
                    crate::models::StaffRole::Doctor => "Doctor",
                    crate::models::StaffRole::Nurse => "Nurse",
                    crate::models::StaffRole::Admin => "Admin",
                    crate::models::StaffRole::Technician => "Technician",
                }),
                Cell::from(staff_member.phone_number.clone()),
                Cell::from(staff_member.address.clone()),
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
                " Staff ({} of {} matches) ",
                self.filtered_staff.len(),
                self.staff.len()
            )
        } else {
            format!(" Staff ({}) ", self.staff.len())
        };

        let table = Table::new(
            rows,
            [
                Constraint::Percentage(5),
                Constraint::Percentage(20),
                Constraint::Percentage(15),
                Constraint::Percentage(20),
                Constraint::Percentage(40),
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
        .highlight_symbol(if self.focus_index == STAFF_LIST {
            "► "
        } else {
            "  "
        });

        if self.filtered_staff.is_empty() {
            let message = if self.search_input.is_empty() {
                "No staff found in database"
            } else {
                "No staff match your search criteria"
            };

            let no_staff = Paragraph::new(message)
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
            frame.render_widget(no_staff, layout[2]);
        } else {
            frame.render_stateful_widget(table, layout[2], &mut self.state.clone());
        }

        let help_text = if self.is_searching {
            "Type to search | ↓/Enter: To results | Esc: Cancel search"
        } else {
            "/ or s: Search | ↑↓: Navigate | Enter: View Details | R: Refresh | Tab: Focus"
        };

        let help_paragraph = Paragraph::new(help_text)
            .style(Style::default().fg(Color::Rgb(140, 140, 170)))
            .alignment(Alignment::Center);
        frame.render_widget(help_paragraph, layout[3]);

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

    fn render_details_view(&self, frame: &mut Frame) {
        let area = frame.area();

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(16),
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

        let title = Paragraph::new("👤 STAFF DETAILS")
            .style(
                Style::default()
                    .fg(Color::Rgb(230, 230, 250))
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Rgb(16, 16, 28)),
            )
            .alignment(Alignment::Center);
        frame.render_widget(title, layout[0]);

        if let Some(staff_member) = self.selected_staff() {
            let content_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Min(4),
                ])
                .margin(1)
                .split(layout[1]);

            let title_style = Style::default().fg(Color::Cyan);

            let id_name_block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(Span::styled(" Basic Information ", title_style))
                .border_style(Style::default().fg(Color::White))
                .style(Style::default().bg(Color::Rgb(22, 22, 35)));

            let id_name_text = format!("  ID: {}, Name: {}", staff_member.id, staff_member.name);

            let id_name_widget = Paragraph::new(id_name_text)
                .style(Style::default().fg(Color::Rgb(220, 220, 240)))
                .block(id_name_block);

            frame.render_widget(id_name_widget, content_layout[0]);

            let role_block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(Span::styled(" Role ", title_style))
                .border_style(Style::default().fg(Color::White))
                .style(Style::default().bg(Color::Rgb(22, 22, 35)));

            let role_text = format!(
                "  {}",
                match staff_member.role {
                    crate::models::StaffRole::Doctor => "Doctor",
                    crate::models::StaffRole::Nurse => "Nurse",
                    crate::models::StaffRole::Admin => "Administrator",
                    crate::models::StaffRole::Technician => "Technician",
                }
            );

            let role_widget = Paragraph::new(role_text)
                .style(Style::default().fg(Color::Rgb(220, 220, 240)))
                .block(role_block);

            frame.render_widget(role_widget, content_layout[1]);

            let phone_block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(Span::styled(" Phone Number ", title_style))
                .border_style(Style::default().fg(Color::White))
                .style(Style::default().bg(Color::Rgb(22, 22, 35)));

            let phone_text = format!("  {}", staff_member.phone_number);

            let phone_widget = Paragraph::new(phone_text)
                .style(Style::default().fg(Color::Rgb(220, 220, 240)))
                .block(phone_block);

            frame.render_widget(phone_widget, content_layout[2]);

            let address_block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(Span::styled(" Address ", title_style))
                .border_style(Style::default().fg(Color::White))
                .style(Style::default().bg(Color::Rgb(22, 22, 35)));

            let address_text = format!("    {}", staff_member.address);

            let address_widget = Paragraph::new(address_text)
                .style(Style::default().fg(Color::Rgb(220, 220, 240)))
                .block(address_block)
                .wrap(Wrap { trim: true });

            frame.render_widget(address_widget, content_layout[3]);
        }

        let back_button = Paragraph::new("► Back ◄")
            .style(
                Style::default()
                    .fg(Color::Rgb(129, 199, 245))
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center);
        frame.render_widget(back_button, layout[2]);

        let help_text = "Enter/Esc/Backspace: Return to list";
        let help_paragraph = Paragraph::new(help_text)
            .style(Style::default().fg(Color::Rgb(140, 140, 170)))
            .alignment(Alignment::Center);
        frame.render_widget(help_paragraph, layout[3]);
    }
}

impl Default for ListStaff {
    fn default() -> Self {
        Self::new()
    }
}
