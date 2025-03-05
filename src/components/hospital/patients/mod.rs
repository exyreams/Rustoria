//! Patient management module within the Hospital application.

use crate::app::SelectedApp;
use crate::components::hospital::patients::add::AddPatient;
use crate::components::hospital::patients::list::ListPatients; // Import ListPatients
use crate::components::Component;
use crate::tui::Frame;
use anyhow::Result;
use crossterm::event::KeyEvent;

pub mod add;
pub mod list; // Add the list module

// Action enum to manage the actions of patient component
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatientAction {
    BackToHome,
}

// Add a state enum to manage switching between Add and List
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatientsState {
    AddPatient,
    ListPatients,
}

pub struct Patients {
    pub add_patient: AddPatient,     // Component for adding a patient
    pub list_patients: ListPatients, // Component for listing patients
    pub state: PatientsState,        // Add a state to track which component is active
}

impl Patients {
    pub fn new() -> Self {
        Self {
            add_patient: AddPatient::new(),
            list_patients: ListPatients::new(),
            state: PatientsState::ListPatients, // Default state can be ListPatients
        }
    }

    // Add a method to initialize data (fetch patients) when switching to ListPatients
    pub fn initialize_list(&mut self) -> Result<()> {
        if self.state == PatientsState::ListPatients {
            // Only fetch if in ListPatients state
            self.list_patients.fetch_patients()?;
        }
        Ok(())
    }
}

impl Component for Patients {
    fn handle_input(&mut self, event: KeyEvent) -> Result<Option<crate::app::SelectedApp>> {
        // Handle input based on the current state
        match self.state {
            PatientsState::AddPatient => {
                if let Some(action) = self.add_patient.handle_input(event)? {
                    match action {
                        PatientAction::BackToHome => {
                            self.state = PatientsState::ListPatients; // Switch back to ListPatients
                            self.initialize_list()?;
                            return Ok(Some(SelectedApp::None)); // Go to home
                        }
                    }
                }
            }
            PatientsState::ListPatients => {
                if let Some(action) = self.list_patients.handle_input(event)? {
                    match action {
                        PatientAction::BackToHome => {
                            return Ok(Some(SelectedApp::None)); // Go to home.  Don't change list/add state.
                        }
                    }
                }
            }
        }
        Ok(None)
    }

    fn render(&self, frame: &mut Frame) {
        // Render the appropriate component based on the state
        match self.state {
            PatientsState::AddPatient => self.add_patient.render(frame),
            PatientsState::ListPatients => self.list_patients.render(frame),
        }
    }
}
