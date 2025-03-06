//! Patient management module within the Hospital application.

use crate::app::SelectedApp;
use crate::components::hospital::patients::add::AddPatient;
use crate::components::hospital::patients::delete::DeletePatient;
use crate::components::hospital::patients::list::ListPatients;
use crate::components::hospital::patients::update::UpdatePatient;
use crate::components::Component;
use crate::tui::Frame;
use anyhow::Result;
use crossterm::event::KeyEvent;

pub mod add;
pub mod delete;
pub mod list;
pub mod update;

/// Represents the actions that can be performed within the patient management component.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatientAction {
    /// Return to the home screen.
    BackToHome,
    /// Return to the patient list.
    #[allow(dead_code)]
    BackToList,
}

/// Represents the different states of the patient management component.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatientsState {
    /// State for adding a new patient.
    AddPatient,
    /// State for listing all patients.
    ListPatients,
    /// State for deleting a patient.
    DeletePatient,
    /// State for updating patient information.
    UpdatePatient,
}

/// Manages the patient-related functionalities within the hospital application.
///
/// This struct encapsulates the components for adding, listing, deleting, and updating patient information.
pub struct Patients {
    /// The component for adding a new patient.
    pub add_patient: AddPatient,
    /// The component for listing patients.
    pub list_patients: ListPatients,
    /// The component for deleting a patient, wrapped in an `Option`.  `None` if not active.
    pub delete_patient: Option<DeletePatient>,
    /// The component for updating patient information, wrapped in an `Option`. `None` if not active.
    pub update_patient: Option<UpdatePatient>,
    /// The current state of the patient management component.
    pub state: PatientsState,
}

impl Patients {
    /// Creates a new `Patients` component.
    ///
    /// Initializes the components for adding, listing, deleting, and updating patients.
    /// Sets the initial state to `ListPatients`.
    pub fn new() -> Self {
        Self {
            add_patient: AddPatient::new(),
            list_patients: ListPatients::new(),
            delete_patient: None,
            update_patient: None,
            state: PatientsState::ListPatients,
        }
    }

    /// Initializes the patient list.
    ///
    /// Fetches the list of patients if the current state is `ListPatients`.
    ///
    /// # Errors
    ///
    /// Returns an error if fetching the patient list fails.
    pub fn initialize_list(&mut self) -> Result<()> {
        if self.state == PatientsState::ListPatients {
            self.list_patients.fetch_patients()?;
        }
        Ok(())
    }
}

impl Component for Patients {
    /// Handles key input events for the patient management component.
    ///
    /// This function processes key events and takes actions based on the current state.
    /// It manages the transitions between different states (add, list, delete, update)
    /// and delegates input handling to the relevant sub-components.
    ///
    /// # Arguments
    ///
    /// * `event` - The key event to handle.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Some(SelectedApp::None))` if the component should return to the main app.
    /// Returns `Ok(None)` if the event was handled within the component.
    /// Returns an error if input handling fails.
    fn handle_input(&mut self, event: KeyEvent) -> Result<Option<SelectedApp>> {
        match self.state {
            PatientsState::AddPatient => {
                if let Some(action) = self.add_patient.handle_input(event)? {
                    match action {
                        PatientAction::BackToHome => {
                            self.state = PatientsState::ListPatients;
                            self.initialize_list()?;
                            return Ok(Some(SelectedApp::None));
                        }
                        _ => {}
                    }
                }
            }
            PatientsState::ListPatients => {
                if let Some(action) = self.list_patients.handle_input(event)? {
                    match action {
                        PatientAction::BackToHome => return Ok(Some(SelectedApp::None)),
                        _ => {}
                    }
                }
            }
            PatientsState::DeletePatient => {
                if let Some(delete_patient) = &mut self.delete_patient {
                    if let Some(selected_app) = delete_patient.handle_input(event)? {
                        match selected_app {
                            SelectedApp::None => {
                                self.state = PatientsState::ListPatients;
                                self.delete_patient = None;
                                self.initialize_list()?;
                                return Ok(Some(SelectedApp::None));
                            }
                            _ => {}
                        }
                    }
                }
            }
            PatientsState::UpdatePatient => {
                if let Some(update_patient) = &mut self.update_patient {
                    if let Some(action) = update_patient.handle_input(event)? {
                        match action {
                            SelectedApp::None => {
                                self.state = PatientsState::ListPatients;
                                self.update_patient = None;
                                self.initialize_list()?;
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

    /// Renders the patient management component to the terminal frame.
    ///
    /// Based on the current state, this function calls the `render` method of the
    /// appropriate sub-component (add, list, delete, update) to display the UI.
    ///
    /// # Arguments
    ///
    /// * `frame` - The terminal frame to render to.
    fn render(&self, frame: &mut Frame) {
        match self.state {
            PatientsState::AddPatient => self.add_patient.render(frame),
            PatientsState::ListPatients => self.list_patients.render(frame),
            PatientsState::DeletePatient => {
                if let Some(delete_patient) = &self.delete_patient {
                    delete_patient.render(frame);
                }
            }
            PatientsState::UpdatePatient => {
                if let Some(update_patient) = &self.update_patient {
                    update_patient.render(frame);
                }
            }
        }
    }
}
