//! The main application state and logic for Rustoria.
//!
//! This module contains the core application logic, managing the different
//! application states, handling user input, and rendering the user interface.
//! It encapsulates the application's state machine, the various screens (login, register, home), and the main event loop.
//! The primary type exposed is the `App` struct, which manages the overall application flow.

use crate::auth::{login, Credentials};
use crate::components::hospital;
use crate::components::hospital::records::delete::DeleteRecord;
use crate::components::hospital::records::update::UpdateRecord;
use crate::components::hospital::records::RecordsState;
use crate::components::hospital::staff::delete::DeleteStaff;
use crate::components::hospital::staff::update::UpdateStaff;
use crate::components::{home::Home, login::Login, register::Register, Component};
use crate::tui::{self, Tui};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

/// Enum representing the different applications/views within Rustoria.
///
/// This enum defines the possible "screens" or applications within the main Rustoria application.
/// Each variant represents a distinct area of functionality (e.g., adding a patient, listing staff).
/// Used for state management and determining which component to render and which input to handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectedApp {
    /// Represents the "Add Patient" view.
    PatientAdd,
    /// Represents the "List Patients" view.
    PatientList,
    /// Represents the "Delete Patient" view.
    PatientDelete,
    /// Represents the "Update Patient" view.
    PatientUpdate,
    /// Represents the "Add Staff" view.
    StaffAdd,
    /// Assign Shift
    StaffAssign,
    /// Represents the "List Staff" view.
    StaffList,
    /// Represents the "Delete Staff" view.
    StaffDelete,
    /// Represents the "Update Staff" view.
    StaffUpdate,
    /// Represents the "Add Staff" view.
    RecordStore,
    /// Represents the "Adding Medical Records" view.
    RecordRetrieve,
    /// Represents the "Listing Medical Records" view.
    RecordUpdate,
    /// Represents the "Deleting Medical Records" view.
    RecordDelete,
    /// Represents the "Hospital" view (which manages Patients and Staff).
    Hospital,
    /// Represents no specific application selection.
    None,
    /// Represents the "Quit" action.
    Quit,
}

/// Enum representing the possible states of the entire application.
///
/// This enum defines the overall state of the Rustoria application.
/// It dictates which screen is active (e.g., Login, Home) and controls the application's behavior.
/// Used by the `App` struct to manage the application's lifecycle and transition between different states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    /// Represents the initial state of the application.
    Init,
    /// Represents the login screen state.
    Login,
    /// Represents the registration screen state.
    Register,
    /// Represents the home screen state (after login).
    Home,
    /// Represents a running state, with a specific application selected.
    Running(SelectedApp),
    /// Represents the state when the application is about to quit.
    #[allow(dead_code)]
    Quitting,
}

/// Main application struct for Rustoria.
///
/// This struct manages the overall application flow, including
/// state transitions, input handling, and UI rendering. It holds the application's state,
/// components (login, register, home, hospital), and the main event loop.
/// The `App` struct orchestrates the interaction between different components and manages the application's lifecycle.
pub struct App {
    /// The current state of the application.
    pub state: AppState,
    /// Flag indicating if the application should quit.
    pub should_quit: bool,
    /// The home screen component.
    pub home: Home,
    /// The login screen component.
    pub login: Login,
    /// The registration screen component.
    pub register: Register,
    /// The Hospital application component (optional, only exists when active).
    pub hospital: Option<hospital::HospitalApp>,
}

impl App {
    /// Creates a new instance of the `App`.
    ///
    /// Initializes the application with its initial state and components.
    /// The initial state is set to `Init`, and the `should_quit` flag is set to `false`.
    ///
    /// # Returns
    ///
    /// A new `App` instance.
    pub fn new() -> Self {
        Self {
            state: AppState::Init,
            should_quit: false,
            home: Home::new(),
            login: Login::new(),
            register: Register::new(),
            hospital: None, // HospitalApp is only created when needed
        }
    }

    /// Runs the application's main loop.
    ///
    /// This function controls the main application flow:
    /// rendering the UI, handling user input, and updating the application state.
    /// The loop continues until the `should_quit` flag is set to `true`.
    ///
    /// # Arguments
    ///
    /// * `tui` - A mutable reference to the `Tui` struct for terminal interaction.
    ///
    /// # Errors
    ///
    /// Returns an error if there is an issue during rendering or input handling.
    pub fn run(&mut self, tui: &mut Tui) -> Result<()> {
        // Set initial state to Login.
        self.state = AppState::Login;

        // Main application loop.
        while !self.should_quit {
            // Draw the UI.
            tui.draw(|frame| self.render_ui(frame))?;
            // Handle user input.
            self.handle_input(tui)?;
        }
        Ok(())
    }

    /// Handles input events and updates application state accordingly.
    ///
    /// This function processes user input from the terminal, such as key presses,
    /// and updates the application's state based on the input and the current state.
    /// It uses a match statement to handle input differently based on the current `AppState`.
    ///
    /// # Arguments
    ///
    /// * `tui` - A mutable reference to the `Tui` struct for terminal interaction.
    ///
    /// # Errors
    ///
    /// Returns an error if there is an issue with input handling.
    fn handle_input(&mut self, tui: &mut Tui) -> Result<()> {
        match tui.next_event()? {
            tui::Event::Input(event) => {
                // Global keybinding: Ctrl+Q to quit
                if let crossterm::event::Event::Key(KeyEvent {
                    code: KeyCode::Char('q'),
                    modifiers: crossterm::event::KeyModifiers::CONTROL,
                    ..
                }) = event
                {
                    self.should_quit = true;
                    return Ok(());
                }

                // State-specific input handling.
                match self.state {
                    AppState::Init => {
                        // Transition to Login state.
                        self.state = AppState::Login;
                    }
                    AppState::Login => {
                        // Handle input on the login screen.
                        if let crossterm::event::Event::Key(key) = event {
                            // Pass the key event to the login component
                            let result = self.login.handle_input(key)?;
                            if let Some(selected_app) = result {
                                match selected_app {
                                    SelectedApp::Quit => {
                                        self.should_quit = true;
                                        return Ok(());
                                    }
                                    SelectedApp::None => {
                                        // Attempt to log in
                                        let credentials = Credentials {
                                            username: self.login.username.clone(),
                                            password: self.login.password.clone(),
                                        };

                                        // Call the login function to attempt authentication
                                        match login(credentials) {
                                            Ok(user_id) => {
                                                // Successful login: load username, switch to Home state
                                                self.home.load_username(user_id)?;
                                                self.state = AppState::Home;
                                            }
                                            Err(err) => {
                                                // Login failed: display error message
                                                self.login.error_message = Some(format!("{}", err));
                                            }
                                        }
                                    }
                                    SelectedApp::Hospital => {
                                        // Go to register page (from login screen)
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
                                    | SelectedApp::RecordDelete => {
                                        // These shouldn't be selectable from login screen, but we need to handle them
                                        // You could show an error message or just ignore them
                                        self.login.error_message =
                                            Some("Please log in first.".to_string());
                                    }
                                }
                            }
                        }
                    }
                    AppState::Register => {
                        // Handle input on the registration screen.
                        if let crossterm::event::Event::Key(key) = event {
                            if let Some(_selected_app) = self.register.handle_input(key)? {
                                // Successful registration or "Back to Login"
                                self.state = AppState::Login;

                                // If registration was successful, show a message.
                                if self.register.registration_success {
                                    self.login.username.clear();
                                    self.login.password.clear();
                                    self.login.error_message = None; // Clear errors
                                    self.login.set_success_message(
                                        "Registration successful! Please log in.".to_string(),
                                    );
                                }
                            }
                        }
                    }

                    AppState::Home => {
                        // Handle input on the home screen.
                        if let crossterm::event::Event::Key(key) = event {
                            if let Some(selected_app) = self.home.handle_input(key)? {
                                match selected_app {
                                    SelectedApp::PatientAdd => {
                                        // Initialize HospitalApp with AddPatient state
                                        self.hospital = Some(hospital::HospitalApp::new());
                                        if let Some(hospital) = &mut self.hospital {
                                            hospital.set_patients_state(
                                                hospital::patients::PatientsState::AddPatient,
                                            );
                                        }
                                        self.state = AppState::Running(selected_app);
                                    }
                                    SelectedApp::PatientList => {
                                        // Initialize HospitalApp with ListPatients state
                                        self.hospital = Some(hospital::HospitalApp::new());
                                        if let Some(hospital) = &mut self.hospital {
                                            hospital.set_patients_state(
                                                hospital::patients::PatientsState::ListPatients,
                                            );
                                        }
                                        self.state = AppState::Running(selected_app);
                                    }
                                    SelectedApp::PatientDelete => {
                                        // Initialize HospitalApp with DeletePatient state
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
                                        // Initialize HospitalApp with UpdatePatient state
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
                                        // Initialize HospitalApp with AddStaff state
                                        self.hospital = Some(hospital::HospitalApp::new());
                                        if let Some(hospital) = &mut self.hospital {
                                            hospital.set_state(hospital::HospitalState::Staff);
                                            // Use StaffState instead of HospitalState
                                            hospital.set_staff_state(
                                                hospital::staff::StaffState::AddStaff,
                                            );
                                        }
                                        self.state = AppState::Running(selected_app);
                                    }
                                    SelectedApp::StaffAssign => {
                                        // Initialize HospitalApp, set state, AND fetch staff data.
                                        self.hospital = Some(hospital::HospitalApp::new()); // Create if it doesn't exist.
                                        if let Some(hospital) = &mut self.hospital {
                                            hospital.set_state(hospital::HospitalState::Staff);
                                            hospital.set_staff_state(crate::components::hospital::staff::StaffState::AssignStaff);

                                            // Initialize AssignStaff if it hasn't been initialized yet.
                                            if hospital.staff.assign_staff.is_none() {
                                                let mut assign_staff = crate::components::hospital::staff::assign::AssignStaff::new();
                                                assign_staff.fetch_staff()?; // Crucial: Load the staff data!
                                                hospital.staff.assign_staff = Some(assign_staff);
                                            }
                                        }
                                        self.state = AppState::Running(selected_app);
                                        // Correct place
                                    }
                                    SelectedApp::StaffList => {
                                        // Initialize HospitalApp with ListStaff state
                                        self.hospital = Some(hospital::HospitalApp::new());
                                        if let Some(hospital) = &mut self.hospital {
                                            hospital.set_state(hospital::HospitalState::Staff);
                                            // Use StaffState instead of HospitalState
                                            hospital.set_staff_state(
                                                hospital::staff::StaffState::ListStaff,
                                            );
                                        }
                                        self.state = AppState::Running(selected_app);
                                    }

                                    SelectedApp::StaffUpdate => {
                                        // Initialize, set state, AND fetch staff data
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
                                        // Initialize, set state, AND fetch staff data
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
                                                delete_staff.fetch_staff()?; // <--- KEY CHANGE
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

                                    SelectedApp::Hospital => {
                                        // Instantiate HospitalApp (if needed) and switch to it.
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
                        // Handle input for running applications
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
                        | SelectedApp::RecordDelete => {
                            // Handle input in the Hospital component
                            if let Some(hospital) = &mut self.hospital {
                                if let crossterm::event::Event::Key(key) = event {
                                    if let Some(action) = hospital.handle_input(key)? {
                                        match action {
                                            SelectedApp::None => {
                                                // Go back to Home, and clean up the hospital state.
                                                self.state = AppState::Home;
                                                self.hospital = None; // Drop the HospitalApp
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                            } else {
                                // If hospital somehow wasn't initialized, go back to Home
                                self.state = AppState::Home;
                            }
                        }
                        SelectedApp::StaffAssign => {
                            if let Some(hospital) = &mut self.hospital {
                                // We *don't* change the state here.  We stay in Running(StaffAssign)
                                // until the *hospital* component tells us to go back to Home.
                                if let crossterm::event::Event::Key(key_event) = event {
                                    if let Some(selected_app) = hospital.handle_input(key_event)? {
                                        match selected_app {
                                            SelectedApp::None => {
                                                self.state = AppState::Home;
                                                self.hospital = None;
                                            }
                                            _ => {} // Handle other actions if necessary
                                        }
                                    }
                                }
                            }
                        }
                        _ => {
                            // For other Running states (if any), default back to Home
                            self.state = AppState::Home;
                        }
                    },

                    AppState::Quitting => {
                        self.should_quit = true;
                    }
                }
            }
            tui::Event::Tick => {
                // Perform periodic updates, e.g., checking timeouts
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

    /// Renders the UI based on the current application state.
    ///
    /// This function calls the `render` method of the active component
    /// to draw the UI elements to the terminal. The specific component rendered depends on the current `AppState`.
    ///
    /// # Arguments
    ///
    /// * `frame` - A mutable reference to the `Frame` for rendering.
    fn render_ui(&self, frame: &mut crate::tui::Frame<'_>) {
        match self.state {
            AppState::Init => {} // Nothing to render in Init state
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
            | AppState::Running(SelectedApp::RecordDelete) => {
                // Render the HospitalApp (which will render its sub-components)
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
