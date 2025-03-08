//! Hospital management module.
//!
//! This module serves as the central hub for managing hospital-related data,
//! encompassing patients, staff, and records. It orchestrates the interactions
//! between different components, encapsulating their respective states and logic.
//! The module exposes the `HospitalApp` struct, which represents the main application.

use self::patients::PatientsState;
use self::records::Records;
use self::records::RecordsState;
use self::staff::Staff;
use self::staff::StaffState;
use crate::components::Component;
use crate::tui::Frame;
use anyhow::Result;
use crossterm::event::KeyEvent;

pub mod patients;
pub mod records;
pub mod staff;

/// Enum representing the different states of the Hospital application.
///
/// `HospitalState` defines the top-level states that the `HospitalApp` can be in,
/// dictating which sub-component is currently active and rendered. Each variant
/// corresponds to a specific area of the application, such as managing patients or staff.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HospitalState {
    /// Represents the state where the patient management component is active.
    Patients,
    /// Represents the state where the staff management component is active.
    Staff,
    Records,
}

/// Struct representing the main Hospital application.
///
/// `HospitalApp` is the top-level struct that manages the overall state and
/// behavior of the hospital application. It holds instances of the `Patients`,
/// `Staff`, and `Records` components, delegating input handling and rendering
/// to the currently active component based on the `HospitalState`.
pub struct HospitalApp {
    /// The current state of the hospital application.
    pub state: HospitalState,
    /// The patient management component.
    pub patients: patients::Patients,
    /// The records management component.
    pub records: Records,
    /// The staff management component.
    pub staff: Staff,
}

impl HospitalApp {
    /// Creates a new instance of the `HospitalApp`.
    ///
    /// This constructor initializes the `HospitalApp` by creating instances of
    /// the `Patients`, `Staff`, and `Records` components and initializing their lists.
    /// It sets the initial state to `HospitalState::Patients`.
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
        let mut staff = Staff::new();

        // Initialize the staff list
        staff
            .initialize_list()
            .expect("Failed to initialize staff list");

        // Initialize the Records component
        let mut records = Records::new();
        records
            .initialize_list()
            .expect("Failed to initialize records list");

        Self {
            state: HospitalState::Patients, // Default state can now be either Patients or Staff
            patients,
            staff,
            records,
        }
    }

    /// Sets the state of the patients component.
    ///
    /// This method updates the internal state of the `patients` component,
    /// determining which view or mode the patient management section is in. If the new state is `PatientsState::ListPatients`,
    /// the patient list is re-initialized to reflect any changes.
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
    /// This method allows switching between different sections of the application,
    /// such as `Patients`, `Staff`, or `Records`. Changing the state determines
    /// which component will handle input and be rendered to the screen.
    ///
    /// # Arguments
    ///
    /// * `new_state` - The new `HospitalState` to set.
    pub fn set_state(&mut self, new_state: HospitalState) {
        self.state = new_state;
    }

    /// Sets the state of the staff component.
    ///
    /// This method updates the internal state of the `staff` component,
    /// similar to `set_patients_state`. If the new state is `StaffState::ListStaff`,
    /// the staff list is re-initialized.
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

    /// Sets the state of the records component.
    ///
    /// This method updates the internal state of the `records` component.
    /// If the new state is `RecordsState::RetrieveRecords`,
    /// the records list is re-initialized.
    ///
    /// # Arguments
    ///
    /// * `state` - The new `RecordsState` to set.
    pub fn set_records_state(&mut self, state: RecordsState) {
        self.records.state = state;
        if state == RecordsState::RetrieveRecords {
            if let Err(e) = self.records.initialize_list() {
                eprintln!("Error initializing records list: {}", e);
            }
        }
    }
}

impl Component for HospitalApp {
    /// Handles user input events.
    ///
    /// This function receives key events from the user interface and routes them
    /// to the currently active component based on the `HospitalState`. It returns
    /// a `Result` indicating whether the input was handled successfully and whether
    /// an application selection was made.
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
            HospitalState::Records => {
                if let Some(action) = self.records.handle_input(event)? {
                    return Ok(Some(action));
                }
            }
        }
        Ok(None)
    }

    /// Renders the hospital application to the terminal.
    ///
    /// This function is responsible for drawing the user interface of the active
    /// component onto the terminal screen. It receives a `Frame` object, which
    /// represents the drawing surface, and delegates the rendering to the appropriate
    /// component based on the current `HospitalState`.
    ///
    /// # Arguments
    ///
    /// * `frame` - The frame to render to.
    fn render(&self, frame: &mut Frame) {
        match self.state {
            HospitalState::Patients => self.patients.render(frame),
            HospitalState::Staff => self.staff.render(frame), // Render Staff component
            HospitalState::Records => self.records.render(frame),
        }
    }
}
