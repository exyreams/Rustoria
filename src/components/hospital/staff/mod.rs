//! Staff management module within the Hospital application.
//!
//! This module provides functionality for managing staff members, including adding and listing staff.
//! It utilizes submodules for different actions and manages the state of the staff component.

use self::add::AddStaff;
use self::list::ListStaff;
use crate::app::SelectedApp;
use crate::components::Component;
use crate::tui::Frame;
use anyhow::Result;
use crossterm::event::KeyEvent;

pub mod add;
pub mod list;

/// Represents the actions that can be performed within the staff management component.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StaffAction {
    /// Return to the home screen.
    BackToHome,
    /// Return to the staff list.
    #[allow(dead_code)]
    BackToList,
}

/// Represents the different states of the staff management component.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StaffState {
    /// State for adding a new staff member.
    AddStaff,
    /// State for listing all staff.
    ListStaff,
    // Future states: UpdateStaff, DeleteStaff
}

/// Manages the staff-related functionalities within the hospital application.
///
/// This struct handles the different states of the staff management component
/// and delegates actions to the appropriate sub-components.
pub struct Staff {
    /// Component for adding new staff members.
    add_staff: AddStaff,
    /// Component for listing staff members.
    list_staff: ListStaff,
    /// The current state of the staff component.
    pub state: StaffState,
}

impl Staff {
    /// Creates a new instance of the `Staff` component.
    ///
    /// Initializes the `AddStaff` and `ListStaff` components and sets the initial state to `ListStaff`.
    /// Fetches the staff list on creation to ensure it's up-to-date.
    pub fn new() -> Self {
        // Initialize the ListStaff component
        let mut list_staff = ListStaff::new();
        // Fetch staff data immediately
        list_staff.fetch_staff().expect("Failed to fetch staff"); // Fetch staff on creation

        Self {
            add_staff: AddStaff::new(),
            list_staff,
            state: StaffState::ListStaff, // Start in ListStaff state
        }
    }

    /// Initializes or refreshes the staff list if the current state is `ListStaff`.
    ///
    /// This function is called to ensure that the staff list is up-to-date.
    ///
    /// # Errors
    ///
    /// Returns an error if fetching the staff fails.
    pub fn initialize_list(&mut self) -> Result<()> {
        // Only fetch staff if we are in the ListStaff state
        if self.state == StaffState::ListStaff {
            self.list_staff.fetch_staff()?;
        }
        Ok(())
    }
}

impl Component for Staff {
    /// Handles user input events.
    ///
    /// This function processes `KeyEvent`s and performs actions based on the current `StaffState`.
    /// It delegates input handling to the active sub-component (AddStaff or ListStaff).
    ///
    /// # Arguments
    ///
    /// * `event` - The key event to handle.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Some(SelectedApp))` if an application selection is made, `Ok(None)` otherwise, or an `Err` if an error occurs.
    fn handle_input(&mut self, event: KeyEvent) -> Result<Option<SelectedApp>> {
        // Handle input based on the current state
        match self.state {
            StaffState::AddStaff => {
                // Delegate input handling to the AddStaff component
                if let Some(selected_app) = self.add_staff.handle_input(event)? {
                    // Handle the result from AddStaff
                    match selected_app {
                        SelectedApp::None => {
                            // Return to the ListStaff view after adding
                            self.state = StaffState::ListStaff;
                            self.initialize_list()?; // Refresh list
                            return Ok(Some(SelectedApp::None));
                        }
                        _ => {}
                    }
                }
            }
            StaffState::ListStaff => {
                // Delegate input handling to the ListStaff component.
                if let Some(selected_app) = self.list_staff.handle_input(event)? {
                    // Handle the result from ListStaff, if any.  This is
                    // where we process StaffAction::BackToHome, for example.
                    match selected_app {
                        SelectedApp::None => {
                            return Ok(Some(SelectedApp::None));
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(None)
    }

    /// Renders the staff component to the terminal.
    ///
    /// This function calls the `render` method of the active sub-component
    /// (AddStaff or ListStaff) based on the current `StaffState`.
    ///
    /// # Arguments
    ///
    /// * `frame` - The frame to render to.
    fn render(&self, frame: &mut Frame) {
        // Render the appropriate component based on the current state
        match self.state {
            StaffState::AddStaff => self.add_staff.render(frame),
            StaffState::ListStaff => self.list_staff.render(frame),
        }
    }
}
