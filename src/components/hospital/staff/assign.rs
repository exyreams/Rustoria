//! Assign Staff component for the Hospital application.
//!
//! This module provides a UI for assigning shifts to staff members. It features:
//! - Staff selection using a searchable table.
//! - Date selection using a 6-month calendar widget.
//! - Shift selection with arrow-based navigation.
//! - View assigned shifts for staff members.
//! - Confirmation dialog before saving.
//! - Error and success message handling.

use crate::app::SelectedApp;
use crate::components::Component;
use crate::db;
use crate::models::StaffMember;
use crate::tui::Frame;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::calendar::{CalendarEventStore, Monthly};
use ratatui::{prelude::*, widgets::*};
use std::time::{Duration, Instant};
use time::macros::format_description;
use time::Date;
use time::Weekday;

/// Represents the different shifts available.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shift {
    /// Morning shift.
    Morning,
    /// Afternoon shift.
    Afternoon,
    /// Night shift.
    Night,
}

/// Represents the different states of the AssignStaff component.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignState {
    /// The user is selecting a staff member.
    SelectingStaff,
    /// The user is selecting a date.
    SelectingDate,
    /// The user is selecting a shift.
    SelectingShift,
    /// The user is viewing assigned shifts for a staff member.
    ViewingAssignments,
    /// The user is confirming the assignment (this state is handled directly in the input and render methods).
    #[allow(dead_code)]
    Confirming,
}

/// Component for assigning shifts to staff members.
pub struct AssignStaff {
    /// All staff (for the selection table).
    staff: Vec<StaffMember>,
    /// Filtered staff based on search input.
    filtered_staff: Vec<StaffMember>,
    /// Search input for filtering staff.
    search_input: String,
    /// Flag indicating if search is active.
    is_searching: bool,
    /// Table state for staff selection.
    table_state: TableState,
    /// List state for shift selection.
    shift_list_state: ListState,
    /// The selected staff member.
    selected_staff: Option<StaffMember>,
    /// Current state of the component.
    assign_state: AssignState,
    /// The date selected in the calendar.
    selected_date: Option<Date>,
    /// The selected shift (Morning, Afternoon, Night).
    selected_shift: Option<Shift>,
    /// Whether the confirmation dialog is visible.
    show_confirmation: bool,
    /// Error message, if any.
    error_message: Option<String>,
    /// Timer for error message display.
    error_timer: Option<Instant>,
    /// Success message, if any.
    success_message: Option<String>,
    /// Timer for success message display.
    success_timer: Option<Instant>,
    /// Cached staff assignments for viewing
    staff_assignments: Vec<(Date, String)>,
    /// Currently focused month index (for Tab navigation)
    focused_month: usize,
}

impl AssignStaff {
    /// Creates a new `AssignStaff` component.
    pub fn new() -> Self {
        let mut shift_list_state = ListState::default();
        shift_list_state.select(Some(0)); // Default to first shift

        Self {
            staff: Vec::new(),
            filtered_staff: Vec::new(),
            search_input: String::new(),
            is_searching: false,
            table_state: TableState::default(),
            shift_list_state,
            selected_staff: None,
            assign_state: AssignState::SelectingStaff,
            selected_date: None,
            selected_shift: None,
            show_confirmation: false,
            error_message: None,
            error_timer: None,
            success_message: None,
            success_timer: None,
            staff_assignments: Vec::new(),
            focused_month: 0, // Start with the first month focused
        }
    }

    /// Fetches all staff members from the database.
    pub fn fetch_staff(&mut self) -> Result<()> {
        self.staff = db::get_all_staff()?;
        self.filter_staff();

        // Select the first item if the filtered list is not empty.
        if !self.filtered_staff.is_empty() {
            self.table_state.select(Some(0));
        } else {
            self.table_state.select(None);
        }

        Ok(())
    }

    /// Fetches assigned shifts for a staff member.
    fn fetch_staff_assignments(&mut self, staff_id: i64) -> Result<()> {
        // Call the database function to get assigned shifts
        match db::get_assigned_shifts_for_staff(staff_id) {
            Ok(assignments) => {
                self.staff_assignments = assignments;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    /// Filters the staff list based on the current search input.
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
                })
                .cloned()
                .collect();
        }

        // Always select the first item after filtering if list isn't empty
        if !self.filtered_staff.is_empty() {
            self.table_state.select(Some(0));
        } else {
            self.table_state.select(None);
        }
    }

    /// Selects the next staff member in the table.
    fn select_next_staff(&mut self) {
        if self.filtered_staff.is_empty() {
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => {
                if i >= self.filtered_staff.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    /// Selects the previous staff member in the table.
    fn select_previous_staff(&mut self) {
        if self.filtered_staff.is_empty() {
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.filtered_staff.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    /// Loads the selected staff member from the table.
    fn load_selected_staff(&mut self) -> Result<()> {
        if let Some(selected) = self.table_state.selected() {
            if selected < self.filtered_staff.len() {
                self.selected_staff = Some(self.filtered_staff[selected].clone());
                self.assign_state = AssignState::SelectingDate;

                // Initialize the selected date to today if not already set
                if self.selected_date.is_none() {
                    self.selected_date = Some(time::OffsetDateTime::now_utc().date());
                }

                // Reset focused month when entering date selection
                self.focused_month = 0;

                return Ok(());
            }
        }
        self.set_error("No staff selected".to_string());
        Err(anyhow::anyhow!("No staff selected"))
    }

    /// Cycles to the next month in calendar view when Tab is pressed
    fn cycle_month_focus(&mut self) {
        self.focused_month = (self.focused_month + 1) % 6;

        // Update selected date to be day 1 of the focused month
        let today = time::OffsetDateTime::now_utc().date();
        let mut current_date = today;

        // Find the selected month based on focused_month
        for _ in 0..self.focused_month {
            if let Some(next_month) = current_date.checked_add(time::Duration::days(32)) {
                current_date =
                    time::Date::from_calendar_date(next_month.year(), next_month.month(), 1)
                        .unwrap_or(next_month);
            }
        }

        // Set selected date to the 1st of the focused month
        self.selected_date = Some(
            time::Date::from_calendar_date(current_date.year(), current_date.month(), 1)
                .unwrap_or(current_date),
        );
    }

    /// Handles arrow key navigation within date selection, keeping within current month
    /// Handles arrow key navigation within date selection, keeping within current month
    fn navigate_date(&mut self, direction: &str) {
        if let Some(date) = self.selected_date {
            let new_date = match direction {
                "left" => {
                    // Go to previous day, but stay in current month
                    if date.day() > 1 {
                        if let Some(prev) = date.previous_day() {
                            prev
                        } else {
                            date
                        }
                    } else {
                        // Wrap to end of month
                        let days_in_month = self.get_days_in_month(date.year(), date.month());
                        time::Date::from_calendar_date(date.year(), date.month(), days_in_month)
                            .unwrap_or(date)
                    }
                }
                "right" => {
                    // Go to next day, but stay in current month
                    let days_in_month = self.get_days_in_month(date.year(), date.month());
                    if date.day() < days_in_month {
                        if let Some(next) = date.next_day() {
                            next
                        } else {
                            date
                        }
                    } else {
                        // Wrap to beginning of month
                        time::Date::from_calendar_date(date.year(), date.month(), 1).unwrap_or(date)
                    }
                }
                "up" => {
                    // Go up a week, but stay in current month
                    let new_day = if date.day() > 7 {
                        date.day() - 7
                    } else {
                        // Wrap to last part of month
                        let days_in_month = self.get_days_in_month(date.year(), date.month());
                        let offset = 7 - date.day();
                        if days_in_month >= date.day() + (7 - offset) {
                            days_in_month - (7 - offset)
                        } else {
                            days_in_month
                        }
                    };

                    time::Date::from_calendar_date(date.year(), date.month(), new_day)
                        .unwrap_or(date)
                }
                "down" => {
                    // Go down a week, but stay in current month
                    let days_in_month = self.get_days_in_month(date.year(), date.month());
                    let new_day = if date.day() + 7 <= days_in_month {
                        date.day() + 7
                    } else {
                        // Wrap to first part of month
                        date.day() + 7 - days_in_month
                    };

                    time::Date::from_calendar_date(date.year(), date.month(), new_day)
                        .unwrap_or(date)
                }
                _ => date,
            };

            self.selected_date = Some(new_date);

            // Update focused_month to match the month of the selected date
            let today = time::OffsetDateTime::now_utc().date();
            let mut current_date = today;
            let mut month_index = 0;

            // Find the index of the month containing the selected date
            for i in 0..6 {
                if new_date.year() == current_date.year()
                    && new_date.month() == current_date.month()
                {
                    month_index = i;
                    break;
                }

                if let Some(next_month) = current_date.checked_add(time::Duration::days(32)) {
                    current_date =
                        time::Date::from_calendar_date(next_month.year(), next_month.month(), 1)
                            .unwrap_or(next_month);
                }
            }

            self.focused_month = month_index;
        } else {
            self.selected_date = Some(time::OffsetDateTime::now_utc().date());
        }
    }

    /// Helper function to get the number of days in a month
    fn get_days_in_month(&self, year: i32, month: time::Month) -> u8 {
        let current = time::Date::from_calendar_date(year, month, 1).unwrap();
        let next_month = if month == time::Month::December {
            time::Date::from_calendar_date(year + 1, time::Month::January, 1).unwrap()
        } else {
            let next_month_num = month as u8 + 1;
            let next_month = time::Month::try_from(next_month_num).unwrap();
            time::Date::from_calendar_date(year, next_month, 1).unwrap()
        };

        let days_diff = next_month.to_julian_day() - current.to_julian_day();
        days_diff as u8
    }

    /// Handles the assignment of the shift in the database.
    fn assign_shift(&mut self) -> Result<()> {
        if let (Some(staff), Some(date), Some(shift)) = (
            &self.selected_staff,
            &self.selected_date,
            &self.selected_shift,
        ) {
            let shift_str = match shift {
                Shift::Morning => "Morning",
                Shift::Afternoon => "Afternoon",
                Shift::Night => "Night",
            };

            match db::assign_staff_shift(staff.id, date, shift_str) {
                Ok(_) => {
                    self.success_message =
                        Some(format!("Shift assigned to {} successfully!", staff.name));
                    self.success_timer = Some(Instant::now());
                    self.reset();
                    Ok(())
                }
                Err(e) => {
                    self.set_error(format!("Database error: {}", e));
                    Err(e)
                }
            }
        } else {
            self.set_error("Please select staff, date, and shift.".to_string());
            Err(anyhow::anyhow!(
                "Staff, date, or shift not selected for assignment."
            ))
        }
    }

    /// Resets the component to its initial state.
    fn reset(&mut self) {
        self.selected_staff = None;
        self.selected_date = None;
        self.selected_shift = None;
        self.assign_state = AssignState::SelectingStaff;
        self.show_confirmation = false;
        self.clear_error();
        self.clear_success();
        self.staff_assignments.clear();
        self.focused_month = 0;

        // Refresh staff data
        if let Ok(staff) = db::get_all_staff() {
            self.staff = staff;
            self.filter_staff();
        }
    }

    /// Sets an error message and starts the error timer.
    fn set_error(&mut self, message: String) {
        self.error_message = Some(message);
        self.error_timer = Some(Instant::now());
    }

    /// Clears the error message and timer.
    fn clear_error(&mut self) {
        self.error_message = None;
        self.error_timer = None;
    }

    /// Clears the success message and timer.
    fn clear_success(&mut self) {
        self.success_message = None;
        self.success_timer = None;
    }

    /// Checks if the error message timeout has been reached.
    fn check_error_timeout(&mut self) {
        if let Some(timer) = self.error_timer {
            if timer.elapsed() > Duration::from_secs(5) {
                self.clear_error();
            }
        }
    }

    /// Checks if the success message timeout has been reached.
    fn check_success_timeout(&mut self) {
        if let Some(timer) = self.success_timer {
            if timer.elapsed() > Duration::from_secs(5) {
                self.clear_success();
            }
        }
    }

    /// Checks both error and success message timeouts.
    fn check_timeouts(&mut self) {
        self.check_error_timeout();
        self.check_success_timeout();
    }

    /// Handles user input events for the component.
    fn handle_input(&mut self, key: KeyEvent) -> Result<Option<SelectedApp>> {
        self.check_timeouts();

        if self.show_confirmation {
            match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    self.show_confirmation = false;
                    let _ = self.assign_shift();
                    return Ok(None);
                }
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                    self.show_confirmation = false;
                }
                _ => {}
            }
            return Ok(None);
        }

        match self.assign_state {
            AssignState::SelectingStaff => {
                match key.code {
                    // Search mode handling
                    KeyCode::Char(c) if self.is_searching => {
                        self.search_input.push(c);
                        self.filter_staff();
                        self.clear_error();
                    }
                    KeyCode::Backspace if self.is_searching => {
                        self.search_input.pop();
                        self.filter_staff();
                        self.clear_error();
                    }
                    KeyCode::Down if self.is_searching && !self.filtered_staff.is_empty() => {
                        // Move focus from search field to table
                        self.is_searching = false;
                        self.table_state.select(Some(0));
                    }
                    KeyCode::Esc if self.is_searching => {
                        // Cancel search
                        self.is_searching = false;
                        self.search_input.clear();
                        self.filter_staff();
                    }
                    KeyCode::Char('/') | KeyCode::Char('s') | KeyCode::Char('S')
                        if !self.is_searching =>
                    {
                        self.is_searching = true; // Start searching.
                    }

                    // Navigation in table
                    KeyCode::Up => self.select_previous_staff(),
                    KeyCode::Down => self.select_next_staff(),

                    // Staff selection
                    KeyCode::Enter => {
                        let _ = self.load_selected_staff();
                    }

                    // View assignments
                    KeyCode::Char('v') | KeyCode::Char('V') => {
                        if let Some(selected) = self.table_state.selected() {
                            if selected < self.filtered_staff.len() {
                                self.selected_staff = Some(self.filtered_staff[selected].clone());
                                // Fetch assignments for the selected staff
                                if let Err(e) =
                                    self.fetch_staff_assignments(self.filtered_staff[selected].id)
                                {
                                    self.set_error(format!("Failed to load assignments: {}", e));
                                } else {
                                    self.assign_state = AssignState::ViewingAssignments;
                                }
                            } else {
                                self.set_error("No staff selected".to_string());
                            }
                        } else {
                            self.set_error("No staff selected".to_string());
                        }
                    }

                    KeyCode::Esc => {
                        // Back to main menu
                        return Ok(Some(SelectedApp::None));
                    }
                    _ => {}
                }
            }
            AssignState::SelectingDate => {
                match key.code {
                    KeyCode::Left => {
                        self.navigate_date("left");
                    }
                    KeyCode::Right => {
                        self.navigate_date("right");
                    }
                    KeyCode::Up => {
                        self.navigate_date("up");
                    }
                    KeyCode::Down => {
                        self.navigate_date("down");
                    }
                    KeyCode::Tab => {
                        // Add Tab key to cycle through months
                        self.cycle_month_focus();
                    }
                    KeyCode::Enter => {
                        // Confirm date selection, move to shift selection.
                        if self.selected_date.is_some() {
                            self.assign_state = AssignState::SelectingShift;
                            // Initialize shift selection
                            self.selected_shift = Some(Shift::Morning);
                            self.shift_list_state.select(Some(0));
                        } else {
                            self.selected_date = Some(time::OffsetDateTime::now_utc().date());
                        }
                    }
                    KeyCode::Esc => {
                        // Go back to staff selection.
                        self.assign_state = AssignState::SelectingStaff;
                    }
                    _ => {}
                }
            }
            AssignState::SelectingShift => {
                match key.code {
                    // Navigate with arrows (new approach)
                    KeyCode::Up => {
                        let new_index = match self.shift_list_state.selected() {
                            Some(0) => 2, // Wrap around to bottom
                            Some(i) => i - 1,
                            None => 0,
                        };
                        self.shift_list_state.select(Some(new_index));
                        self.selected_shift = match new_index {
                            0 => Some(Shift::Morning),
                            1 => Some(Shift::Afternoon),
                            2 => Some(Shift::Night),
                            _ => Some(Shift::Morning),
                        };
                    }
                    KeyCode::Down => {
                        let new_index = match self.shift_list_state.selected() {
                            Some(2) => 0, // Wrap around to top
                            Some(i) => i + 1,
                            None => 0,
                        };
                        self.shift_list_state.select(Some(new_index));
                        self.selected_shift = match new_index {
                            0 => Some(Shift::Morning),
                            1 => Some(Shift::Afternoon),
                            2 => Some(Shift::Night),
                            _ => Some(Shift::Morning),
                        };
                    }
                    KeyCode::Enter => {
                        if self.selected_shift.is_some() {
                            self.show_confirmation = true; // Show confirmation.
                        }
                    }
                    KeyCode::Esc => {
                        // Go back to date selection
                        self.assign_state = AssignState::SelectingDate;
                    }
                    _ => {}
                }
            }
            AssignState::ViewingAssignments => {
                match key.code {
                    KeyCode::Esc => {
                        // Go back to staff selection
                        self.assign_state = AssignState::SelectingStaff;
                    }
                    _ => {}
                }
            }
            AssignState::Confirming => {
                // Handled at the beginning of the function
            }
        }

        Ok(None)
    }
}

impl Default for AssignStaff {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for AssignStaff {
    /// Handles user input events for the component.
    fn handle_input(&mut self, event: KeyEvent) -> Result<Option<SelectedApp>> {
        self.handle_input(event)
    }

    /// Renders the component to the terminal.
    fn render(&self, frame: &mut Frame) {
        match self.assign_state {
            AssignState::SelectingStaff => self.render_staff_selection(frame),
            AssignState::SelectingDate => self.render_date_selection(frame),
            AssignState::SelectingShift => self.render_shift_selection(frame),
            AssignState::ViewingAssignments => self.render_assigned_shifts(frame),
            AssignState::Confirming => { /* Rendered in other states */ }
        }

        if self.show_confirmation {
            self.render_confirmation_dialog(frame);
        }
    }
}

// Rendering helper methods
impl AssignStaff {
    /// Renders the staff selection screen.
    fn render_staff_selection(&self, frame: &mut Frame) {
        let area = frame.area();

        // Add a subtle background pattern
        let background = Block::default().style(Style::default().bg(Color::Rgb(16, 16, 28)));
        frame.render_widget(background, area);

        // Create a very subtle pattern using direct buffer access
        for y in (0..area.height).step_by(4) {
            for x in (0..area.width).step_by(8) {
                if (x + y) % 8 == 0 {
                    let pos = (area.x + x, area.y + y);
                    if pos.0 < frame.area().width && pos.1 < frame.area().height {
                        frame.buffer_mut()[pos].set_bg(Color::Rgb(20, 20, 32));
                    }
                }
            }
        }

        // Main layout with help text at bottom
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Length(3), // Search input
                Constraint::Min(5),    // Table
                Constraint::Length(1), // Message area
                Constraint::Length(2), // Help text
            ])
            .margin(2) // Increased margin
            .split(area);

        // Header
        let header_block = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(Color::Rgb(75, 75, 120)))
            .style(Style::default().bg(Color::Rgb(16, 16, 28)));
        frame.render_widget(header_block, layout[0]);

        let title = Paragraph::new(" 📅 SELECT STAFF TO ASSIGN SHIFT")
            .style(
                Style::default()
                    .fg(Color::Rgb(230, 230, 250))
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Rgb(16, 16, 28)),
            )
            .alignment(Alignment::Center);
        frame.render_widget(title, layout[0]);

        // Search input field
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

        // Staff table
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
            .fg(Color::Rgb(250, 250, 110))
            .bg(Color::Rgb(40, 40, 60))
            .add_modifier(Modifier::BOLD);
        let normal_style = Style::default()
            .bg(Color::Rgb(26, 26, 36))
            .fg(Color::Rgb(220, 220, 240));

        // Create table rows
        let mut rows = Vec::new();
        for staff_member in &self.filtered_staff {
            rows.push(Row::new(vec![
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
            ]));
        }

        if self.filtered_staff.is_empty() {
            // Fix: Create a centered message without alignment method
            let message = if self.search_input.is_empty() {
                "No staff found in database"
            } else {
                "No staff match your search criteria"
            };

            // Create a single centered cell that spans all columns
            let table_width = layout[2].width as usize - 4; // Accounting for borders
            let padded_message = format!("{:^width$}", message, width = table_width);
            rows.push(
                Row::new(vec![Cell::from(padded_message)
                    .style(Style::default().fg(Color::Rgb(180, 180, 200)))])
                .height(2),
            );
        }

        // Adjust column widths to better distribute space
        let (constraints, header_cells) = if self.filtered_staff.is_empty() {
            // When empty, use a single column constraint
            (
                vec![Constraint::Percentage(100)],
                vec![Cell::from("")], // Empty header
            )
        } else {
            (
                vec![
                    Constraint::Length(6),      // ID - slightly smaller
                    Constraint::Percentage(35), // Name - percentage based
                    Constraint::Percentage(25), // Role - percentage based
                    Constraint::Percentage(40), // Phone - wider to fill empty space
                ],
                vec![
                    Cell::from("ID").style(Style::default().add_modifier(Modifier::BOLD)),
                    Cell::from("Name").style(Style::default().add_modifier(Modifier::BOLD)),
                    Cell::from("Role").style(Style::default().add_modifier(Modifier::BOLD)),
                    Cell::from("Phone").style(Style::default().add_modifier(Modifier::BOLD)),
                ],
            )
        };

        let table = Table::new(rows, constraints.clone())
            .header(
                Row::new(header_cells)
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

        // Fix: Create a mutable copy of the table state for rendering
        let mut table_state_copy = self.table_state.clone();
        frame.render_stateful_widget(table, layout[2], &mut table_state_copy);

        // Display error message
        if let Some(error) = &self.error_message {
            let error_paragraph = Paragraph::new(error.as_str())
                .style(
                    Style::default()
                        .fg(Color::Rgb(240, 100, 100))
                        .add_modifier(Modifier::BOLD),
                )
                .alignment(Alignment::Center);
            frame.render_widget(error_paragraph, layout[3]);
        } else if let Some(success) = &self.success_message {
            let success_paragraph = Paragraph::new(success.as_str())
                .style(
                    Style::default()
                        .fg(Color::Rgb(140, 219, 140))
                        .add_modifier(Modifier::BOLD),
                )
                .alignment(Alignment::Center);
            frame.render_widget(success_paragraph, layout[3]);
        }

        // Help text
        let help_text = if self.is_searching {
            "Type to search | ↓: To results | Esc: Cancel search"
        } else {
            "/ or s: Search | ↑/↓: Navigate | Enter: Select staff | v: View assignments | Esc: Back"
        };

        let help_paragraph = Paragraph::new(help_text)
            .style(Style::default().fg(Color::Rgb(180, 180, 200)))
            .alignment(Alignment::Center);
        frame.render_widget(help_paragraph, layout[4]);
    }

    /// Renders the date selection screen using a 6-month calendar grid.
    fn render_date_selection(&self, frame: &mut Frame) {
        let area = frame.area();

        // Add a subtle background pattern
        let background = Block::default().style(Style::default().bg(Color::Rgb(16, 16, 28)));
        frame.render_widget(background, area);

        // Create a very subtle pattern using direct buffer access
        for y in (0..area.height).step_by(4) {
            for x in (0..area.width).step_by(8) {
                if (x + y) % 8 == 0 {
                    let pos = (area.x + x, area.y + y);
                    if pos.0 < frame.area().width && pos.1 < frame.area().height {
                        frame.buffer_mut()[pos].set_bg(Color::Rgb(20, 20, 32));
                    }
                }
            }
        }

        // Main layout
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(18),   // Calendar
                Constraint::Length(2), // Legend
                Constraint::Length(1), // Message area
                Constraint::Length(2), // Help text
            ])
            .margin(2) // Added margin
            .split(area);

        // Header
        let header_block = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(Color::Rgb(75, 75, 120)))
            .style(Style::default().bg(Color::Rgb(16, 16, 28)));
        frame.render_widget(header_block, layout[0]);

        let title = Paragraph::new("📅 SELECT DATE")
            .style(
                Style::default()
                    .fg(Color::Rgb(230, 230, 250))
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Rgb(16, 16, 28)),
            )
            .alignment(Alignment::Center);
        frame.render_widget(title, layout[0]);

        // Create calendar grid (3x2 for 6 months)
        let calendar_area = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Calendar ")
            .title_style(
                Style::default()
                    .fg(Color::Rgb(230, 230, 250))
                    .add_modifier(Modifier::BOLD),
            )
            .border_style(Style::default().fg(Color::Rgb(140, 140, 200)))
            .style(Style::default().bg(Color::Rgb(22, 22, 35)));

        let calendar_inner = calendar_area.inner(layout[1]);
        frame.render_widget(calendar_area, layout[1]);

        // Split into two rows
        let calendar_rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(calendar_inner);

        // Split each row into three columns
        let top_row = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(34),
            ])
            .split(calendar_rows[0]);

        let bottom_row = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(34),
            ])
            .split(calendar_rows[1]);

        // Get current date if no date is selected
        let today = time::OffsetDateTime::now_utc().date();
        let selected_date = self.selected_date.unwrap_or(today);

        // Create calendar events store for styling dates
        let mut events = CalendarEventStore::default();

        // Highlight the selected date with yellow background and dark text
        events.add(
            selected_date,
            Style::default()
                .fg(Color::Rgb(20, 20, 50)) // Dark text color for contrast
                .bg(Color::Rgb(250, 250, 110)) // Yellow background
                .add_modifier(Modifier::BOLD),
        );

        // Highlight today with more visible green background
        events.add(
            today,
            Style::default()
                .fg(Color::Rgb(230, 230, 250)) // Light text
                .bg(Color::Rgb(40, 120, 50)) // Green background
                .add_modifier(Modifier::BOLD),
        );

        // Highlight already assigned shifts with more visible background colors
        for (date, shift) in &self.staff_assignments {
            let shift_style = match shift.as_str() {
                "Morning" => Style::default()
                    .fg(Color::Rgb(250, 250, 250)) // White text
                    .bg(Color::Rgb(180, 140, 30)), // Amber background
                "Afternoon" => Style::default()
                    .fg(Color::Rgb(250, 250, 250)) // White text
                    .bg(Color::Rgb(180, 70, 40)), // Orange-red background
                "Night" => Style::default()
                    .fg(Color::Rgb(250, 250, 250)) // White text
                    .bg(Color::Rgb(50, 60, 150)), // Blue background
                _ => Style::default()
                    .fg(Color::Rgb(220, 220, 240))
                    .bg(Color::Rgb(40, 40, 40)),
            };
            events.add(*date, shift_style);
        }

        // Highlight weekends
        let weekend_style = Style::default()
            .fg(Color::Rgb(255, 100, 100))
            .bg(Color::Rgb(35, 25, 25)); // Dark red background

        // Default style for dates
        let default_style = Style::default()
            .fg(Color::Rgb(220, 220, 240))
            .bg(Color::Rgb(26, 26, 36));

        // Common date styling logic for all months
        let apply_date_styles = |date: Date, events: &mut CalendarEventStore| {
            if date.weekday() == Weekday::Saturday || date.weekday() == Weekday::Sunday {
                events.add(date, weekend_style);
            }
        };

        // Calculate dates for 6 months
        let mut month_dates = Vec::new();
        let mut current_date = today;

        // Add current month
        month_dates.push(current_date);

        // Add 5 future months
        for _ in 0..5 {
            // Get next month (roughly by adding 32 days and then getting 1st of that month)
            if let Some(next_month) = current_date.checked_add(time::Duration::days(32)) {
                current_date =
                    time::Date::from_calendar_date(next_month.year(), next_month.month(), 1)
                        .unwrap_or(next_month);
                month_dates.push(current_date);
            }
        }

        // Apply styling to dates in all months (spanning about 6 months)
        let start_date = today.checked_sub(time::Duration::days(60)).unwrap_or(today);
        let end_date = today
            .checked_add(time::Duration::days(180))
            .unwrap_or(today);

        let mut style_date = start_date;
        while style_date <= end_date {
            apply_date_styles(style_date, &mut events);

            if let Some(next_day) = style_date.checked_add(time::Duration::days(1)) {
                style_date = next_day;
            } else {
                break;
            }
        }

        // Render first row of months with month names in block titles
        for (i, month_date) in month_dates.iter().take(3).enumerate() {
            let month_name = format!(" {} {} ", month_date.month(), month_date.year());

            // Highlight the month block that is currently focused (for Tab navigation)
            let border_style = if i == self.focused_month {
                Style::default().fg(Color::Rgb(250, 250, 110))
            } else {
                Style::default().fg(Color::Rgb(140, 140, 200))
            };

            let month = Monthly::new(*month_date, events.clone())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .title(month_name)
                        .title_style(
                            Style::default()
                                .fg(Color::Rgb(230, 230, 250))
                                .add_modifier(Modifier::BOLD),
                        )
                        .border_style(border_style)
                        .style(Style::default().bg(Color::Rgb(26, 26, 36))),
                )
                .show_month_header(
                    Style::default()
                        .fg(Color::Rgb(230, 230, 250))
                        .bg(Color::Rgb(60, 60, 100))
                        .add_modifier(Modifier::BOLD),
                )
                .show_weekdays_header(
                    Style::default()
                        .fg(Color::Rgb(180, 180, 250))
                        .bg(Color::Rgb(40, 40, 60))
                        .add_modifier(Modifier::BOLD),
                )
                .default_style(default_style);

            frame.render_widget(month, top_row[i]);
        }

        // Render second row of months
        for (i, month_date) in month_dates.iter().skip(3).take(3).enumerate() {
            let month_name = format!(" {} {} ", month_date.month(), month_date.year());

            // Highlight the month block that is currently focused (for Tab navigation)
            let border_style = if i + 3 == self.focused_month {
                Style::default().fg(Color::Rgb(250, 250, 110))
            } else {
                Style::default().fg(Color::Rgb(140, 140, 200))
            };

            let month = Monthly::new(*month_date, events.clone())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .title(month_name)
                        .title_style(
                            Style::default()
                                .fg(Color::Rgb(230, 230, 250))
                                .add_modifier(Modifier::BOLD),
                        )
                        .border_style(border_style)
                        .style(Style::default().bg(Color::Rgb(26, 26, 36))),
                )
                .show_month_header(
                    Style::default()
                        .fg(Color::Rgb(230, 230, 250))
                        .bg(Color::Rgb(60, 60, 100))
                        .add_modifier(Modifier::BOLD),
                )
                .show_weekdays_header(
                    Style::default()
                        .fg(Color::Rgb(180, 180, 250))
                        .bg(Color::Rgb(40, 40, 60))
                        .add_modifier(Modifier::BOLD),
                )
                .default_style(default_style);

            frame.render_widget(month, bottom_row[i]);
        }

        // Create a legend for the calendar with the improved styles
        let legend_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ])
            .split(layout[2]);

        let today_legend = Paragraph::new(" ● Today ")
            .style(Style::default().fg(Color::Rgb(40, 120, 50)))
            .alignment(Alignment::Center);

        let selected_legend = Paragraph::new(" ● Selected ")
            .style(Style::default().fg(Color::Rgb(250, 250, 110)))
            .alignment(Alignment::Center);

        let weekend_legend = Paragraph::new(" ● Weekend ")
            .style(Style::default().fg(Color::Rgb(255, 100, 100)))
            .alignment(Alignment::Center);

        let assigned_legend = Paragraph::new(" ● Assigned ")
            .style(Style::default().fg(Color::Rgb(180, 70, 40)))
            .alignment(Alignment::Center);

        frame.render_widget(today_legend, legend_layout[0]);
        frame.render_widget(selected_legend, legend_layout[1]);
        frame.render_widget(weekend_legend, legend_layout[2]);
        frame.render_widget(assigned_legend, legend_layout[3]);

        // Display error message
        if let Some(error) = &self.error_message {
            let error_paragraph = Paragraph::new(error.as_str())
                .style(
                    Style::default()
                        .fg(Color::Rgb(240, 100, 100))
                        .add_modifier(Modifier::BOLD),
                )
                .alignment(Alignment::Center);
            frame.render_widget(error_paragraph, layout[3]);
        }

        // Help text
        let help_text =
            "↑↓←→: Navigate within month | Tab: Switch month | Enter: Select date | Esc: Back";
        let help_paragraph = Paragraph::new(help_text)
            .style(Style::default().fg(Color::Rgb(180, 180, 200)))
            .alignment(Alignment::Center);
        frame.render_widget(help_paragraph, layout[4]);
    }

    /// Renders the shift selection screen with arrow-based navigation.
    fn render_shift_selection(&self, frame: &mut Frame) {
        let area = frame.area();

        // Add a subtle background pattern
        let background = Block::default().style(Style::default().bg(Color::Rgb(16, 16, 28)));
        frame.render_widget(background, area);

        // Create a very subtle pattern using direct buffer access
        for y in (0..area.height).step_by(4) {
            for x in (0..area.width).step_by(8) {
                if (x + y) % 8 == 0 {
                    let pos = (area.x + x, area.y + y);
                    if pos.0 < frame.area().width && pos.1 < frame.area().height {
                        frame.buffer_mut()[pos].set_bg(Color::Rgb(20, 20, 32));
                    }
                }
            }
        }

        // Main layout with padding
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(10),   // Shift options
                Constraint::Length(1), // Message area
                Constraint::Length(2), // Help text
            ])
            .margin(2) // Add more padding on all sides
            .split(area);

        // Header
        let header_block = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(Color::Rgb(75, 75, 120)))
            .style(Style::default().bg(Color::Rgb(16, 16, 28)));
        frame.render_widget(header_block, layout[0]);

        let title = Paragraph::new(" 🕒 SELECT SHIFT")
            .style(
                Style::default()
                    .fg(Color::Rgb(230, 230, 250))
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Rgb(16, 16, 28)),
            )
            .alignment(Alignment::Center);
        frame.render_widget(title, layout[0]);

        // Display staff and date info
        let info_text =
            if let (Some(staff), Some(date)) = (&self.selected_staff, &self.selected_date) {
                let date_str = date
                    .format(&format_description!("[year]-[month]-[day]"))
                    .unwrap_or_else(|_| "Unknown date".to_string());

                format!("Assigning shift to: {} on {}", staff.name, date_str)
            } else {
                "Select a shift".to_string()
            };

        // Create list area with a block
        let list_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(Span::styled(
                format!(" {} ", info_text),
                Style::default()
                    .fg(Color::Rgb(230, 230, 250))
                    .add_modifier(Modifier::BOLD),
            ))
            .border_style(Style::default().fg(Color::Rgb(140, 140, 200)))
            .style(Style::default().bg(Color::Rgb(26, 26, 36)));

        let inner_area = list_block.inner(layout[1]);
        frame.render_widget(list_block, layout[1]);

        // Add header inside the block (with proper padding)
        let header_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Header
                Constraint::Min(1),    // List items
            ])
            .margin(1)
            .split(inner_area);

        let header = Row::new(vec![
            Cell::from("Shift").style(
                Style::default()
                    .fg(Color::Rgb(180, 180, 250))
                    .bg(Color::Rgb(40, 40, 60))
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from("Time").style(
                Style::default()
                    .fg(Color::Rgb(180, 180, 250))
                    .bg(Color::Rgb(40, 40, 60))
                    .add_modifier(Modifier::BOLD),
            ),
        ]);

        let header_table = Table::new(
            vec![header],
            [Constraint::Percentage(50), Constraint::Percentage(50)],
        )
        .block(Block::default());

        frame.render_widget(header_table, header_layout[0]);

        // Shift items with icons
        let shift_items = [
            ("🌅 Morning", "6am - 2pm"),
            ("🌇 Afternoon", "2pm - 10pm"),
            ("🌃 Night", "10pm - 6am"),
        ]
        .iter()
        .map(|(shift, time)| {
            // Apply color based on shift type
            let shift_color = match *shift {
                "🌅 Morning" => Color::Rgb(230, 180, 80), // Sunrise yellow
                "🌇 Afternoon" => Color::Rgb(230, 120, 80), // Sunset orange
                "🌃 Night" => Color::Rgb(100, 130, 200),  // Night blue
                _ => Color::Rgb(220, 220, 240),           // Default
            };

            ListItem::new(Line::from(vec![
                Span::styled(format!("{:<18}", shift), Style::default().fg(shift_color)),
                Span::raw("│ "), // Add divider between columns
                Span::raw(time.to_string()),
            ]))
            .style(Style::default().fg(Color::Rgb(220, 220, 240)))
        })
        .collect::<Vec<ListItem>>();

        // Clone the list state for rendering
        let mut list_state_copy = self.shift_list_state.clone();

        let list = List::new(shift_items)
            .highlight_style(
                Style::default()
                    .bg(Color::Rgb(60, 60, 100))
                    .fg(Color::Rgb(250, 250, 110))
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("► ");

        frame.render_stateful_widget(list, header_layout[1], &mut list_state_copy);

        // Display error/success message
        if let Some(error) = &self.error_message {
            let error_paragraph = Paragraph::new(error.as_str())
                .style(
                    Style::default()
                        .fg(Color::Rgb(240, 100, 100))
                        .add_modifier(Modifier::BOLD),
                )
                .alignment(Alignment::Center);
            frame.render_widget(error_paragraph, layout[2]);
        } else if let Some(success) = &self.success_message {
            let success_paragraph = Paragraph::new(success.as_str())
                .style(
                    Style::default()
                        .fg(Color::Rgb(140, 219, 140))
                        .add_modifier(Modifier::BOLD),
                )
                .alignment(Alignment::Center);
            frame.render_widget(success_paragraph, layout[2]);
        }

        // Help text
        let help_text = "↑/↓: Navigate | Enter: Select shift | Esc: Back";
        let help_paragraph = Paragraph::new(help_text)
            .style(Style::default().fg(Color::Rgb(180, 180, 200)))
            .alignment(Alignment::Center);
        frame.render_widget(help_paragraph, layout[3]);
    }

    /// Renders the assigned shifts view for a staff member.
    fn render_assigned_shifts(&self, frame: &mut Frame) {
        let area = frame.area();

        // Add a subtle background pattern
        let background = Block::default().style(Style::default().bg(Color::Rgb(16, 16, 28)));
        frame.render_widget(background, area);

        // Create a very subtle pattern using direct buffer access
        for y in (0..area.height).step_by(4) {
            for x in (0..area.width).step_by(8) {
                if (x + y) % 8 == 0 {
                    let pos = (area.x + x, area.y + y);
                    if pos.0 < frame.area().width && pos.1 < frame.area().height {
                        frame.buffer_mut()[pos].set_bg(Color::Rgb(20, 20, 32));
                    }
                }
            }
        }

        // Main layout
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(10),   // Assignments table
                Constraint::Length(1), // Message area
                Constraint::Length(2), // Help text
            ])
            .margin(2)
            .split(area);

        // Header
        let header_block = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(Color::Rgb(75, 75, 120)))
            .style(Style::default().bg(Color::Rgb(16, 16, 28)));
        frame.render_widget(header_block, layout[0]);

        let title = if let Some(staff) = &self.selected_staff {
            Paragraph::new(format!(
                " 📋 SHIFTS ASSIGNED TO {} ",
                staff.name.to_uppercase()
            ))
            .style(
                Style::default()
                    .fg(Color::Rgb(230, 230, 250))
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Rgb(16, 16, 28)),
            )
            .alignment(Alignment::Center)
        } else {
            Paragraph::new(" 📋 ASSIGNED SHIFTS ")
                .style(
                    Style::default()
                        .fg(Color::Rgb(230, 230, 250))
                        .add_modifier(Modifier::BOLD)
                        .bg(Color::Rgb(16, 16, 28)),
                )
                .alignment(Alignment::Center)
        };
        frame.render_widget(title, layout[0]);

        // Table for assignments
        if let Some(_staff) = &self.selected_staff {
            let table_block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" Assigned Shifts ")
                .title_style(
                    Style::default()
                        .fg(Color::Rgb(230, 230, 250))
                        .add_modifier(Modifier::BOLD),
                )
                .border_style(Style::default().fg(Color::Rgb(140, 140, 200)))
                .style(Style::default().bg(Color::Rgb(26, 26, 36)));

            let normal_style = Style::default()
                .bg(Color::Rgb(26, 26, 36))
                .fg(Color::Rgb(220, 220, 240));

            // Create table rows with enhanced formatting
            let rows = if self.staff_assignments.is_empty() {
                vec![Row::new(vec![
                    Cell::from(""),
                    Cell::from("No shifts assigned")
                        .style(Style::default().fg(Color::Rgb(180, 180, 200))),
                    Cell::from(""),
                ])]
            } else {
                self.staff_assignments
                    .iter()
                    .map(|(date, shift)| {
                        let date_str = date
                            .format(&format_description!("[month repr:short] [day], [year]"))
                            .unwrap_or_else(|_| "Unknown".to_string());

                        let (shift_style, shift_icon, time_range) = match shift.as_str() {
                            "Morning" => (
                                Style::default()
                                    .fg(Color::Rgb(230, 180, 80))
                                    .bg(Color::Rgb(30, 30, 30)),
                                "🌅 Morning",
                                "6am - 2pm",
                            ),
                            "Afternoon" => (
                                Style::default()
                                    .fg(Color::Rgb(230, 120, 80))
                                    .bg(Color::Rgb(30, 30, 30)),
                                "🌇 Afternoon",
                                "2pm - 10pm",
                            ),
                            "Night" => (
                                Style::default()
                                    .fg(Color::Rgb(100, 130, 200))
                                    .bg(Color::Rgb(30, 30, 30)),
                                "🌃 Night",
                                "10pm - 6am",
                            ),
                            _ => (
                                normal_style,
                                shift.as_ref(), // Fix: use as_ref() instead of as_str()
                                "",
                            ),
                        };

                        Row::new(vec![
                            Cell::from(date_str).style(normal_style),
                            Cell::from(shift_icon).style(shift_style),
                            Cell::from(time_range).style(normal_style),
                        ])
                    })
                    .collect()
            };

            let table = Table::new(
                rows,
                [
                    Constraint::Length(16), // Date
                    Constraint::Length(14), // Shift
                    Constraint::Min(10),    // Time
                ],
            )
            .header(
                Row::new(vec![
                    Cell::from("Date").style(Style::default().add_modifier(Modifier::BOLD)),
                    Cell::from("Shift").style(Style::default().add_modifier(Modifier::BOLD)),
                    Cell::from("Time").style(Style::default().add_modifier(Modifier::BOLD)),
                ])
                .style(
                    Style::default()
                        .bg(Color::Rgb(80, 60, 130))
                        .fg(Color::Rgb(180, 180, 250)),
                )
                .height(1),
            )
            .block(table_block);

            frame.render_widget(table, layout[1]);
        } else {
            // No staff selected
            let message = Paragraph::new("No staff member selected")
                .style(Style::default().fg(Color::Rgb(180, 180, 200)))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .style(Style::default().bg(Color::Rgb(26, 26, 36))),
                );
            frame.render_widget(message, layout[1]);
        }

        // Display error message if any
        if let Some(error) = &self.error_message {
            let error_paragraph = Paragraph::new(error.as_str())
                .style(
                    Style::default()
                        .fg(Color::Rgb(240, 100, 100))
                        .add_modifier(Modifier::BOLD),
                )
                .alignment(Alignment::Center);
            frame.render_widget(error_paragraph, layout[2]);
        }

        // Help text
        let help_text = "Esc: Back to staff selection";
        let help_paragraph = Paragraph::new(help_text)
            .style(Style::default().fg(Color::Rgb(180, 180, 200)))
            .alignment(Alignment::Center);
        frame.render_widget(help_paragraph, layout[3]);
    }

    /// Renders the confirmation dialog.
    fn render_confirmation_dialog(&self, frame: &mut Frame) {
        let area = frame.area();
        let dialog_width = 50;
        let dialog_height = 10;

        let dialog_area = Rect::new(
            (area.width.saturating_sub(dialog_width)) / 2,
            (area.height.saturating_sub(dialog_height)) / 2,
            dialog_width,
            dialog_height,
        );

        frame.render_widget(Clear, dialog_area);

        let dialog_block = Block::default()
            .title(" Confirm Assignment ")
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

        // Dialog content layout
        let inner_area = dialog_block.inner(dialog_area);
        let content_layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(4), // Increased height for details
                Constraint::Length(2),
            ])
            .split(inner_area);

        // Confirmation message with details
        let staff_name = self
            .selected_staff
            .as_ref()
            .map(|s| s.name.clone())
            .unwrap_or("Unknown".to_string());

        let date_str = self
            .selected_date
            .map(|d| {
                d.format(&format_description!("[year]-[month]-[day]"))
                    .unwrap_or_else(|_| "Unknown".to_string())
            })
            .unwrap_or_else(|| "Unknown".to_string());

        let (shift_str, shift_time) = match self.selected_shift {
            Some(Shift::Morning) => ("🌅 Morning", "6am - 2pm"),
            Some(Shift::Afternoon) => ("🌇 Afternoon", "2pm - 10pm"),
            Some(Shift::Night) => ("🌃 Night", "10pm - 6am"),
            None => ("Unknown", ""),
        };

        let message_text = format!(
            "Assign {} ({}) shift to\n{} on {}?\n(y/n)",
            shift_str, shift_time, staff_name, date_str
        );

        let message = Paragraph::new(message_text)
            .style(Style::default().fg(Color::Rgb(220, 220, 240)))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        frame.render_widget(message, content_layout[0]);

        // Yes/No options
        let choices = Paragraph::new("Y: Yes | N: No")
            .style(Style::default().fg(Color::Rgb(220, 220, 240)))
            .alignment(Alignment::Center);
        frame.render_widget(choices, content_layout[1]);
    }
}
