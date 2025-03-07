//! Home component for Rustoria.

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

/// Represents the home screen UI component.
pub struct Home {
    /// The logged-in user's username.
    username: Option<String>,
    /// Selection mode: 0 for menus, 1 for back button
    selection_mode: usize,
    /// Show logout confirmation dialog
    show_logout_dialog: bool,
    /// Logout dialog selected option (0: Yes, 1: No)
    logout_dialog_selected: usize,
    /// Active panel (0 for left/features, 1 for right/submenus)
    active_panel: usize,
    /// Selected feature in the left panel
    selected_feature_index: usize,
    /// Submenu states for each feature
    submenu_states: Vec<ListState>,
    /// Feature names
    features: Vec<&'static str>,
    /// Submenu options for each feature
    submenu_options: Vec<Vec<&'static str>>,
}

impl Home {
    /// Creates a new `Home` component.
    pub fn new() -> Self {
        // Define feature names
        let features = vec![
            "Billing & Finance",
            "Inventory Management",
            "Medical Records",
            "Patient Management",
            "Reports & Analytics",
            "Staff Scheduling",
        ];

        // Define submenu options for each feature
        let submenu_options = vec![
            // Billing & Finance
            vec![
                "Generate Invoice",
                "View Billing Reports",
                "Process Payment",
            ],
            // Inventory Management
            vec!["Add Inventory", "Check Stock", "Auto Reorder"],
            // Medical Records
            vec!["Store Record", "Retrieve Records", "Update Record"],
            // Patient Management
            vec![
                "Add Patient",
                "List Patients",
                "Update Patient",
                "Delete Patient",
            ],
            // Reports & Analytics
            vec!["Generate Report", "Export Reports"],
            // Staff Scheduling
            vec!["Add Staff", "Assign Shift", "List Staff", "Remove Staff"],
        ];

        // Initialize submenu states for each feature
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

    /// Loads the username from the database.
    pub fn load_username(&mut self, user_id: i64) -> Result<()> {
        self.username = Some(db::get_username(user_id)?);
        Ok(())
    }

    /// Handles user input events.
    ///
    /// This function processes keyboard input to navigate the UI, select menu items, and handle the logout process.
    ///
    /// # Arguments
    ///
    /// * `key` - The key event received from the terminal.
    ///
    /// # Returns
    ///
    /// * `Result<Option<SelectedApp>>` -  Returns `Ok(Some(SelectedApp))` if a menu item is selected that navigates to a new app, `Ok(Some(SelectedApp::None))` if the user logs out, `Ok(None)` if the input is handled within the Home component, or `Err` if an error occurs.
    pub fn handle_input(&mut self, key: KeyEvent) -> Result<Option<SelectedApp>> {
        if self.show_logout_dialog {
            return self.handle_logout_dialog_input(key);
        }

        match key.code {
            KeyCode::Tab => {
                // Toggle between menu selection and back button
                self.selection_mode = (self.selection_mode + 1) % 2;
            }
            KeyCode::Left => {
                if self.selection_mode == 0 && self.active_panel == 1 {
                    // Move from right panel to left panel
                    self.active_panel = 0;
                }
            }
            KeyCode::Right => {
                if self.selection_mode == 0 && self.active_panel == 0 {
                    // Move from left panel to right panel
                    self.active_panel = 1;
                }
            }
            KeyCode::Up => {
                if self.selection_mode == 0 {
                    if self.active_panel == 0 {
                        // Navigate in the features panel (left)
                        if self.selected_feature_index > 0 {
                            self.selected_feature_index -= 1;
                        } else {
                            self.selected_feature_index = self.features.len() - 1;
                        }
                    } else {
                        // Navigate in the submenu panel (right)
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
                        // Navigate in the features panel (left)
                        self.selected_feature_index =
                            (self.selected_feature_index + 1) % self.features.len();
                    } else {
                        // Navigate in the submenu panel (right)
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
                        // Selected a submenu item - navigate to appropriate screen
                        let feature_idx = self.selected_feature_index;
                        let submenu_idx = self.submenu_states[feature_idx].selected().unwrap_or(0);

                        // Return different SelectedApp based on feature and submenu
                        return Ok(Some(match feature_idx {
                            // Billing & Finance
                            0 => match submenu_idx {
                                // 0 => SelectedApp::BillingInvoice,    // Generate Invoice
                                // 1 => SelectedApp::BillingReports,    // View Billing Reports
                                // 2 => SelectedApp::BillingPayment,    // Process Payment
                                _ => SelectedApp::Hospital,
                            },
                            // Inventory Management
                            1 => match submenu_idx {
                                // 0 => SelectedApp::InventoryAdd,      // Add Inventory
                                // 1 => SelectedApp::InventoryCheck,    // Check Stock
                                // 2 => SelectedApp::InventoryReorder,  // Auto Reorder
                                _ => SelectedApp::Hospital,
                            },
                            // Medical Records
                            2 => match submenu_idx {
                                // 0 => SelectedApp::MedicalStore,      // Store Record
                                // 1 => SelectedApp::MedicalRetrieve,   // Retrieve Records
                                // 2 => SelectedApp::MedicalUpdate,     // Update Record
                                _ => SelectedApp::Hospital,
                            },
                            // Patient Management
                            3 => match submenu_idx {
                                0 => SelectedApp::PatientAdd,    // Add Patient
                                1 => SelectedApp::PatientList,   // List Patients
                                2 => SelectedApp::PatientUpdate, // Update Patient
                                3 => SelectedApp::PatientDelete, // Delete Patient
                                _ => SelectedApp::Hospital,
                            },
                            // Reports & Analytics
                            4 => match submenu_idx {
                                // 0 => SelectedApp::ReportsGenerate,   // Generate Report
                                // 1 => SelectedApp::ReportsExport,     // Export Reports
                                _ => SelectedApp::Hospital,
                            },
                            // Staff Scheduling
                            5 => match submenu_idx {
                                0 => SelectedApp::StaffAdd, // Add Staff
                                // 1 => SelectedApp::StaffAssign,       // Assign Shift
                                2 => SelectedApp::StaffList, // List Staff
                                // 3 => SelectedApp::StaffRemove,       // Delete Staff
                                // 3 => SelectedApp::StaffUpdate,       // Update Staff
                                _ => SelectedApp::Hospital,
                            },
                            _ => SelectedApp::Hospital,
                        }));
                    } else {
                        // If in left panel, move to right panel
                        self.active_panel = 1;
                    }
                } else {
                    // Logout button - show confirmation dialog
                    self.show_logout_dialog = true;
                    self.logout_dialog_selected = 1; // Default to "No"
                }
            }
            KeyCode::Esc => {
                if self.active_panel == 1 {
                    // If in right panel, go back to left panel
                    self.active_panel = 0;
                } else {
                    // If in left panel, show logout confirmation
                    self.show_logout_dialog = true;
                    self.logout_dialog_selected = 1; // Default to "No"
                }
            }
            _ => {}
        }

        Ok(None)
    }

    /// Handles user input within the logout confirmation dialog.
    ///
    /// Processes keyboard input for selecting "Yes" or "No" in the logout confirmation dialog.
    ///
    /// # Arguments
    ///
    /// * `key` - The key event received from the terminal.
    ///
    /// # Returns
    ///
    /// * `Result<Option<SelectedApp>>` - Returns `Ok(Some(SelectedApp::None))` if the user confirms logout (selects "Yes"),  `Ok(None)` if the dialog is still active or cancelled, or `Err` if an error occurs.
    fn handle_logout_dialog_input(&mut self, key: KeyEvent) -> Result<Option<SelectedApp>> {
        match key.code {
            KeyCode::Left | KeyCode::Right => {
                self.logout_dialog_selected = 1 - self.logout_dialog_selected; // Toggle between Yes/No
            }
            KeyCode::Enter => {
                self.show_logout_dialog = false;
                if self.logout_dialog_selected == 0 {
                    // Yes selected - logout
                    return Ok(Some(SelectedApp::None));
                }
            }
            KeyCode::Esc => {
                self.show_logout_dialog = false; // Cancel dialog
            }
            _ => {}
        }
        Ok(None)
    }
}

impl Component for Home {
    /// Handles input events for the `Home` component.
    ///
    /// This function simply calls the `handle_input` method of the `Home` struct.
    ///
    /// # Arguments
    ///
    /// * `event` - The key event to handle.
    ///
    /// # Returns
    ///
    /// * `Result<Option<SelectedApp>>` - Returns the result of `handle_input`.
    fn handle_input(&mut self, event: KeyEvent) -> Result<Option<SelectedApp>> {
        self.handle_input(event)
    }

    /// Renders the `Home` component to the terminal.
    ///
    /// This function draws the UI elements of the home screen, including the welcome message,
    /// feature list, submenu options, and logout confirmation dialog.
    ///
    /// # Arguments
    ///
    /// * `frame` - A mutable reference to the `Frame` used for rendering.
    fn render(&self, frame: &mut Frame) {
        // Apply global background
        frame.render_widget(
            Block::default().style(Style::default().bg(Color::Rgb(16, 16, 28))),
            frame.area(),
        );

        let area = frame.area();

        // Main vertical layout
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5), // Welcome banner area
                Constraint::Length(1), // Instruction text
                Constraint::Min(10),   // Main content area
                Constraint::Length(3), // Help text
                Constraint::Length(3), // Logout button
            ])
            .split(area);

        // Welcome banner with gradient background
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

        // Instruction text
        let instruction = Paragraph::new("Please select a task:")
            .style(Style::default().fg(Color::Rgb(180, 190, 254)))
            .alignment(Alignment::Center);

        frame.render_widget(instruction, main_layout[1]);

        // Main content area - split into left and right panels
        let content_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(40), // Left panel (features)
                Constraint::Percentage(60), // Right panel (submenu)
            ])
            .spacing(2)
            .margin(1)
            .split(main_layout[2]);

        // Left panel - Hospital Management Features
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

        // Apply padding to left panel content with extra top padding
        let left_padded = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Added extra top padding
                Constraint::Min(1),
            ])
            .split(left_inner);

        // Left panel content (Feature list)
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

                // Add icons for each feature
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

        // Right panel - Submenu options
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

        // Current submenu options for the selected feature
        let current_submenu = &self.submenu_options[self.selected_feature_index];
        let current_submenu_state = &self.submenu_states[self.selected_feature_index];

        // Right panel content (Submenu list) - without the feature name as title
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

        // Help text
        let help_text =
            "←→: Switch panels | ↑↓: Navigate | Enter: Select | Tab: Logout | Esc: Back";
        let help_paragraph = Paragraph::new(help_text)
            .style(Style::default().fg(Color::Rgb(140, 140, 170)))
            .alignment(Alignment::Center);

        frame.render_widget(help_paragraph, main_layout[3]);

        // Logout button
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

        // Render logout confirmation dialog if needed
        if self.show_logout_dialog {
            self.render_logout_dialog(frame, area);
        }
    }
}

impl Home {
    /// Renders the logout confirmation dialog.
    ///
    /// This function displays a dialog box asking the user to confirm their logout.
    ///
    /// # Arguments
    ///
    /// * `frame` - A mutable reference to the `Frame` used for rendering.
    /// * `area` - The `Rect` representing the available area for rendering.
    fn render_logout_dialog(&self, frame: &mut Frame, area: Rect) {
        let dialog_width = 40;
        let dialog_height = 8;

        let dialog_area = Rect::new(
            (area.width.saturating_sub(dialog_width)) / 2,
            (area.height.saturating_sub(dialog_height)) / 2,
            dialog_width,
            dialog_height,
        );

        // Clear the background
        frame.render_widget(Clear, dialog_area);

        // Render dialog box - updated to match the app theme
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

        // Dialog content
        let inner_area = dialog_block.inner(dialog_area);

        let content_layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(2), // Message
                Constraint::Length(2), // Buttons
            ])
            .split(inner_area);

        let message = Paragraph::new("Are you sure you want to logout?")
            .style(Style::default().fg(Color::Rgb(220, 220, 240)))
            .add_modifier(Modifier::BOLD)
            .alignment(Alignment::Center);

        frame.render_widget(message, content_layout[0]);

        // Buttons
        let buttons_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(content_layout[1]);

        // Updated button styles to match the app theme
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
