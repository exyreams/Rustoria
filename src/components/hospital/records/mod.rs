use crate::app::SelectedApp;
use crate::components::Component;
use crate::tui::Frame;
use anyhow::Result;
use crossterm::event::KeyEvent;

pub mod delete;
pub mod retrieve;
pub mod store;
pub mod update;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordsState {
    StoreRecord,
    RetrieveRecords,
    DeleteRecord,
    UpdateRecord,
}

pub struct Records {
    pub store_record: store::StoreRecord,
    pub retrieve_records: retrieve::RetrieveRecords,
    pub delete_record: Option<delete::DeleteRecord>,
    pub update_record: Option<update::UpdateRecord>,
    pub state: RecordsState,
}

impl Records {
    pub fn new() -> Self {
        Self {
            store_record: store::StoreRecord::new(),
            retrieve_records: retrieve::RetrieveRecords::new(),
            delete_record: None,
            update_record: None,
            state: RecordsState::RetrieveRecords,
        }
    }

    pub fn initialize_list(&mut self) -> Result<()> {
        if self.state == RecordsState::RetrieveRecords {
            self.retrieve_records.fetch_records()?;
        }
        self.store_record.load_patients()?;
        Ok(())
    }
}

impl Component for Records {
    fn handle_input(&mut self, event: KeyEvent) -> Result<Option<SelectedApp>> {
        match self.state {
            RecordsState::StoreRecord => {
                if let Some(selected_app) = self.store_record.handle_input(event)? {
                    match selected_app {
                        SelectedApp::None => {
                            self.state = RecordsState::RetrieveRecords;
                            self.initialize_list()?;
                            return Ok(Some(SelectedApp::None));
                        }
                        _ => {}
                    }
                }
            }
            RecordsState::RetrieveRecords => {
                if let Some(selected_app) = self.retrieve_records.handle_input(event)? {
                    match selected_app {
                        SelectedApp::None => return Ok(Some(SelectedApp::None)),
                        _ => {}
                    }
                }
            }
            RecordsState::DeleteRecord => {
                if let Some(delete_record) = &mut self.delete_record {
                    if let Some(selected_app) = delete_record.handle_input(event)? {
                        match selected_app {
                            SelectedApp::None => {
                                self.state = RecordsState::RetrieveRecords;
                                self.delete_record = None;
                                self.initialize_list()?;
                                return Ok(Some(SelectedApp::None));
                            }
                            _ => {}
                        }
                    }
                }
            }
            RecordsState::UpdateRecord => {
                if let Some(update_record) = &mut self.update_record {
                    if let Some(selected_app) = update_record.handle_input(event)? {
                        match selected_app {
                            SelectedApp::None => {
                                self.state = RecordsState::RetrieveRecords;
                                self.update_record = None;
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
            RecordsState::StoreRecord => self.store_record.render(frame),
            RecordsState::RetrieveRecords => self.retrieve_records.render(frame),
            RecordsState::DeleteRecord => {
                if let Some(delete_record) = &self.delete_record {
                    delete_record.render(frame);
                }
            }
            RecordsState::UpdateRecord => {
                if let Some(update_record) = &self.update_record {
                    update_record.render(frame);
                }
            }
        }
    }
}

impl Default for Records {
    fn default() -> Self {
        Self::new()
    }
}
