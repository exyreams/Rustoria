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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatientAction {
    BackToHome,
    #[allow(dead_code)]
    BackToList,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatientsState {
    AddPatient,
    ListPatients,
    DeletePatient,
    UpdatePatient,
}

pub struct Patients {
    pub add_patient: AddPatient,
    pub list_patients: ListPatients,
    pub delete_patient: Option<DeletePatient>,
    pub update_patient: Option<UpdatePatient>,
    pub state: PatientsState,
}

impl Patients {
    pub fn new() -> Self {
        Self {
            add_patient: AddPatient::new(),
            list_patients: ListPatients::new(),
            delete_patient: None,
            update_patient: None,
            state: PatientsState::ListPatients,
        }
    }

    pub fn initialize_list(&mut self) -> Result<()> {
        if self.state == PatientsState::ListPatients {
            self.list_patients.fetch_patients()?;
        }
        Ok(())
    }
}

impl Component for Patients {
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
