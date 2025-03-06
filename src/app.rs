//! The main application state and logic for Rustoria.

use crate::auth::{login, Credentials};
use crate::components::hospital;
use crate::components::{home::Home, login::Login, register::Register, Component};
use crate::tui::{self, Tui};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

/// Enum representing the different applications/views within Rustoria.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectedApp {
    PatientAdd,
    PatientList,
    PatientDelete,
    PatientUpdate,
    StaffAdd, // Add StaffAdd
    Hospital,
    None,
    Quit,
}

/// Enum representing the possible states of the entire application.
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

/// Main application struct for Rustoria.
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
    pub fn run(&mut self, tui: &mut Tui) -> Result<()> {
        self.state = AppState::Login;

        while !self.should_quit {
            tui.draw(|frame| self.render_ui(frame))?;
            self.handle_input(tui)?;
        }
        Ok(())
    }

    /// Handles input events and updates application state accordingly.
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
                                        // Attempt to log in
                                        let credentials = Credentials {
                                            username: self.login.username.clone(),
                                            password: self.login.password.clone(),
                                        };

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
                                    | SelectedApp::StaffAdd => {
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
                                    // Add StaffAdd case:
                                    SelectedApp::StaffAdd => {
                                        self.hospital = Some(hospital::HospitalApp::new()); // Create HospitalApp
                                        if let Some(hospital) = &mut self.hospital {
                                            hospital.set_state(hospital::HospitalState::Staff);
                                            // Set the HospitalState to Staff
                                        }
                                        self.state = AppState::Running(selected_app);
                                        // Set AppState to Running with StaffAdd
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
                        SelectedApp::PatientAdd
                        | SelectedApp::PatientList
                        | SelectedApp::PatientDelete
                        | SelectedApp::PatientUpdate
                        | SelectedApp::StaffAdd => {
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
            | AppState::Running(SelectedApp::StaffAdd) => {
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
