//! The main application state and logic for Rustoria.

use crate::auth::{login, Credentials};
use crate::components::{home::Home, login::Login, Component};
use crate::tui::{self, Tui};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

/// Enum representing the different applications within Rustoria.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectedApp {
    Hospital,
    Pharmacy,
    None,
    Quit,
}

/// Enum representing the possible states of the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    Init,
    //Initial state
    Login,
    // User login process
    Home,
    // Main application home page
    Running(SelectedApp), // Running a specific sub-application
    Quitting,             // Application is exiting
}

/// Main application struct for Rustoria.
///
/// This struct holds the overall state of the application, including
/// the current application state, UI components, and any shared data.
pub struct App {
    /// Current state of the application
    pub state: AppState,
    /// Flag indicating whether the application should quit
    pub should_quit: bool,
    /// Home page component
    pub home: Home,
    /// Login component
    pub login: Login,
}

impl App {
    /// Creates a new instance of the `App`.
    pub fn new() -> Self {
        Self {
            state: AppState::Init, // Start in the Init state
            should_quit: false,
            home: Home::new(),
            login: Login::new(),
        }
    }

    /// Runs the main application loop.
    ///
    /// This function handles input events, updates the application state,
    /// and renders the UI until the application is quit.
    pub fn run(&mut self, tui: &mut Tui) -> Result<()> {
        // Start in login state.
        self.state = AppState::Login;

        while !self.should_quit {
            // Render the UI
            tui.draw(|frame| self.render_ui(frame))?;

            // Handle input events
            self.handle_input(tui)?;
        }
        Ok(())
    }

    /// Handles crossterm input events and updates the application state accordingly.
    fn handle_input(&mut self, tui: &mut Tui) -> Result<()> {
        match tui.next_event()? {
            // Handle input events
            tui::Event::Input(event) => {
                // Global keybindings that work in any state
                if let crossterm::event::Event::Key(KeyEvent {
                    code: KeyCode::Char('q'),
                    modifiers: crossterm::event::KeyModifiers::CONTROL,
                    ..
                }) = event
                {
                    self.should_quit = true;
                    return Ok(());
                }

                // Delegate event handling to the appropriate component based on app state
                match self.state {
                    AppState::Init => {
                        // Initial state. Go straight to login.
                        self.state = AppState::Login;
                    }

                    AppState::Login => {
                        if let crossterm::event::Event::Key(key) = event {
                            let result = self.login.handle_input(key)?;

                            if let Some(selected_app) = result {
                                match selected_app {
                                    // Handle exit confirmation
                                    SelectedApp::Quit => {
                                        self.should_quit = true;
                                        return Ok(());
                                    }
                                    // Handle login attempt (any other variant signals login)
                                    _ => {
                                        // Login validation is already done in login component
                                        let credentials = Credentials {
                                            username: self.login.username.clone(),
                                            password: self.login.password.clone(),
                                        };

                                        match login(credentials) {
                                            Ok(_) => {
                                                self.state = AppState::Home; // Success
                                            }
                                            Err(err) => {
                                                // Show login error message
                                                self.login.error_message = Some(format!("{}", err));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    AppState::Home => {
                        if let crossterm::event::Event::Key(key) = event {
                            match self.home.handle_input(key)? {
                                Some(selected_app) => match selected_app {
                                    SelectedApp::Hospital | SelectedApp::Pharmacy => {
                                        self.state = AppState::Running(selected_app);
                                    }
                                    SelectedApp::Quit => {
                                        self.should_quit = true;
                                        return Ok(());
                                    }
                                    SelectedApp::None => {
                                        // Handle None case, e.g., do nothing
                                    }
                                },
                                None => {
                                    // No action needed, input was consumed without state change
                                }
                            }
                        }
                    }

                    AppState::Running(selected_app) => {
                        // Delegate to the specific application (Hospital or Pharmacy)
                        match selected_app {
                            SelectedApp::Hospital => {
                                // self.hospital_app.handle_input(event)?;
                                if let crossterm::event::Event::Key(KeyEvent {
                                    code: KeyCode::Esc,
                                    ..
                                }) = event
                                {
                                    self.state = AppState::Home; // Go back to home
                                }
                            }
                            SelectedApp::Pharmacy => {
                                // self.pharmacy_app.handle_input(event)?;
                                if let crossterm::event::Event::Key(KeyEvent {
                                    code: KeyCode::Esc,
                                    ..
                                }) = event
                                {
                                    self.state = AppState::Home; // Go back to home
                                }
                            }
                            _ => {
                                // This case shouldn't occur in Running state
                                self.state = AppState::Home; // Reset to home
                            }
                        }
                    }

                    AppState::Quitting => {
                        self.should_quit = true;
                    }
                }
            }

            // Handle tick events - for animations, timeouts, etc.
            tui::Event::Tick => {
                // Update components that need regular updates
                // For example, update error message timeouts in login component
                if let AppState::Login = self.state {
                    self.login.check_error_timeout();
                }
            }
        }

        Ok(())
    }

    /// Renders the UI based on the current application state.
    fn render_ui(&self, frame: &mut crate::tui::Frame<'_>) {
        match self.state {
            AppState::Init => { /* Render nothing for now*/ }
            AppState::Login => self.login.render(frame),
            AppState::Home => self.home.render(frame),
            AppState::Running(_) => {
                // Render the selected application's UI
                match self.state {
                    AppState::Running(SelectedApp::Hospital) => {
                        // Render hospital app UI
                        // frame.render_widget( ... hospital UI elements ... );
                    }
                    AppState::Running(SelectedApp::Pharmacy) => {
                        // Render pharmacy app UI
                        // frame.render_widget( ... pharmacy UI elements ... );
                    }
                    _ => {} // Do nothing for other Running states (shouldn't happen)
                }
            }
            AppState::Quitting => {
                // Maybe show a "Goodbye" message.
            }
        }
    }
}
