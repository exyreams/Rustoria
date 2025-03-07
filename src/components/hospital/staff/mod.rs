//! Staff management module within the Hospital application.
//!
//! Provides functionality for managing staff members: adding, listing,
//! updating, and deleting.  Uses submodules for each action.

use self::add::AddStaff;
use self::assign::AssignStaff;
use self::delete::DeleteStaff;
use self::list::ListStaff;
use self::update::UpdateStaff;
use crate::app::SelectedApp;
use crate::components::Component;
use crate::tui::Frame;
use anyhow::Result;
use crossterm::event::KeyEvent;

/// Module for adding staff members.
pub mod add;
/// Module for assigning shifts to staff members.
pub mod assign;
/// Module for deleting staff members.
pub mod delete;
/// Module for listing staff members.
pub mod list;
/// Module for updating staff member information.
pub mod update;

/// Actions that can be performed within the staff management component.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StaffAction {
    /// Return to the home screen.
    BackToHome,
    /// Return to the staff list.
    #[allow(dead_code)]
    BackToList,
}

/// Different states of the staff management component.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StaffState {
    /// Adding a new staff member.
    AddStaff,
    /// Listing all staff.
    ListStaff,
    /// Deleting a staff member.
    DeleteStaff,
    /// Updating staff information.
    UpdateStaff,
    /// Assigning a shift to a staff member.
    AssignStaff,
}

/// Manages staff-related functionalities within the hospital application.
pub struct Staff {
    /// Component for adding staff.
    add_staff: AddStaff,
    /// Component for listing staff.
    list_staff: ListStaff,
    /// Wrapped in Option because it's only active during deletion.
    pub delete_staff: Option<DeleteStaff>,
    /// Wrapped in Option because it's only active during updates.
    pub update_staff: Option<UpdateStaff>,
    /// Component for assigning shifts
    pub assign_staff: Option<AssignStaff>,
    /// Current state of the staff component.
    pub state: StaffState,
}

impl Staff {
    /// Creates a new `Staff` component.
    pub fn new() -> Self {
        // Initialize the ListStaff component
        let mut list_staff = ListStaff::new();
        list_staff.fetch_staff().expect("Failed to fetch staff"); // Fetch on creation

        Self {
            add_staff: AddStaff::new(),
            list_staff,
            delete_staff: None,
            update_staff: None,
            assign_staff: None,           // Initialize assign_staff
            state: StaffState::ListStaff, // Start in ListStaff state
        }
    }

    /// Initializes or refreshes the staff list (if in ListStaff state).
    pub fn initialize_list(&mut self) -> Result<()> {
        // Only fetch if in ListStaff
        if self.state == StaffState::ListStaff {
            self.list_staff.fetch_staff()?;
        }
        Ok(())
    }
}

impl Component for Staff {
    /// Handles user input events.
    fn handle_input(&mut self, event: KeyEvent) -> Result<Option<SelectedApp>> {
        // Handle input based on current state
        match self.state {
            StaffState::AddStaff => {
                if let Some(selected_app) = self.add_staff.handle_input(event)? {
                    match selected_app {
                        SelectedApp::None => {
                            // Return to ListStaff after adding
                            self.state = StaffState::ListStaff;
                            self.initialize_list()?; // Refresh list
                            return Ok(Some(SelectedApp::None));
                        }
                        _ => {}
                    }
                }
            }
            StaffState::ListStaff => {
                if let Some(selected_app) = self.list_staff.handle_input(event)? {
                    match selected_app {
                        SelectedApp::None => {
                            return Ok(Some(SelectedApp::None));
                        }
                        _ => {}
                    }
                }
            }
            StaffState::DeleteStaff => {
                if let Some(delete_staff) = &mut self.delete_staff {
                    if let Some(selected_app) = delete_staff.handle_input(event)? {
                        match selected_app {
                            SelectedApp::None => {
                                self.state = StaffState::ListStaff;
                                self.delete_staff = None; // Remove component
                                self.initialize_list()?; // Refresh list
                                return Ok(Some(SelectedApp::None));
                            }
                            _ => {}
                        }
                    }
                }
            }
            StaffState::UpdateStaff => {
                if let Some(update_staff) = &mut self.update_staff {
                    if let Some(selected_app) = update_staff.handle_input(event)? {
                        match selected_app {
                            SelectedApp::None => {
                                self.state = StaffState::ListStaff;
                                self.update_staff = None; // Remove component
                                self.initialize_list()?; // Refresh list
                                return Ok(Some(SelectedApp::None));
                            }
                            _ => {}
                        }
                    }
                }
            }
            StaffState::AssignStaff => {
                // Handle input for AssignStaff
                if let Some(assign_staff) = &mut self.assign_staff {
                    if let Some(selected_app) = assign_staff.handle_input(event)? {
                        match selected_app {
                            SelectedApp::None => {
                                self.state = StaffState::ListStaff;
                                self.assign_staff = None; // Remove component
                                self.initialize_list()?; // Refresh the list.
                                return Ok(Some(SelectedApp::None));
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        Ok(None)
    }

    /// Renders the staff component to the terminal.
    fn render(&self, frame: &mut Frame) {
        // Render based on current state
        match self.state {
            StaffState::AddStaff => self.add_staff.render(frame),
            StaffState::ListStaff => self.list_staff.render(frame),
            StaffState::DeleteStaff => {
                if let Some(delete_staff) = &self.delete_staff {
                    delete_staff.render(frame);
                }
            }
            StaffState::UpdateStaff => {
                if let Some(update_staff) = &self.update_staff {
                    update_staff.render(frame);
                }
            }
            StaffState::AssignStaff => {
                // Render AssignStaff
                if let Some(assign_staff) = &self.assign_staff {
                    assign_staff.render(frame);
                }
            }
        }
    }
}

impl Default for Staff {
    fn default() -> Self {
        Self::new()
    }
}
