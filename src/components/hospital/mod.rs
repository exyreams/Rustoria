//! Hospital management module.
//!
//! This module represents the main application for managing hospital data,
//! including patients and staff. It orchestrates the different components
//! and their states.

use self::patients::PatientsState;
use self::staff::Staff;
use self::staff::StaffState; // Import StaffState
use crate::components::Component;
use crate::tui::Frame;
use anyhow::Result;
use crossterm::event::KeyEvent;

pub mod patients;
pub mod staff;

/// Enum representing the different states of the Hospital application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HospitalState {
    /// Represents the state where the patient management component is active.
    Patients,
    /// Represents the state where the staff management component is active.
    Staff,
}

/// Struct representing the main Hospital application.
///
/// This struct manages the overall state of the hospital application
/// and delegates functionality to its sub-components (patients and staff).
pub struct HospitalApp {
    /// The current state of the hospital application.
    pub state: HospitalState,
    /// The patient management component.
    pub patients: patients::Patients,
    /// The staff management component.
    pub staff: Staff,
}

impl HospitalApp {
    /// Creates a new instance of the `HospitalApp`.
    ///
    /// Initializes the patients and staff components and sets the initial state.
    ///
    /// # Returns
    ///
    /// A new instance of `HospitalApp`.
    pub fn new() -> Self {
        // Initialize the Patients component
        let mut patients = patients::Patients::new();
        // Initialize the patient list
        patients
            .initialize_list()
            .expect("Failed to initialize patient list");

        // Initialize the Staff component
        let mut staff = Staff::new(); // Initialize Staff
                                      // Initialize the staff list
        staff
            .initialize_list()
            .expect("Failed to initialize staff list");

        Self {
            state: HospitalState::Patients, // Default state can now be either Patients or Staff
            patients,
            staff,
        }
    }

    /// Sets the state of the patients component.
    ///
    /// This method updates the internal state of the `patients` component
    /// and re-initializes the patient list if necessary.
    ///
    /// # Arguments
    ///
    /// * `state` - The new `PatientsState` to set.
    pub fn set_patients_state(&mut self, state: PatientsState) {
        self.patients.state = state;
        if state == PatientsState::ListPatients {
            if let Err(e) = self.patients.initialize_list() {
                eprintln!("Error initializing patient list: {}", e);
            }
        }
    }

    /// Sets the current hospital application state.
    ///
    /// # Arguments
    ///
    /// * `new_state` - The new `HospitalState` to set.
    pub fn set_state(&mut self, new_state: HospitalState) {
        self.state = new_state;
    }

    /// Sets the state of the staff component.
    ///
    /// This method updates the internal state of the `staff` component
    /// and re-initializes the staff list if necessary.
    ///
    /// # Arguments
    ///
    /// * `state` - The new `StaffState` to set.
    pub fn set_staff_state(&mut self, state: StaffState) {
        self.staff.state = state;
        if state == StaffState::ListStaff {
            if let Err(e) = self.staff.initialize_list() {
                eprintln!("Error initializing staff list: {}", e);
            }
        }
    }
}

impl Component for HospitalApp {
    /// Handles user input events.
    ///
    /// This function processes `KeyEvent`s and delegates them to the
    /// currently active sub-component (`patients` or `staff`).
    ///
    /// # Arguments
    ///
    /// * `event` - The key event to handle.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Some(crate::app::SelectedApp))` if an application selection is made,
    /// `Ok(None)` otherwise, or an `Err` if an error occurs.
    fn handle_input(&mut self, event: KeyEvent) -> Result<Option<crate::app::SelectedApp>> {
        match self.state {
            HospitalState::Patients => {
                // Delegate input handling to the Patients component
                if let Some(action) = self.patients.handle_input(event)? {
                    return Ok(Some(action));
                }
            }
            HospitalState::Staff => {
                // Delegate to the Staff component
                if let Some(action) = self.staff.handle_input(event)? {
                    return Ok(Some(action));
                }
            }
        }
        Ok(None)
    }

    /// Renders the hospital application to the terminal.
    ///
    /// This function renders the currently active sub-component
    /// (`patients` or `staff`) to the provided frame.
    ///
    /// # Arguments
    ///
    /// * `frame` - The frame to render to.
    fn render(&self, frame: &mut Frame) {
        match self.state {
            HospitalState::Patients => self.patients.render(frame),
            HospitalState::Staff => self.staff.render(frame), // Render Staff component
        }
    }
}
