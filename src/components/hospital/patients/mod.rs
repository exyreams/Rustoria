//! Patient management module within the Hospital application.

use crate::components::hospital::patients::add::AddPatient; // Import AddPatient
use crate::components::Component;
use crate::tui::Frame;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::Paragraph;

pub mod add; // Declare the add module

// Action enum to manage the actions of patient component
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatientAction {
    BackToHome,
}

// State for patient management
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum PatientState {
    Main, // Main patient list
    Adding,
    // Add other states as you implement more features (e.g., Editing, Viewing)
}

pub struct Patients {
    state: PatientState,     // Current state
    add_patient: AddPatient, // Component for adding a patient
}

impl Patients {
    pub fn new() -> Self {
        Self {
            state: PatientState::Adding, // Start in Add mode for now
            add_patient: AddPatient::new(),
        }
    }
}

impl Component for Patients {
    fn handle_input(&mut self, event: KeyEvent) -> Result<Option<crate::app::SelectedApp>> {
        // Handle input based on the current state
        match self.state {
            PatientState::Adding => {
                if let Some(action) = self.add_patient.handle_input(event)? {
                    match action {
                        // When action is back we get back to main menu
                        PatientAction::BackToHome => {
                            return Ok(Some(crate::app::SelectedApp::None))
                        }
                    }
                }
            }
            PatientState::Main => {
                // Handle input for the main patient list (if you had one)
                if event.code == KeyCode::Char('a') {
                    self.state = PatientState::Adding;
                }
            }
        }
        Ok(None)
    }

    fn render(&self, frame: &mut Frame) {
        // Render the UI based on the current state
        match self.state {
            PatientState::Adding => {
                self.add_patient.render(frame);
            }
            PatientState::Main => {
                // Render the main patient list (placeholder for now)
                let paragraph = Paragraph::new("Patient Management Main Screen (Placeholder)")
                    .alignment(ratatui::prelude::Alignment::Center);
                frame.render_widget(paragraph, frame.area());
            }
        }
    }
}
