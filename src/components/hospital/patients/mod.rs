//! Patient management module within the Hospital application.

use crate::components::hospital::patients::add::AddPatient;
use crate::components::Component;
use crate::tui::Frame;
use anyhow::Result;
use crossterm::event::KeyEvent;

pub mod add; // Declare the add module

// Action enum to manage the actions of patient component
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatientAction {
    BackToHome,
}

pub struct Patients {
    add_patient: AddPatient, // Component for adding a patient
}

impl Patients {
    pub fn new() -> Self {
        Self {
            add_patient: AddPatient::new(),
        }
    }
}

impl Component for Patients {
    fn handle_input(&mut self, event: KeyEvent) -> Result<Option<crate::app::SelectedApp>> {
        // Since we're only using the AddPatient component, just forward the input handling
        if let Some(action) = self.add_patient.handle_input(event)? {
            match action {
                PatientAction::BackToHome => return Ok(Some(crate::app::SelectedApp::None)),
            }
        }
        Ok(None)
    }

    fn render(&self, frame: &mut Frame) {
        // Directly render the AddPatient component
        self.add_patient.render(frame);
    }
}
