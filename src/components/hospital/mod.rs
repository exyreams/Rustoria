//! Hospital management module.
//!
//! This module serves as the main application module for the hospital,
//! providing top-level navigation and management of the hospital's
//! functionalities, specifically the patient management system.

use self::patients::PatientsState;
use self::staff::Staff;
use crate::components::Component;
use crate::tui::Frame;
use anyhow::Result;
use crossterm::event::KeyEvent;

pub mod patients;
pub mod staff;

/// Enum representing the different states of the Hospital application.
///
/// This enum defines the different views or sections within the hospital
/// application, such as the Patients section.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HospitalState {
    /// Represents the Patients section of the hospital application.
    Patients,
    /// Represents the Staff Scheduling section of the hospital application.
    Staff,
}

/// Struct representing the main Hospital application.
///
/// This struct manages the overall state and components of the hospital
/// application, providing entry points for handling input and rendering
/// the UI.
pub struct HospitalApp {
    /// The current state of the hospital application.
    pub state: HospitalState,
    /// The patients component, which handles patient-related functionalities.
    pub patients: patients::Patients,
    /// The patients component, which handles patient-related functionalities.
    pub staff: Staff,
}

impl HospitalApp {
    /// Creates a new `HospitalApp` instance.
    ///
    /// Initializes the `patients` component and fetches the initial patient list.
    ///
    /// # Returns
    ///
    /// A new `HospitalApp` instance.
    pub fn new() -> Self {
        // Create a new Patients component
        let mut patients = patients::Patients::new();
        // Initialize the patient list, panicking if it fails
        patients
            .initialize_list()
            .expect("Failed to initialize patient list");

        Self {
            // Set the initial state to Patients
            state: HospitalState::Patients,
            // Store the initialized Patients component
            patients,
            staff: Staff::new(),
        }
    }
    /// Sets the state of the `Patients` component.
    ///
    /// This method allows external components to change the `PatientsState`
    /// within the `HospitalApp`. It also handles the initialization of
    /// patient data when transitioning to the `ListPatients` state.
    ///
    /// # Arguments
    ///
    /// * `state`: The new `PatientsState` to set.
    pub fn set_patients_state(&mut self, state: PatientsState) {
        // Set the new state of the Patients component
        self.patients.state = state;
        // Initialize data if switching to ListPatients
        if state == PatientsState::ListPatients {
            // Attempt to initialize the patient list
            if let Err(e) = self.patients.initialize_list() {
                // Print an error message to stderr if initialization fails
                eprintln!("Error initializing patient list: {}", e);
            }
        }
    }

    /// Sets the current hospital application state.
    pub fn set_state(&mut self, new_state: HospitalState) {
        self.state = new_state;
    }
}

impl Component for HospitalApp {
    /// Handles input events for the Hospital application.
    ///
    /// This method delegates input handling to the appropriate component based
    /// on the current application state. Currently, it only handles input
    /// for the `Patients` component.
    ///
    /// # Arguments
    ///
    /// * `event`: The key event to handle.
    ///
    /// # Returns
    ///
    /// Returns an `Option<crate::app::SelectedApp>` indicating if a top-level
    /// application action should be taken.
    fn handle_input(&mut self, event: KeyEvent) -> Result<Option<crate::app::SelectedApp>> {
        match self.state {
            // Handle input when in the Patients state
            HospitalState::Patients => {
                // Delegate input handling to the Patients component
                if let Some(action) = self.patients.handle_input(event)? {
                    // Return any action returned by the Patients component
                    return Ok(Some(action)); // Directly return any action from Patients
                }
            }
            HospitalState::Staff => {
                // Delegate input handling to the Staff component
                if let Some(action) = self.staff.handle_input(event)? {
                    // Return any action returned by the Staff component
                    return Ok(Some(action)); // Directly return any action from Staff
                }
            }
        }
        // If no action is taken, return None
        Ok(None)
    }

    /// Renders the Hospital application UI.
    ///
    /// This method renders the appropriate component based on the current
    /// application state. Currently, it only renders the `Patients` component.
    ///
    /// # Arguments
    ///
    /// * `frame`: The frame to render the UI to.
    fn render(&self, frame: &mut Frame) {
        match self.state {
            // Render the Patients component when in the Patients state
            HospitalState::Patients => self.patients.render(frame),
            HospitalState::Staff => self.staff.render(frame),
        }
    }
}
