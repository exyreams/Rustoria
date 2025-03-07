//! List Staff component for the Hospital application.
//!
//! This module provides functionality to display and search through a list of staff members.
//! It supports:
//! - Viewing all staff in a tabular format
//! - Searching staff by name, ID, phone, or address
//! - Navigating through staff using keyboard shortcuts
//! - Viewing detailed information about selected staff

use crate::components::hospital::staff::StaffAction;
use crate::components::Component;
use crate::db;
use crate::models::StaffMember;
use crate::tui::Frame;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{prelude::*, widgets::*};

/// UI focus indices for navigating the staff list interface
const SEARCH_FIELD: usize = 0;
const STAFF_LIST: usize = 1;
const BACK_BUTTON: usize = 2;

/// Component for displaying and interacting with a list of hospital staff members.
///
/// This component manages staff data retrieval, search functionality, keyboard navigation,
/// and detailed staff information display.
pub struct ListStaff {
    /// Complete collection of staff members from the database
    staff: Vec<StaffMember>,
    /// Staff members filtered by the current search query
    filtered_staff: Vec<StaffMember>,
    /// Current text in the search input field
    search_input: String,
    /// Whether the user is currently inputting a search query
    is_searching: bool,
    /// Selection state for the staff table
    state: TableState,
    /// Optional error message to display at the bottom of the screen
    error_message: Option<String>,
    /// Whether to display detailed information for the selected staff member
    show_details: bool,
    /// Current UI element that has focus (search field, staff list, or back button)
    focus_index: usize,
}

impl ListStaff {
    /// Creates a new ListStaff component with default values.
    ///
    /// The component starts with focus on the staff list and no staff loaded.
    pub fn new() -> Self {
        Self {
            staff: Vec::new(),
            filtered_staff: Vec::new(),
            search_input: String::new(),
            is_searching: false,
            state: TableState::default(),
            error_message: None,
            show_details: false,
            focus_index: STAFF_LIST,
        }
    }

    /// Retrieves all staff members from the database and updates the component state.
    ///
    /// This method maintains the current search filter and selection state when possible.
    /// If an error occurs during retrieval, it's stored in the error_message field.
    pub fn fetch_staff(&mut self) -> Result<()> {
        match db::get_all_staff() {
            Ok(staff) => {
                self.staff = staff;
                // Apply the current search filter to the new staff data
                self.filter_staff();

                // Update the selection state based on the filtered results
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

    /// Filters the staff list based on the current search query.
    ///
    /// Searches across multiple fields: name, ID, phone number, and address.
    /// Also ensures that the selection state remains valid after filtering.
    fn filter_staff(&mut self) {
        if self.search_input.is_empty() {
            // If no search query, show all staff
            self.filtered_staff = self.staff.clone();
        } else {
            // Filter based on search term across multiple fields
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

        // Adjust the selection to remain valid after filtering
        if let Some(selected) = self.state.selected() {
            if selected >= self.filtered_staff.len() && !self.filtered_staff.is_empty() {
                self.state.select(Some(0));
            } else if self.filtered_staff.is_empty() {
                self.state.select(None);
            }
        }
    }

    /// Selects the next staff member in the filtered list.
    ///
    /// Wraps around to the beginning when reaching the end of the list.
    /// Does nothing if the filtered list is empty.
    fn select_next(&mut self) {
        if self.filtered_staff.is_empty() {
            return;
        }

        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.filtered_staff.len() - 1 {
                    0 // Wrap to beginning
                } else {
                    i + 1
                }
            }
            None => 0, // Select first item if nothing is selected
        };
        self.state.select(Some(i));
    }

    /// Selects the previous staff member in the filtered list.
    ///
    /// Wraps around to the end when reaching the beginning of the list.
    /// Does nothing if the filtered list is empty.
    fn select_previous(&mut self) {
        if self.filtered_staff.is_empty() {
            return;
        }

        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.filtered_staff.len() - 1 // Wrap to end
                } else {
                    i - 1
                }
            }
            None => 0, // Select first item if nothing is selected
        };
        self.state.select(Some(i));
    }

    /// Toggles detailed view for the currently selected staff member.
    ///
    /// Does nothing if no staff member is selected or the list is empty.
    fn toggle_details(&mut self) {
        if !self.filtered_staff.is_empty() && self.state.selected().is_some() {
            self.show_details = !self.show_details;
        }
    }

    /// Moves focus to the next UI element in the tab order.
    ///
    /// Cycles through search field, staff list, and back button.
    /// Updates the is_searching flag when appropriate.
    fn focus_next(&mut self) {
        self.focus_index = (self.focus_index + 1) % 3;
        self.is_searching = self.focus_index == SEARCH_FIELD;
    }

    /// Moves focus to the previous UI element in the tab order.
    ///
    /// Cycles through search field, staff list, and back button.
    /// Updates the is_searching flag when appropriate.
    fn focus_previous(&mut self) {
        self.focus_index = (self.focus_index + 2) % 3;
        self.is_searching = self.focus_index == SEARCH_FIELD;
    }

    /// Activates search mode by focusing the search field and setting appropriate flags.
    fn activate_search(&mut self) {
        self.is_searching = true;
        self.focus_index = SEARCH_FIELD;
    }

    /// Processes keyboard input and updates component state accordingly.
    ///
    /// Handles navigation, search, selection, and action triggers.
    /// Returns Some(StaffAction) when an action should be taken by the parent component.
    pub fn process_input(&mut self, key: KeyEvent) -> Result<Option<StaffAction>> {
        // Handle input differently when in search mode
        if self.is_searching {
            match key.code {
                KeyCode::Char(c) => {
                    // Add character to search input
                    self.search_input.push(c);
                    self.filter_staff();
                }
                KeyCode::Backspace => {
                    // Remove character from search input
                    self.search_input.pop();
                    self.filter_staff();
                }
                KeyCode::Enter | KeyCode::Down | KeyCode::Tab => {
                    // Exit search mode and focus the list if there are results
                    if !self.filtered_staff.is_empty() {
                        self.is_searching = false;
                        self.focus_index = STAFF_LIST;
                        self.state.select(Some(0));
                    }
                }
                KeyCode::Esc => {
                    // Cancel search
                    self.is_searching = false;
                    self.focus_index = STAFF_LIST;
                }
                _ => {}
            }
            return Ok(None);
        }

        // Handle input in normal mode
        match key.code {
            // Activate search mode with '/' or 's'
            KeyCode::Char(c) if c == '/' || c == 's' || c == 'S' => {
                self.activate_search();
                return Ok(None);
            }
            // Tab navigation
            KeyCode::Tab => self.focus_next(),
            KeyCode::BackTab => self.focus_previous(),
            // Vertical navigation
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
            // Horizontal navigation
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
            // Action selection
            KeyCode::Enter => {
                if self.focus_index == BACK_BUTTON {
                    return Ok(Some(StaffAction::BackToHome));
                } else if self.focus_index == STAFF_LIST {
                    self.toggle_details();
                } else if self.focus_index == SEARCH_FIELD {
                    self.activate_search();
                }
            }
            // Shortcut keys
            KeyCode::Char('b') | KeyCode::Char('B') => {
                return Ok(Some(StaffAction::BackToHome));
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                self.fetch_staff()?;
            }
            // Escape handling
            KeyCode::Esc => {
                if self.show_details {
                    self.show_details = false;
                } else {
                    return Ok(Some(StaffAction::BackToHome));
                }
            }
            _ => {}
        }
        Ok(None)
    }

    /// Returns a reference to the currently selected staff member, if any.
    fn selected_staff(&self) -> Option<&StaffMember> {
        self.state
            .selected()
            .and_then(|i| self.filtered_staff.get(i))
    }
}

impl Component for ListStaff {
    /// Handles input events and converts internal actions to application actions.
    fn handle_input(&mut self, event: KeyEvent) -> Result<Option<crate::app::SelectedApp>> {
        match self.process_input(event)? {
            Some(StaffAction::BackToHome) => Ok(Some(crate::app::SelectedApp::None)),
            Some(StaffAction::BackToList) => Ok(None),
            None => Ok(None),
        }
    }

    /// Renders the staff list UI to the terminal frame.
    ///
    /// This includes the header, search field, staff table, details view,
    /// help text, back button, and error messages if present.
    fn render(&self, frame: &mut Frame) {
        // Set the base background color
        let area = frame.area();
        frame.render_widget(
            Block::default().style(Style::default().bg(Color::Rgb(16, 16, 28))),
            area,
        );

        // Create the main layout
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Length(3), // Search input
                Constraint::Min(10),   // Table
                Constraint::Length(3), // Staff details or help text
                Constraint::Length(2), // Back button
                Constraint::Length(1), // Error Message
            ])
            .margin(1)
            .split(area);

        // Render the header
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

        // Render the search box
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

        // Prepare table header
        let header_cells = ["ID", "Name", "Role", "Phone", "Address"]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().fg(Color::Rgb(230, 230, 250))));
        let header = Row::new(header_cells)
            .style(Style::default().bg(Color::Rgb(80, 60, 130)))
            .height(1);

        // Prepare table rows
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

        // Define table selection style
        let selected_style = Style::default()
            .fg(Color::Rgb(250, 250, 110))
            .bg(Color::Rgb(40, 40, 60))
            .add_modifier(Modifier::BOLD);

        // Create dynamic table title based on search state
        let table_title = if !self.search_input.is_empty() {
            format!(
                " Staff ({} of {} matches) ",
                self.filtered_staff.len(),
                self.staff.len()
            )
        } else {
            format!(" Staff ({}) ", self.staff.len())
        };

        // Create the staff table
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

        // Show a message if no staff found
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

        // Show either staff details or help text
        if self.show_details && self.state.selected().is_some() {
            if let Some(staff_member) = self.selected_staff() {
                let details = format!(
                    "Details for {}: Role: {}, Phone: {}, Address: {}",
                    staff_member.name,
                    match staff_member.role {
                        crate::models::StaffRole::Doctor => "Doctor",
                        crate::models::StaffRole::Nurse => "Nurse",
                        crate::models::StaffRole::Admin => "Admin",
                        crate::models::StaffRole::Technician => "Technician",
                    },
                    staff_member.phone_number,
                    staff_member.address
                );

                let details_widget = Paragraph::new(details)
                    .style(Style::default().fg(Color::Rgb(200, 200, 220)))
                    .block(
                        Block::default()
                            .title(" Staff Details ")
                            .borders(Borders::ALL)
                            .border_type(BorderType::Rounded)
                            .border_style(Style::default().fg(Color::Rgb(75, 75, 120))),
                    )
                    .wrap(Wrap { trim: true });

                frame.render_widget(details_widget, layout[3]);
            }
        } else {
            // Display contextual help based on mode
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

        // Render the back button
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

        // Show error message if present
        if let Some(error) = &self.error_message {
            let error_paragraph = Paragraph::new(error.as_str())
                .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
                .alignment(Alignment::Center);
            frame.render_widget(error_paragraph, layout[5]);
        }
    }
}

impl Default for ListStaff {
    fn default() -> Self {
        Self::new()
    }
}
