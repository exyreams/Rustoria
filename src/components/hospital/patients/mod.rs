//! Patient management module within the Hospital application.
//!
//! This module provides functionalities to manage patients, including adding,
//! listing, and deleting patient records. It interacts with other modules
//! within the `components` directory to handle UI components and actions.

use crate::app::SelectedApp;
use crate::components::hospital::patients::add::AddPatient;
use crate::components::hospital::patients::delete::DeletePatient;
use crate::components::hospital::patients::list::ListPatients;
use crate::components::Component;
use crate::tui::Frame;
use anyhow::Result;
use crossterm::event::KeyEvent;

pub mod add;
pub mod delete;
pub mod list;

/// Enum representing actions that can be performed within the patient component.
///
/// These actions dictate the flow of control, such as navigating back to the
/// home screen or back to the patient list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum PatientAction {
    /// Navigate back to the main home screen.
    BackToHome,
    /// Navigate back to the patient list view.
    BackToList,
}

/// Enum representing the different states of the Patients component.
///
/// This enum is used to manage the UI state, determining which sub-component
/// (AddPatient, ListPatients, or DeletePatient) is currently active and rendered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatientsState {
    /// State for adding a new patient.
    AddPatient,
    /// State for listing existing patients.
    ListPatients,
    /// State for deleting a patient.
    DeletePatient,
}

/// Struct representing the Patients component.
///
/// This struct holds the sub-components for adding, listing, and deleting patients,
/// as well as the current state of the component.
pub struct Patients {
    /// Component for adding a new patient.
    pub add_patient: AddPatient,
    /// Component for listing existing patients.
    pub list_patients: ListPatients,
    /// Component for deleting a patient. This is an Option because it's only
    /// instantiated when a patient is selected for deletion.
    pub delete_patient: Option<DeletePatient>,
    /// Current state of the Patients component.
    pub state: PatientsState,
}

impl Patients {
    /// Creates a new `Patients` component.
    ///
    /// Initializes the sub-components and sets the initial state to `ListPatients`.
    pub fn new() -> Self {
        Self {
            add_patient: AddPatient::new(),
            list_patients: ListPatients::new(),
            delete_patient: None,
            state: PatientsState::ListPatients,
        }
    }

    /// Initializes the patient list.
    ///
    /// Fetches the list of patients if the current state is `ListPatients`.
    ///
    /// # Errors
    ///
    /// Returns an error if fetching the patients fails.
    pub fn initialize_list(&mut self) -> Result<()> {
        if self.state == PatientsState::ListPatients {
            self.list_patients.fetch_patients()?;
        }
        Ok(())
    }
}

impl Component for Patients {
    /// Handles input events and returns a potential `SelectedApp` action.
    ///
    /// This method processes key events based on the current `PatientsState`.
    /// It delegates input handling to the appropriate sub-component
    /// (AddPatient, ListPatients, or DeletePatient) and updates the state
    /// or returns an action as needed.
    ///
    /// # Arguments
    ///
    /// * `event`: The key event to handle.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Some(SelectedApp))` if a navigation action should be taken,
    /// `Ok(None)` if no navigation action is needed, or an error if input
    /// handling fails.
    fn handle_input(&mut self, event: KeyEvent) -> Result<Option<SelectedApp>> {
        match self.state {
            // Handle input for AddPatient state
            PatientsState::AddPatient => {
                // If add_patient handles input and returns an action
                if let Some(action) = self.add_patient.handle_input(event)? {
                    match action {
                        // If action is BackToHome
                        PatientAction::BackToHome => {
                            // Change state to ListPatients
                            self.state = PatientsState::ListPatients;
                            // Initialize the list
                            self.initialize_list()?;
                            // Return None to indicate staying within the app, but no specific selection
                            return Ok(Some(SelectedApp::None));
                        }
                        // Handle other potential actions (currently none)
                        _ => {}
                    }
                }
            }
            // Handle input for ListPatients state
            PatientsState::ListPatients => {
                // If list_patients handles input and returns an action
                if let Some(action) = self.list_patients.handle_input(event)? {
                    match action {
                        // If action is BackToHome
                        PatientAction::BackToHome => return Ok(Some(SelectedApp::None)),
                        // Handle other potential actions (currently none, no DeletePatient action)
                        _ => {} // No DeletePatient action
                    }
                }
            }
            // Handle input for DeletePatient state
            PatientsState::DeletePatient => {
                // If delete_patient exists
                if let Some(delete_patient) = &mut self.delete_patient {
                    // If delete_patient handles input and returns a selected app
                    if let Some(selected_app) = delete_patient.handle_input(event)? {
                        match selected_app {
                            // If selected app is None (Back action)
                            SelectedApp::None => {
                                // Change state back to ListPatients
                                self.state = PatientsState::ListPatients;
                                // Reset delete_patient to None
                                self.delete_patient = None;
                                // Re-fetch the patient list
                                self.initialize_list()?;
                                // Return None to indicate staying within the app, but no specific selection
                                return Ok(Some(SelectedApp::None));
                            }
                            // Handle other potential selected apps (currently none)
                            _ => {}
                        }
                    }
                }
            }
        }
        Ok(None)
    }

    /// Renders the Patients component and its sub-components.
    ///
    /// The rendering is delegated to the appropriate sub-component
    /// (AddPatient, ListPatients, or DeletePatient) based on the current `PatientsState`.
    ///
    /// # Arguments
    ///
    /// * `frame`: The frame buffer to render to.
    fn render(&self, frame: &mut Frame) {
        match self.state {
            // Render AddPatient component
            PatientsState::AddPatient => self.add_patient.render(frame),
            // Render ListPatients component
            PatientsState::ListPatients => self.list_patients.render(frame),
            // Render DeletePatient component
            PatientsState::DeletePatient => {
                // If delete_patient exists, render it
                if let Some(delete_patient) = &self.delete_patient {
                    delete_patient.render(frame);
                }
            }
        }
    }
}
