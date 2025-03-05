//! Patient management module within the Hospital application.

use crate::components::hospital::patients::add::AddPatient;
use crate::components::hospital::patients::list::ListPatients;
use crate::components::Component;
use crate::tui::Frame;
use anyhow::Result;
use crossterm::event::KeyEvent;

pub mod add;
pub mod list;

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
    add_patient: AddPatient,     // Component for adding a patient
    list_patients: ListPatients, // Component for listing patients
    state: PatientsState,        // Add a state to track which component is active
}

impl Patients {
    pub fn new() -> Self {
        Self {
            add_patient: AddPatient::new(),
            list_patients: ListPatients::new(),
            state: PatientsState::ListPatients, // Start by showing the list
        }
    }

    // Add a method to initialize data (fetch patients) when switching to ListPatients
    pub fn initialize_list(&mut self) -> Result<()> {
        if self.state == PatientsState::ListPatients {
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
                            return Ok(Some(crate::app::SelectedApp::None))
                        }
                    }
                }
            }
            PatientsState::ListPatients => {
                if let Some(action) = self.list_patients.handle_input(event)? {
                    match action {
                        PatientAction::BackToHome => {
                            return Ok(Some(crate::app::SelectedApp::None))
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
