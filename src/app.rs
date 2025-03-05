//! The main application state and logic for Rustoria.
//!
//! This module defines the core application structure, state management,
//! and event handling for the Rustoria TUI application.  It handles
//! navigation between different application components and manages the
//! overall application lifecycle.

use crate::auth::{login, Credentials};
use crate::components::hospital;
use crate::components::{home::Home, login::Login, register::Register, Component};
use crate::tui::{self, Tui};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

/// Enum representing the different applications within Rustoria.
///
/// This enum defines the different selectable applications or views
/// within the Rustoria TUI, dictating which part of the application
/// is currently active or being navigated to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectedApp {
    /// Represents the "Add Patient" functionality.
    PatientAdd,
    /// Represents the "List Patients" functionality.
    PatientList,
    /// Represents the "Delete Patient" functionality.
    PatientDelete,
    /// Represents the Hospital application.
    Hospital,
    /// Represents no specific application being selected.
    None,
    /// Represents the "Quit" application, indicating the application should exit.
    Quit,
}

/// Enum representing the possible states of the application.
///
/// This enum defines the various states the Rustoria application can be in,
/// controlling the flow and UI rendering of the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    /// Initial state of the application.
    Init,
    /// The login screen state.
    Login,
    /// The registration screen state.
    Register,
    /// The home screen state.
    Home,
    /// Represents a running state, with a specified `SelectedApp`.
    Running(SelectedApp),
    /// The application is in the process of quitting.
    #[allow(dead_code)]
    Quitting,
}

/// Main application struct for Rustoria.
///
/// This struct encapsulates the state and components of the entire
/// Rustoria TUI application.  It manages the different application
/// states, handles user input, and renders the UI.
pub struct App {
    /// The current state of the application.
    pub state: AppState,
    /// A boolean flag indicating whether the application should quit.
    pub should_quit: bool,
    /// The home screen component.
    pub home: Home,
    /// The login screen component.
    pub login: Login,
    /// The registration screen component.
    pub register: Register,
    /// An optional instance of the Hospital application.
    pub hospital: Option<hospital::HospitalApp>, // HospitalApp instance
}

impl App {
    /// Creates a new instance of the `App`.
    ///
    /// Initializes the application with its initial state, components,
    /// and settings.
    ///
    /// # Returns
    ///
    /// A new `App` instance.
    pub fn new() -> Self {
        Self {
            // Set initial state to Init
            state: AppState::Init,
            // Set should_quit flag to false
            should_quit: false,
            // Initialize Home component
            home: Home::new(),
            // Initialize Login component
            login: Login::new(),
            // Initialize Register component
            register: Register::new(),
            // Initialize hospital to None, meaning it isn't active yet
            hospital: None, // Initialize hospital to None
        }
    }

    /// Runs the application main loop.
    ///
    /// This method starts the application, sets the initial state to
    /// Login, and then enters the main loop, handling events and
    /// rendering the UI until the application should quit.
    ///
    /// # Arguments
    ///
    /// * `tui`: A mutable reference to the TUI instance.
    ///
    /// # Errors
    ///
    /// Returns an error if any operation within the loop fails.
    pub fn run(&mut self, tui: &mut Tui) -> Result<()> {
        // Set the initial application state to Login
        self.state = AppState::Login;

        // Main application loop
        while !self.should_quit {
            // Draw the UI
            tui.draw(|frame| self.render_ui(frame))?;
            // Handle user input
            self.handle_input(tui)?;
        }
        Ok(())
    }

    /// Handles user input events.
    ///
    /// This method processes user input from the TUI, updating the
    /// application state and interacting with the different components
    /// based on the current state.
    ///
    /// # Arguments
    ///
    /// * `tui`: A mutable reference to the TUI instance.
    ///
    /// # Errors
    ///
    /// Returns an error if any input handling operation fails.
    fn handle_input(&mut self, tui: &mut Tui) -> Result<()> {
        // Get the next event from the TUI
        match tui.next_event()? {
            // Handle input events
            tui::Event::Input(event) => {
                // Handle Ctrl+C to quit the application
                if let crossterm::event::Event::Key(KeyEvent {
                    code: KeyCode::Char('q'),
                    modifiers: crossterm::event::KeyModifiers::CONTROL,
                    ..
                }) = event
                {
                    // Set should_quit to true to exit the loop
                    self.should_quit = true;
                    return Ok(());
                }

                // Handle input based on the current application state
                match self.state {
                    // Handle Init state
                    AppState::Init => {
                        // Transition to Login state
                        self.state = AppState::Login;
                    }
                    // Handle Login state
                    AppState::Login => {
                        // If a key event occurred
                        if let crossterm::event::Event::Key(key) = event {
                            // Handle the key event using the Login component
                            let result = self.login.handle_input(key)?;
                            // If the Login component returns a selected app
                            if let Some(selected_app) = result {
                                // Handle actions based on the selected app
                                match selected_app {
                                    // Quit action
                                    SelectedApp::Quit => {
                                        // Set should_quit to true to exit the loop
                                        self.should_quit = true;
                                        return Ok(());
                                    }
                                    // No specific app selected
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
                                    SelectedApp::PatientAdd
                                    | SelectedApp::PatientList
                                    | SelectedApp::PatientDelete => {
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
                        // Handle key events in the Register state
                        if let crossterm::event::Event::Key(key) = event {
                            // Delegate input handling to the register component
                            if let Some(_selected_app) = self.register.handle_input(key)? {
                                // When SelectedApp::None is returned from Register, go back to Login
                                // This happens both with "Back to Login" and successful registration
                                self.state = AppState::Login;

                                // If registration was successful, show a success message
                                if self.register.registration_success {
                                    // Clear login fields and error messages
                                    self.login.username.clear();
                                    self.login.password.clear();
                                    self.login.error_message = None; // Clear any errors
                                                                     // Set a success message in the login component
                                    self.login.set_success_message(
                                        "Registration successful! Please log in.".to_string(),
                                    );
                                }
                            }
                        }
                    }

                    // Home State
                    AppState::Home => {
                        // Handle key events in the Home state
                        if let crossterm::event::Event::Key(key) = event {
                            // Delegate input handling to the Home component
                            if let Some(selected_app) = self.home.handle_input(key)? {
                                match selected_app {
                                    SelectedApp::PatientAdd => {
                                        // Initialize HospitalApp, set state to AddPatient
                                        self.hospital = Some(hospital::HospitalApp::new());
                                        if let Some(hospital) = &mut self.hospital {
                                            hospital.set_patients_state(
                                                hospital::patients::PatientsState::AddPatient,
                                            );
                                        }
                                        // Transition to Running state with PatientAdd
                                        self.state = AppState::Running(selected_app);
                                    }
                                    SelectedApp::PatientList => {
                                        // Initialize HospitalApp, set state to ListPatients
                                        self.hospital = Some(hospital::HospitalApp::new());
                                        if let Some(hospital) = &mut self.hospital {
                                            hospital.set_patients_state(
                                                hospital::patients::PatientsState::ListPatients,
                                            );
                                        }
                                        // Transition to Running state with PatientList
                                        self.state = AppState::Running(selected_app);
                                    }
                                    // Add PatientDelete case
                                    SelectedApp::PatientDelete => {
                                        // Create new HospitalApp
                                        self.hospital = Some(hospital::HospitalApp::new()); // Create
                                                                                            // Set delete patient state
                                        if let Some(hospital) = &mut self.hospital {
                                            hospital.set_patients_state(
                                                hospital::patients::PatientsState::DeletePatient,
                                            ); // Set DeletePatient state
                                               // Initialize the DeletePatient component
                                            hospital.patients.delete_patient = Some(
                                                hospital::patients::delete::DeletePatient::new(),
                                            ); // Create the component
                                        }
                                        // Transition to Running state with PatientDelete
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
                        SelectedApp::PatientAdd
                        | SelectedApp::PatientList
                        | SelectedApp::PatientDelete => {
                            // Add PatientDelete here
                            // Handle input in the Hospital component
                            if let Some(hospital) = &mut self.hospital {
                                if let crossterm::event::Event::Key(key) = event {
                                    // Delegate input to the hospital component
                                    if let Some(action) = hospital.handle_input(key)? {
                                        match action {
                                            // If the hospital component returns None, go back to Home
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
                        _ => {
                            self.state = AppState::Home;
                        }
                    },
                    AppState::Quitting => {
                        self.should_quit = true;
                    }
                }
            }
            // Handle tick events for periodic updates
            tui::Event::Tick => {
                // Check for error timeouts in the Login state
                if let AppState::Login = self.state {
                    self.login.check_error_timeout();
                }
                // Check for error timeouts in the Register state
                if let AppState::Register = self.state {
                    self.register.check_error_timeout();
                }
            }
        }
        Ok(())
    }

    /// Renders the UI based on the current application state.
    ///
    /// This method calls the `render` method of the appropriate component
    /// based on the current `AppState`.
    ///
    /// # Arguments
    ///
    /// * `frame`: A mutable reference to the TUI frame.
    fn render_ui(&self, frame: &mut crate::tui::Frame<'_>) {
        match self.state {
            // No UI to render in Init state
            AppState::Init => {}
            // Render the Login component
            AppState::Login => self.login.render(frame),
            // Render the Register component
            AppState::Register => self.register.render(frame),
            // Render the Home component
            AppState::Home => self.home.render(frame),
            // Render the Hospital component based on the selected sub-application
            AppState::Running(SelectedApp::PatientAdd)
            | AppState::Running(SelectedApp::PatientList)
            | AppState::Running(SelectedApp::PatientDelete) => {
                // Render the Hospital app if it exists
                if let Some(hospital) = &self.hospital {
                    hospital.render(frame);
                }
            }
            // Render Hospital App if it exists
            AppState::Running(SelectedApp::Hospital) => {
                // Render the Hospital app if it exists
                if let Some(hospital) = &self.hospital {
                    hospital.render(frame);
                }
            }

            AppState::Running(SelectedApp::None) | AppState::Running(SelectedApp::Quit) => todo!(),
            AppState::Quitting => todo!(),
        }
    }
}
