//! Hospital management module.

use crate::components::Component;
use crate::tui::Frame;
use anyhow::Result;
use crossterm::event::KeyEvent;

pub mod patients; // Add this line

// This enum represents the different states within the Hospital module
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HospitalState {
    Patients, // Add more states later (Staff, Billing, etc.)
              // Add other states as you add features
}

pub struct HospitalApp {
    state: HospitalState,             // Current state
    pub patients: patients::Patients, // The Patients component
}

impl HospitalApp {
    pub fn new() -> Self {
        Self {
            state: HospitalState::Patients, // Start with the Patients component
            patients: patients::Patients::new(), // Initialize Patients component
        }
    }
}

impl Component for HospitalApp {
    fn handle_input(&mut self, event: KeyEvent) -> Result<Option<crate::app::SelectedApp>> {
        // For now, we'll just delegate all input to the Patients component.
        // Later, you might handle top-level navigation here (e.g., switching
        // between Patients, Staff, Billing, etc., using tabs or a menu).

        match self.state {
            HospitalState::Patients => {
                if let Some(_) = self.patients.handle_input(event)? {
                    // When PatientAction::BackToHome is received, return to home
                    return Ok(Some(crate::app::SelectedApp::None));
                }
            } // Handle other HospitalStates later
        }
        Ok(None)
    }

    fn render(&self, frame: &mut Frame) {
        // For now, just render the Patients component.  Later, you might
        // render a top-level UI for the Hospital module (e.g., tabs for
        // different sections).
        match self.state {
            HospitalState::Patients => self.patients.render(frame),
        }
    }
}
