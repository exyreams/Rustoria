use crate::auth::{login, Credentials};
use crate::components::hospital::finance::FinanceState;
use crate::components::hospital::records::delete::DeleteRecord;
use crate::components::hospital::records::update::UpdateRecord;
use crate::components::hospital::records::RecordsState;
use crate::components::hospital::staff::delete::DeleteStaff;
use crate::components::hospital::staff::update::UpdateStaff;
use crate::components::hospital::{self, HospitalState};
use crate::components::{home::Home, login::Login, register::Register, Component};
use crate::tui::{self, Tui};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectedApp {
    PatientAdd,
    PatientList,
    PatientDelete,
    PatientUpdate,
    StaffAdd,
    StaffAssign,
    StaffList,
    StaffDelete,
    StaffUpdate,
    RecordStore,
    RecordRetrieve,
    RecordUpdate,
    RecordDelete,
    BillingInvoice,
    BillingView,
    BillingUpdate,
    Hospital,
    None,
    Quit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    Init,
    Login,
    Register,
    Home,
    Running(SelectedApp),
    #[allow(dead_code)]
    Quitting,
}

pub struct App {
    pub state: AppState,
    pub should_quit: bool,
    pub home: Home,
    pub login: Login,
    pub register: Register,
    pub hospital: Option<hospital::HospitalApp>,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: AppState::Init,
            should_quit: false,
            home: Home::new(),
            login: Login::new(),
            register: Register::new(),
            hospital: None,
        }
    }

    pub fn run(&mut self, tui: &mut Tui) -> Result<()> {
        self.state = AppState::Login;

        while !self.should_quit {
            tui.draw(|frame| self.render_ui(frame))?;

            self.handle_input(tui)?;
        }
        Ok(())
    }

    fn handle_input(&mut self, tui: &mut Tui) -> Result<()> {
        match tui.next_event()? {
            tui::Event::Input(event) => {
                if let crossterm::event::Event::Key(KeyEvent {
                    code: KeyCode::Char('q'),
                    modifiers: crossterm::event::KeyModifiers::CONTROL,
                    ..
                }) = event
                {
                    self.should_quit = true;
                    return Ok(());
                }

                match self.state {
                    AppState::Init => {
                        self.state = AppState::Login;
                    }
                    AppState::Login => {
                        if let crossterm::event::Event::Key(key) = event {
                            let result = self.login.handle_input(key)?;
                            if let Some(selected_app) = result {
                                match selected_app {
                                    SelectedApp::Quit => {
                                        self.should_quit = true;
                                        return Ok(());
                                    }
                                    SelectedApp::None => {
                                        let credentials = Credentials {
                                            username: self.login.username.clone(),
                                            password: self.login.password.clone(),
                                        };

                                        match login(credentials) {
                                            Ok(user_id) => {
                                                self.home.load_username(user_id)?;
                                                self.state = AppState::Home;
                                            }
                                            Err(err) => {
                                                self.login.error_message = Some(format!("{}", err));
                                            }
                                        }
                                    }
                                    SelectedApp::Hospital => {
                                        self.state = AppState::Register;
                                    }
                                    SelectedApp::PatientAdd
                                    | SelectedApp::PatientList
                                    | SelectedApp::PatientDelete
                                    | SelectedApp::PatientUpdate
                                    | SelectedApp::StaffAdd
                                    | SelectedApp::StaffAssign
                                    | SelectedApp::StaffList
                                    | SelectedApp::StaffDelete
                                    | SelectedApp::StaffUpdate
                                    | SelectedApp::RecordStore
                                    | SelectedApp::RecordRetrieve
                                    | SelectedApp::RecordUpdate
                                    | SelectedApp::RecordDelete
                                    | SelectedApp::BillingInvoice
                                    | SelectedApp::BillingView
                                    | SelectedApp::BillingUpdate => {
                                        self.login.error_message =
                                            Some("Please log in first.".to_string());
                                    }
                                }
                            }
                        }
                    }
                    AppState::Register => {
                        if let crossterm::event::Event::Key(key) = event {
                            if let Some(_selected_app) = self.register.handle_input(key)? {
                                self.state = AppState::Login;

                                if self.register.registration_success {
                                    self.login.username.clear();
                                    self.login.password.clear();
                                    self.login.error_message = None;
                                    self.login.set_success_message(
                                        "Registration successful! Please log in.".to_string(),
                                    );
                                }
                            }
                        }
                    }

                    AppState::Home => {
                        if let crossterm::event::Event::Key(key) = event {
                            if let Some(selected_app) = self.home.handle_input(key)? {
                                match selected_app {
                                    SelectedApp::PatientAdd => {
                                        self.hospital = Some(hospital::HospitalApp::new());
                                        if let Some(hospital) = &mut self.hospital {
                                            hospital.set_patients_state(
                                                hospital::patients::PatientsState::AddPatient,
                                            );
                                        }
                                        self.state = AppState::Running(selected_app);
                                    }
                                    SelectedApp::PatientList => {
                                        self.hospital = Some(hospital::HospitalApp::new());
                                        if let Some(hospital) = &mut self.hospital {
                                            hospital.set_patients_state(
                                                hospital::patients::PatientsState::ListPatients,
                                            );
                                        }
                                        self.state = AppState::Running(selected_app);
                                    }
                                    SelectedApp::PatientDelete => {
                                        self.hospital = Some(hospital::HospitalApp::new());
                                        if let Some(hospital) = &mut self.hospital {
                                            hospital.set_patients_state(
                                                hospital::patients::PatientsState::DeletePatient,
                                            );
                                            hospital.patients.delete_patient = Some(
                                                hospital::patients::delete::DeletePatient::new(),
                                            );
                                        }
                                        self.state = AppState::Running(selected_app);
                                    }
                                    SelectedApp::PatientUpdate => {
                                        self.hospital = Some(hospital::HospitalApp::new());
                                        if let Some(hospital) = &mut self.hospital {
                                            hospital.set_patients_state(
                                                hospital::patients::PatientsState::UpdatePatient,
                                            );
                                            hospital.patients.update_patient = Some(
                                                hospital::patients::update::UpdatePatient::new(),
                                            );
                                        }
                                        self.state = AppState::Running(selected_app);
                                    }

                                    SelectedApp::StaffAdd => {
                                        self.hospital = Some(hospital::HospitalApp::new());
                                        if let Some(hospital) = &mut self.hospital {
                                            hospital.set_state(hospital::HospitalState::Staff);

                                            hospital.set_staff_state(
                                                hospital::staff::StaffState::AddStaff,
                                            );
                                        }
                                        self.state = AppState::Running(selected_app);
                                    }
                                    SelectedApp::StaffAssign => {
                                        self.hospital = Some(hospital::HospitalApp::new());
                                        if let Some(hospital) = &mut self.hospital {
                                            hospital.set_state(hospital::HospitalState::Staff);
                                            hospital.set_staff_state(crate::components::hospital::staff::StaffState::AssignStaff);

                                            if hospital.staff.assign_staff.is_none() {
                                                let mut assign_staff = crate::components::hospital::staff::assign::AssignStaff::new();
                                                assign_staff.fetch_staff()?;
                                                hospital.staff.assign_staff = Some(assign_staff);
                                            }
                                        }
                                        self.state = AppState::Running(selected_app);
                                    }
                                    SelectedApp::StaffList => {
                                        self.hospital = Some(hospital::HospitalApp::new());
                                        if let Some(hospital) = &mut self.hospital {
                                            hospital.set_state(hospital::HospitalState::Staff);
                                            hospital.set_staff_state(
                                                hospital::staff::StaffState::ListStaff,
                                            );
                                        }
                                        self.state = AppState::Running(selected_app);
                                    }

                                    SelectedApp::StaffUpdate => {
                                        self.hospital = Some(hospital::HospitalApp::new());
                                        if let Some(hospital) = &mut self.hospital {
                                            hospital.set_state(hospital::HospitalState::Staff);
                                            hospital.set_staff_state(
                                                hospital::staff::StaffState::UpdateStaff,
                                            );
                                            hospital.staff.update_staff = Some(UpdateStaff::new());
                                            if let Some(update_staff) =
                                                &mut hospital.staff.update_staff
                                            {
                                                update_staff.fetch_staff()?; // <--- KEY CHANGE
                                            }
                                        }
                                        self.state = AppState::Running(selected_app);
                                    }
                                    SelectedApp::StaffDelete => {
                                        self.hospital = Some(hospital::HospitalApp::new());
                                        if let Some(hospital) = &mut self.hospital {
                                            hospital.set_state(hospital::HospitalState::Staff);
                                            hospital.set_staff_state(
                                                hospital::staff::StaffState::DeleteStaff,
                                            );
                                            hospital.staff.delete_staff = Some(DeleteStaff::new());
                                            if let Some(delete_staff) =
                                                &mut hospital.staff.delete_staff
                                            {
                                                delete_staff.fetch_staff()?;
                                            }
                                        }
                                        self.state = AppState::Running(selected_app);
                                    }
                                    SelectedApp::RecordStore => {
                                        self.hospital = Some(hospital::HospitalApp::new());
                                        if let Some(hospital) = &mut self.hospital {
                                            hospital.set_state(hospital::HospitalState::Records);
                                            hospital.set_records_state(RecordsState::StoreRecord);
                                            hospital.records.initialize_list()?;
                                        }
                                        self.state = AppState::Running(selected_app);
                                    }
                                    SelectedApp::RecordRetrieve => {
                                        self.hospital = Some(hospital::HospitalApp::new());
                                        if let Some(hospital) = &mut self.hospital {
                                            hospital.set_state(hospital::HospitalState::Records);
                                            hospital
                                                .set_records_state(RecordsState::RetrieveRecords);
                                            hospital.records.initialize_list()?;
                                        }
                                        self.state = AppState::Running(selected_app);
                                    }
                                    SelectedApp::RecordUpdate => {
                                        self.hospital = Some(hospital::HospitalApp::new());
                                        if let Some(hospital) = &mut self.hospital {
                                            hospital.set_state(hospital::HospitalState::Records);
                                            hospital.set_records_state(RecordsState::UpdateRecord);
                                            hospital.records.update_record =
                                                Some(UpdateRecord::new());
                                            if let Some(update_record) =
                                                &mut hospital.records.update_record
                                            {
                                                update_record.fetch_records()?;
                                            }
                                        }
                                        self.state = AppState::Running(selected_app);
                                    }
                                    SelectedApp::RecordDelete => {
                                        self.hospital = Some(hospital::HospitalApp::new());
                                        if let Some(hospital) = &mut self.hospital {
                                            hospital.set_state(hospital::HospitalState::Records);
                                            hospital.set_records_state(RecordsState::DeleteRecord);
                                            hospital.records.delete_record =
                                                Some(DeleteRecord::new());
                                            if let Some(delete_record) =
                                                &mut hospital.records.delete_record
                                            {
                                                delete_record.fetch_records()?;
                                            }
                                        }
                                        self.state = AppState::Running(selected_app);
                                    }
                                    SelectedApp::BillingInvoice => {
                                        self.hospital = Some(hospital::HospitalApp::new());
                                        if let Some(hospital) = &mut self.hospital {
                                            hospital.set_state(HospitalState::Finance);
                                            hospital.set_finance_state(FinanceState::Invoice);
                                        }
                                        self.state = AppState::Running(selected_app);
                                    }
                                    SelectedApp::BillingView => {
                                        self.hospital = Some(hospital::HospitalApp::new());
                                        if let Some(hospital) = &mut self.hospital {
                                            hospital.set_state(HospitalState::Finance);
                                            hospital.set_finance_state(FinanceState::View);
                                        }
                                        self.state = AppState::Running(selected_app);
                                    }
                                    SelectedApp::BillingUpdate => {
                                        self.hospital = Some(hospital::HospitalApp::new());
                                        if let Some(hospital) = &mut self.hospital {
                                            hospital.set_state(HospitalState::Finance);
                                            hospital.set_finance_state(FinanceState::Update);
                                        }
                                        self.state = AppState::Running(selected_app);
                                    }
                                    SelectedApp::Hospital => {
                                        self.hospital = Some(hospital::HospitalApp::new());
                                        self.state = AppState::Running(selected_app);
                                    }
                                    SelectedApp::Quit => {
                                        self.should_quit = true;
                                        return Ok(());
                                    }
                                    SelectedApp::None => {
                                        // Go back to login (logout)
                                        self.state = AppState::Login;
                                    }
                                }
                            }
                        }
                    }
                    AppState::Running(selected_app) => match selected_app {
                        SelectedApp::PatientAdd
                        | SelectedApp::PatientList
                        | SelectedApp::PatientDelete
                        | SelectedApp::PatientUpdate
                        | SelectedApp::StaffAdd
                        | SelectedApp::StaffList
                        | SelectedApp::StaffDelete
                        | SelectedApp::StaffUpdate
                        | SelectedApp::RecordStore
                        | SelectedApp::RecordRetrieve
                        | SelectedApp::RecordUpdate
                        | SelectedApp::RecordDelete
                        | SelectedApp::BillingInvoice
                        | SelectedApp::BillingView
                        | SelectedApp::BillingUpdate => {
                            if let Some(hospital) = &mut self.hospital {
                                if let crossterm::event::Event::Key(key) = event {
                                    if let Some(action) = hospital.handle_input(key)? {
                                        match action {
                                            SelectedApp::None => {
                                                self.state = AppState::Home;
                                                self.hospital = None;
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                            } else {
                                self.state = AppState::Home;
                            }
                        }
                        SelectedApp::StaffAssign => {
                            if let Some(hospital) = &mut self.hospital {
                                if let crossterm::event::Event::Key(key_event) = event {
                                    if let Some(selected_app) = hospital.handle_input(key_event)? {
                                        match selected_app {
                                            SelectedApp::None => {
                                                self.state = AppState::Home;
                                                self.hospital = None;
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                            }
                        }
                        _ => {
                            self.state = AppState::Home;
                        }
                    },

                    AppState::Quitting => {
                        self.should_quit = true;
                    }
                }
            }
            tui::Event::Tick => {
                if let AppState::Login = self.state {
                    self.login.check_error_timeout();
                }
                if let AppState::Register = self.state {
                    self.register.check_error_timeout();
                }
            }
        }
        Ok(())
    }

    fn render_ui(&self, frame: &mut crate::tui::Frame<'_>) {
        match self.state {
            AppState::Init => {}
            AppState::Login => self.login.render(frame),
            AppState::Register => self.register.render(frame),
            AppState::Home => self.home.render(frame),
            AppState::Running(SelectedApp::PatientAdd)
            | AppState::Running(SelectedApp::PatientList)
            | AppState::Running(SelectedApp::PatientDelete)
            | AppState::Running(SelectedApp::PatientUpdate)
            | AppState::Running(SelectedApp::StaffAdd)
            | AppState::Running(SelectedApp::StaffAssign)
            | AppState::Running(SelectedApp::StaffList)
            | AppState::Running(SelectedApp::StaffDelete)
            | AppState::Running(SelectedApp::StaffUpdate)
            | AppState::Running(SelectedApp::RecordStore)
            | AppState::Running(SelectedApp::RecordRetrieve)
            | AppState::Running(SelectedApp::RecordUpdate)
            | AppState::Running(SelectedApp::RecordDelete)
            | AppState::Running(SelectedApp::BillingInvoice)
            | AppState::Running(SelectedApp::BillingUpdate)
            | AppState::Running(SelectedApp::BillingView) => {
                if let Some(hospital) = &self.hospital {
                    hospital.render(frame);
                }
            }
            AppState::Running(SelectedApp::Hospital) => {
                if let Some(hospital) = &self.hospital {
                    hospital.render(frame);
                }
            }

            AppState::Running(SelectedApp::None) | AppState::Running(SelectedApp::Quit) => todo!(),
            AppState::Quitting => todo!(),
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
