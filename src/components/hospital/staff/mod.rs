use self::add::AddStaff;
use self::assign::AssignStaff;
use self::delete::DeleteStaff;
use self::list::ListStaff;
use self::update::UpdateStaff;
use crate::app::SelectedApp;
use crate::components::Component;
use crate::tui::Frame;
use anyhow::Result;
use crossterm::event::KeyEvent;
pub mod add;
pub mod assign;
pub mod delete;
pub mod list;
pub mod update;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StaffAction {
    BackToHome,
    #[allow(dead_code)]
    BackToList,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StaffState {
    AddStaff,
    ListStaff,
    DeleteStaff,
    UpdateStaff,
    AssignStaff,
}

pub struct Staff {
    add_staff: AddStaff,
    list_staff: ListStaff,
    pub delete_staff: Option<DeleteStaff>,
    pub update_staff: Option<UpdateStaff>,
    pub assign_staff: Option<AssignStaff>,
    pub state: StaffState,
}

impl Staff {
    pub fn new() -> Self {
        let mut list_staff = ListStaff::new();
        list_staff.fetch_staff().expect("Failed to fetch staff");

        Self {
            add_staff: AddStaff::new(),
            list_staff,
            delete_staff: None,
            update_staff: None,
            assign_staff: None,
            state: StaffState::ListStaff,
        }
    }

    pub fn initialize_list(&mut self) -> Result<()> {
        // Only fetch if in ListStaff
        if self.state == StaffState::ListStaff {
            self.list_staff.fetch_staff()?;
        }
        Ok(())
    }
}

impl Component for Staff {
    fn handle_input(&mut self, event: KeyEvent) -> Result<Option<SelectedApp>> {
        match self.state {
            StaffState::AddStaff => {
                if let Some(selected_app) = self.add_staff.handle_input(event)? {
                    match selected_app {
                        SelectedApp::None => {
                            self.state = StaffState::ListStaff;
                            self.initialize_list()?;
                            return Ok(Some(SelectedApp::None));
                        }
                        _ => {}
                    }
                }
            }
            StaffState::ListStaff => {
                if let Some(selected_app) = self.list_staff.handle_input(event)? {
                    match selected_app {
                        SelectedApp::None => {
                            return Ok(Some(SelectedApp::None));
                        }
                        _ => {}
                    }
                }
            }
            StaffState::DeleteStaff => {
                if let Some(delete_staff) = &mut self.delete_staff {
                    if let Some(selected_app) = delete_staff.handle_input(event)? {
                        match selected_app {
                            SelectedApp::None => {
                                self.state = StaffState::ListStaff;
                                self.delete_staff = None;
                                self.initialize_list()?;
                                return Ok(Some(SelectedApp::None));
                            }
                            _ => {}
                        }
                    }
                }
            }
            StaffState::UpdateStaff => {
                if let Some(update_staff) = &mut self.update_staff {
                    if let Some(selected_app) = update_staff.handle_input(event)? {
                        match selected_app {
                            SelectedApp::None => {
                                self.state = StaffState::ListStaff;
                                self.update_staff = None;
                                self.initialize_list()?;
                                return Ok(Some(SelectedApp::None));
                            }
                            _ => {}
                        }
                    }
                }
            }
            StaffState::AssignStaff => {
                if let Some(assign_staff) = &mut self.assign_staff {
                    if let Some(selected_app) = assign_staff.handle_input(event)? {
                        match selected_app {
                            SelectedApp::None => {
                                self.state = StaffState::ListStaff;
                                self.assign_staff = None;
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
            StaffState::AddStaff => self.add_staff.render(frame),
            StaffState::ListStaff => self.list_staff.render(frame),
            StaffState::DeleteStaff => {
                if let Some(delete_staff) = &self.delete_staff {
                    delete_staff.render(frame);
                }
            }
            StaffState::UpdateStaff => {
                if let Some(update_staff) = &self.update_staff {
                    update_staff.render(frame);
                }
            }
            StaffState::AssignStaff => {
                if let Some(assign_staff) = &self.assign_staff {
                    assign_staff.render(frame);
                }
            }
        }
    }
}

impl Default for Staff {
    fn default() -> Self {
        Self::new()
    }
}
