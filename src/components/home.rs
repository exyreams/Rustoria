use crate::app::SelectedApp;
use crate::components::Component;
use crate::db;
use crate::tui::Frame;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Padding, Paragraph},
};

pub struct Home {
    username: Option<String>,
    selection_mode: usize,
    show_logout_dialog: bool,
    logout_dialog_selected: usize,
    active_panel: usize,
    selected_feature_index: usize,
    submenu_states: Vec<ListState>,
    features: Vec<&'static str>,
    submenu_options: Vec<Vec<&'static str>>,
}

impl Home {
    pub fn new() -> Self {
        let features = vec![
            "Billing & Finance",
            "Medical Records",
            "Patient Management",
            "Reports & Analytics",
            "Staff Scheduling",
        ];

        let submenu_options = vec![
            vec!["Generate Invoice", "View Invoices", "Update Invoice"],
            vec![
                "Store Record",
                "Retrieve Records",
                "Update Record",
                "Delete Record",
            ],
            vec![
                "Add Patient",
                "List Patients",
                "Update Patient",
                "Delete Patient",
            ],
            vec!["Generate Report", "Export Reports"],
            vec![
                "Add Staff",
                "Assign Shift",
                "Delete Staff",
                "List Staff",
                "Update Staff",
            ],
        ];

        let mut submenu_states = Vec::new();
        for _ in 0..features.len() {
            let mut state = ListState::default();
            state.select(Some(0));
            submenu_states.push(state);
        }

        Self {
            username: None,
            selection_mode: 0,
            show_logout_dialog: false,
            logout_dialog_selected: 0,
            active_panel: 0,
            selected_feature_index: 0,
            submenu_states,
            features,
            submenu_options,
        }
    }

    pub fn load_username(&mut self, user_id: i64) -> Result<()> {
        self.username = Some(db::get_username(user_id)?);
        Ok(())
    }

    pub fn handle_input(&mut self, key: KeyEvent) -> Result<Option<SelectedApp>> {
        if self.show_logout_dialog {
            return self.handle_logout_dialog_input(key);
        }

        match key.code {
            KeyCode::Tab => {
                self.selection_mode = (self.selection_mode + 1) % 2;
            }
            KeyCode::Left => {
                if self.selection_mode == 0 && self.active_panel == 1 {
                    self.active_panel = 0;
                }
            }
            KeyCode::Right => {
                if self.selection_mode == 0 && self.active_panel == 0 {
                    self.active_panel = 1;
                }
            }
            KeyCode::Up => {
                if self.selection_mode == 0 {
                    if self.active_panel == 0 {
                        if self.selected_feature_index > 0 {
                            self.selected_feature_index -= 1;
                        } else {
                            self.selected_feature_index = self.features.len() - 1;
                        }
                    } else {
                        let submenu_state = &mut self.submenu_states[self.selected_feature_index];
                        if let Some(i) = submenu_state.selected() {
                            let max_index =
                                self.submenu_options[self.selected_feature_index].len() - 1;
                            let new_index = if i > 0 { i - 1 } else { max_index };
                            submenu_state.select(Some(new_index));
                        }
                    }
                }
            }
            KeyCode::Down => {
                if self.selection_mode == 0 {
                    if self.active_panel == 0 {
                        self.selected_feature_index =
                            (self.selected_feature_index + 1) % self.features.len();
                    } else {
                        let submenu_state = &mut self.submenu_states[self.selected_feature_index];
                        if let Some(i) = submenu_state.selected() {
                            let max_index =
                                self.submenu_options[self.selected_feature_index].len() - 1;
                            let new_index = (i + 1) % (max_index + 1);
                            submenu_state.select(Some(new_index));
                        }
                    }
                }
            }
            KeyCode::Enter => {
                if self.selection_mode == 0 {
                    if self.active_panel == 1 {
                        let feature_idx = self.selected_feature_index;
                        let submenu_idx = self.submenu_states[feature_idx].selected().unwrap_or(0);

                        return Ok(Some(match feature_idx {
                            0 => match submenu_idx {
                                0 => SelectedApp::BillingInvoice,
                                1 => SelectedApp::BillingView,
                                2 => SelectedApp::BillingUpdate,
                                _ => SelectedApp::Hospital,
                            },

                            1 => match submenu_idx {
                                0 => SelectedApp::RecordStore,
                                1 => SelectedApp::RecordRetrieve,
                                2 => SelectedApp::RecordUpdate,
                                3 => SelectedApp::RecordDelete,
                                _ => SelectedApp::Hospital,
                            },

                            2 => match submenu_idx {
                                0 => SelectedApp::PatientAdd,
                                1 => SelectedApp::PatientList,
                                2 => SelectedApp::PatientUpdate,
                                3 => SelectedApp::PatientDelete,
                                _ => SelectedApp::Hospital,
                            },
                            // Analytics
                            3 => match submenu_idx {
                                // 0 => SelectedApp::ReportsGenerate,   // Generate Report
                                // 1 => SelectedApp::ReportsExport,     // Export Reports
                                _ => SelectedApp::Hospital,
                            },

                            4 => match submenu_idx {
                                0 => SelectedApp::StaffAdd,
                                1 => SelectedApp::StaffAssign,
                                2 => SelectedApp::StaffDelete,
                                3 => SelectedApp::StaffList,
                                4 => SelectedApp::StaffUpdate,
                                _ => SelectedApp::Hospital,
                            },
                            _ => SelectedApp::Hospital,
                        }));
                    } else {
                        self.active_panel = 1;
                    }
                } else {
                    self.show_logout_dialog = true;
                    self.logout_dialog_selected = 1;
                }
            }
            KeyCode::Esc => {
                if self.active_panel == 1 {
                    self.active_panel = 0;
                } else {
                    self.show_logout_dialog = true;
                    self.logout_dialog_selected = 1;
                }
            }
            _ => {}
        }

        Ok(None)
    }

    fn handle_logout_dialog_input(&mut self, key: KeyEvent) -> Result<Option<SelectedApp>> {
        match key.code {
            KeyCode::Left | KeyCode::Right => {
                self.logout_dialog_selected = 1 - self.logout_dialog_selected;
            }
            KeyCode::Enter => {
                self.show_logout_dialog = false;
                if self.logout_dialog_selected == 0 {
                    return Ok(Some(SelectedApp::None));
                }
            }
            KeyCode::Esc => {
                self.show_logout_dialog = false;
            }
            _ => {}
        }
        Ok(None)
    }
}

impl Component for Home {
    fn handle_input(&mut self, event: KeyEvent) -> Result<Option<SelectedApp>> {
        self.handle_input(event)
    }

    fn render(&self, frame: &mut Frame) {
        frame.render_widget(
            Block::default().style(Style::default().bg(Color::Rgb(16, 16, 28))),
            frame.area(),
        );

        let area = frame.area();

        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5),
                Constraint::Length(1),
                Constraint::Min(10),
                Constraint::Length(3),
                Constraint::Length(3),
            ])
            .split(area);

        let username = self.username.as_deref().unwrap_or("User");
        let welcome_text = Line::from(vec![
            Span::styled(
                "Welcome to Rustoria, ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                username,
                Style::default()
                    .fg(Color::Rgb(129, 199, 245))
                    .add_modifier(Modifier::BOLD),
            ),
        ]);

        let welcome_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Rgb(75, 75, 120)))
            .style(Style::default().bg(Color::Rgb(24, 24, 40)));

        let welcome_inner = welcome_block.inner(main_layout[0]);
        frame.render_widget(welcome_block, main_layout[0]);

        let welcome_paragraph = Paragraph::new(welcome_text)
            .alignment(Alignment::Center)
            .block(Block::default().padding(Padding::new(0, 0, 1, 0)));

        frame.render_widget(welcome_paragraph, welcome_inner);

        let instruction = Paragraph::new("Please select a task:")
            .style(Style::default().fg(Color::Rgb(180, 190, 254)))
            .alignment(Alignment::Center);

        frame.render_widget(instruction, main_layout[1]);

        let content_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .spacing(2)
            .margin(1)
            .split(main_layout[2]);

        let left_panel_style = if self.active_panel == 0 && self.selection_mode == 0 {
            Style::default().fg(Color::Rgb(250, 250, 110))
        } else {
            Style::default().fg(Color::Rgb(140, 140, 200))
        };

        let left_panel_block = Block::default()
            .title(" 🏥 Hospital Management ")
            .title_style(
                Style::default()
                    .fg(Color::Rgb(230, 230, 250))
                    .add_modifier(Modifier::BOLD),
            )
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(left_panel_style)
            .style(Style::default().bg(Color::Rgb(22, 22, 35)));

        frame.render_widget(left_panel_block.clone(), content_layout[0]);
        let left_inner = left_panel_block.inner(content_layout[0]);

        let left_padded = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(2), Constraint::Min(1)])
            .split(left_inner);

        let feature_items: Vec<ListItem> = self
            .features
            .iter()
            .enumerate()
            .map(|(idx, feature)| {
                let style = if idx == self.selected_feature_index {
                    if self.active_panel == 0 && self.selection_mode == 0 {
                        Style::default()
                            .fg(Color::Rgb(250, 250, 110))
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                            .fg(Color::Rgb(140, 219, 140))
                            .add_modifier(Modifier::BOLD)
                    }
                } else {
                    Style::default().fg(Color::Rgb(200, 200, 220))
                };

                let prefix = if idx == self.selected_feature_index {
                    " ► "
                } else {
                    "   "
                };

                let icon = match idx {
                    0 => "💰",
                    1 => "📦",
                    2 => "📋",
                    3 => "👤",
                    4 => "📊",
                    5 => "👥",
                    _ => "•",
                };

                ListItem::new(format!("{}{} {}", prefix, icon, feature)).style(style)
            })
            .collect();

        let features_list = List::new(feature_items)
            .block(Block::default())
            .highlight_style(
                Style::default()
                    .bg(Color::Rgb(40, 40, 65))
                    .add_modifier(Modifier::BOLD),
            );

        frame.render_widget(features_list, left_padded[1]);

        let right_panel_style = if self.active_panel == 1 && self.selection_mode == 0 {
            Style::default().fg(Color::Rgb(250, 250, 110))
        } else {
            Style::default().fg(Color::Rgb(140, 140, 200))
        };

        let right_panel_block = Block::default()
            .title(" Sub menu ")
            .title_style(
                Style::default()
                    .fg(Color::Rgb(230, 230, 250))
                    .add_modifier(Modifier::BOLD),
            )
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(right_panel_style)
            .style(Style::default().bg(Color::Rgb(22, 22, 35)));

        frame.render_widget(right_panel_block.clone(), content_layout[1]);
        let right_inner = right_panel_block.inner(content_layout[1]);

        let current_submenu = &self.submenu_options[self.selected_feature_index];
        let current_submenu_state = &self.submenu_states[self.selected_feature_index];

        let submenu_items: Vec<ListItem> = current_submenu
            .iter()
            .enumerate()
            .map(|(idx, option)| {
                let style = if current_submenu_state.selected() == Some(idx) {
                    if self.active_panel == 1 && self.selection_mode == 0 {
                        Style::default()
                            .fg(Color::Rgb(250, 250, 110))
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                            .fg(Color::Rgb(129, 199, 245))
                            .add_modifier(Modifier::BOLD)
                    }
                } else {
                    Style::default().fg(Color::Rgb(200, 200, 220))
                };

                let prefix = if current_submenu_state.selected() == Some(idx) {
                    " ► "
                } else {
                    "   "
                };

                ListItem::new(format!("{}{}", prefix, option)).style(style)
            })
            .collect();

        let submenu_list = List::new(submenu_items)
            .block(Block::default().padding(Padding::new(2, 0, 2, 0)))
            .highlight_style(
                Style::default()
                    .bg(Color::Rgb(40, 40, 65))
                    .add_modifier(Modifier::BOLD),
            );

        frame.render_widget(submenu_list, right_inner);

        let help_text =
            "←→: Switch panels | ↑↓: Navigate | Enter: Select | Tab: Logout | Esc: Back";
        let help_paragraph = Paragraph::new(help_text)
            .style(Style::default().fg(Color::Rgb(140, 140, 170)))
            .alignment(Alignment::Center);

        frame.render_widget(help_paragraph, main_layout[3]);

        let back_text = if self.selection_mode == 1 {
            "[ Logout ]"
        } else {
            "  Logout  "
        };

        let back_style = if self.selection_mode == 1 {
            Style::default()
                .fg(Color::Rgb(255, 100, 100))
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Rgb(180, 180, 200))
        };

        let back_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(if self.selection_mode == 1 {
                Style::default().fg(Color::Rgb(255, 100, 100))
            } else {
                Style::default().fg(Color::Rgb(100, 100, 140))
            })
            .style(Style::default().bg(Color::Rgb(26, 26, 36)));

        frame.render_widget(back_block.clone(), main_layout[4]);

        let inner_logout = back_block.inner(main_layout[4]);
        let back_paragraph = Paragraph::new(back_text)
            .style(back_style)
            .alignment(Alignment::Center);

        frame.render_widget(back_paragraph, inner_logout);

        if self.show_logout_dialog {
            self.render_logout_dialog(frame, area);
        }
    }
}

impl Home {
    fn render_logout_dialog(&self, frame: &mut Frame, area: Rect) {
        let dialog_width = 40;
        let dialog_height = 8;

        let dialog_area = Rect::new(
            (area.width.saturating_sub(dialog_width)) / 2,
            (area.height.saturating_sub(dialog_height)) / 2,
            dialog_width,
            dialog_height,
        );

        frame.render_widget(Clear, dialog_area);

        let dialog_block = Block::default()
            .title(" Confirm Logout ")
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

        let message = Paragraph::new("Are you sure you want to logout?")
            .style(Style::default().fg(Color::Rgb(220, 220, 240)))
            .add_modifier(Modifier::BOLD)
            .alignment(Alignment::Center);

        frame.render_widget(message, content_layout[0]);

        let buttons_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(content_layout[1]);

        let yes_style = if self.logout_dialog_selected == 0 {
            Style::default()
                .fg(Color::Rgb(140, 219, 140))
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Rgb(180, 180, 200))
        };

        let no_style = if self.logout_dialog_selected == 1 {
            Style::default()
                .fg(Color::Rgb(255, 100, 100))
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Rgb(180, 180, 200))
        };

        let yes_text = if self.logout_dialog_selected == 0 {
            "► Yes ◄"
        } else {
            "  Yes  "
        };

        let no_text = if self.logout_dialog_selected == 1 {
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
