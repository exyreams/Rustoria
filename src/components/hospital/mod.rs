use self::finance::Finance;
use self::finance::FinanceState;
use self::patients::PatientsState;
use self::records::Records;
use self::records::RecordsState;
use self::staff::Staff;
use self::staff::StaffState;
use crate::components::Component;
use crate::tui::Frame;
use anyhow::Result;
use crossterm::event::KeyEvent;

pub mod finance;
pub mod patients;
pub mod records;
pub mod staff;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HospitalState {
    Finance,
    Patients,
    Staff,
    Records,
}

pub struct HospitalApp {
    pub finance: Finance,
    pub state: HospitalState,
    pub patients: patients::Patients,
    pub records: Records,
    pub staff: Staff,
}

impl HospitalApp {
    pub fn new() -> Self {
        let finance = Finance::new();

        let mut patients = patients::Patients::new();

        patients
            .initialize_list()
            .expect("Failed to initialize patient list");

        let mut staff = Staff::new();
        staff
            .initialize_list()
            .expect("Failed to initialize staff list");

        let mut records = Records::new();
        records
            .initialize_list()
            .expect("Failed to initialize records list");

        Self {
            state: HospitalState::Patients,
            finance,
            patients,
            staff,
            records,
        }
    }

    pub fn set_patients_state(&mut self, state: PatientsState) {
        self.patients.state = state;
        if state == PatientsState::ListPatients {
            if let Err(e) = self.patients.initialize_list() {
                eprintln!("Error initializing patient list: {}", e);
            }
        }
    }

    pub fn set_state(&mut self, new_state: HospitalState) {
        self.state = new_state;
    }

    pub fn set_staff_state(&mut self, state: StaffState) {
        self.staff.state = state;
        if state == StaffState::ListStaff {
            if let Err(e) = self.staff.initialize_list() {
                eprintln!("Error initializing staff list: {}", e);
            }
        }
    }

    pub fn set_records_state(&mut self, state: RecordsState) {
        self.records.state = state;
        if state == RecordsState::RetrieveRecords {
            if let Err(e) = self.records.initialize_list() {
                eprintln!("Error initializing records list: {}", e);
            }
        }
    }

    pub fn set_finance_state(&mut self, state: FinanceState) {
        self.finance.set_finance_state(state);
    }
}

impl Component for HospitalApp {
    fn handle_input(&mut self, event: KeyEvent) -> Result<Option<crate::app::SelectedApp>> {
        match self.state {
            HospitalState::Finance => {
                if let Some(action) = self.finance.handle_input(event)? {
                    return Ok(Some(action));
                }
            }
            HospitalState::Patients => {
                if let Some(action) = self.patients.handle_input(event)? {
                    return Ok(Some(action));
                }
            }
            HospitalState::Staff => {
                if let Some(action) = self.staff.handle_input(event)? {
                    return Ok(Some(action));
                }
            }
            HospitalState::Records => {
                if let Some(action) = self.records.handle_input(event)? {
                    return Ok(Some(action));
                }
            }
        }
        Ok(None)
    }

    fn render(&self, frame: &mut Frame) {
        match self.state {
            HospitalState::Finance => self.finance.render(frame),
            HospitalState::Patients => self.patients.render(frame),
            HospitalState::Staff => self.staff.render(frame),
            HospitalState::Records => self.records.render(frame),
        }
    }
}
