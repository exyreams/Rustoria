//! The main application state and logic for Rustoria.

use crate::auth::{login, Credentials};
use crate::components::hospital;
use crate::components::{home::Home, login::Login, register::Register, Component};
use crate::tui::{self, Tui};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

/// Enum representing the different applications within Rustoria.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectedApp {
    PatientAdd,  // Add Patient
    PatientList, // List Patients
    Hospital,    // Keep this for future hospital sub-modules
    None,
    Quit,
}

/// Enum representing the possible states of the application.
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
    pub state: AppState,
    pub should_quit: bool,
    pub home: Home,
    pub login: Login,
    pub register: Register,
    pub hospital: Option<hospital::HospitalApp>, // HospitalApp instance
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
            hospital: None, // Initialize hospital to None
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
                                        // Check credentials and get user ID
                                        let credentials = Credentials {
                                            username: self.login.username.clone(),
                                            password: self.login.password.clone(),
                                        };

                                        match login(credentials) {
                                            Ok(user_id) => {
                                                // Load username after successful login
                                                self.home.load_username(user_id)?; // This will now work
                                                self.state = AppState::Home;
                                            }
                                            Err(err) => {
                                                self.login.error_message = Some(format!("{}", err));
                                            }
                                        }
                                    }
                                    // Go to register page
                                    SelectedApp::Hospital => {
                                        self.state = AppState::Register;
                                    }
                                    // Add the missing patterns:
                                    SelectedApp::PatientAdd | SelectedApp::PatientList => {
                                        // These shouldn't be selectable from login screen, but we need to handle them
                                        // You could show an error message or just ignore them
                                        self.login.error_message =
                                            Some("Please log in first.".to_string());
                                    }
                                }
                            }
                        }
                    }
                    // Register State
                    AppState::Register => {
                        if let crossterm::event::Event::Key(key) = event {
                            if let Some(_selected_app) = self.register.handle_input(key)? {
                                // When SelectedApp::None is returned from Register, go back to Login
                                // This happens both with "Back to Login" and successful registration
                                self.state = AppState::Login;

                                // If registration was successful, show a success message
                                if self.register.registration_success {
                                    self.login.username.clear();
                                    self.login.password.clear();
                                    self.login.error_message = None; // Clear any errors
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
                                    // Modified cases:
                                    SelectedApp::PatientAdd => {
                                        self.hospital = Some(hospital::HospitalApp::new()); // Create
                                        if let Some(hospital) = &mut self.hospital {
                                            hospital.patients.state =
                                                hospital::patients::PatientsState::AddPatient;
                                            // Set AddPatient state
                                        }
                                        self.state = AppState::Running(selected_app);
                                    }
                                    SelectedApp::PatientList => {
                                        self.hospital = Some(hospital::HospitalApp::new()); // Create
                                        if let Some(hospital) = &mut self.hospital {
                                            hospital.patients.state =
                                                hospital::patients::PatientsState::ListPatients; // Set ListPatients state
                                            hospital.patients.initialize_list()?;
                                        }
                                        self.state = AppState::Running(selected_app);
                                    }
                                    SelectedApp::Hospital => {
                                        // Instantiate HospitalApp when switching to it.
                                        self.hospital = Some(hospital::HospitalApp::new());
                                        self.state = AppState::Running(selected_app);
                                    }
                                    SelectedApp::Quit => {
                                        self.should_quit = true;
                                        return Ok(());
                                    }
                                    SelectedApp::None => {
                                        // Back to login
                                        self.state = AppState::Login;
                                    }
                                }
                            }
                        }
                    }
                    // Handle Running state
                    AppState::Running(selected_app) => match selected_app {
                        SelectedApp::PatientAdd | SelectedApp::PatientList => {
                            if let Some(hospital) = &mut self.hospital {
                                if let crossterm::event::Event::Key(key) = event {
                                    if let Some(action) = hospital.handle_input(key)? {
                                        match action {
                                            // Go back to home, and clean up the hospital state.
                                            SelectedApp::None => {
                                                self.state = AppState::Home;
                                                self.hospital = None;
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                            } else {
                                // If hospital somehow wasn't initialized, go back to home
                                self.state = AppState::Home;
                            }
                        }
                        // Add other Running states if we have any
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
            // Corrected rendering for the new states:
            AppState::Running(SelectedApp::PatientAdd)
            | AppState::Running(SelectedApp::PatientList) => {
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
