//! Main entry point for the Rustoria application.

mod app;
mod auth;
mod components;
mod db;
mod models;
mod tui;
mod utils;

use anyhow::Result;
use app::App;
use crossterm::{
    event::{DisableMouseCapture},
    terminal::{self, LeaveAlternateScreen},
};
use ratatui::prelude::{CrosstermBackend, Terminal};
use tui::Tui;
use std::io;

/// The main function of the Rustoria application.
fn main() -> Result<()> {
    // Create cleanup guard to ensure terminal is restored on panic
    let _guard = CleanupGuard;

    // Initialize the database
    db::init_db()?;

    // Initialize the terminal
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    terminal.clear()?;

    // Setup the TUI
    let mut tui = Tui::new(terminal);
    tui.init()?;

    // Create the application and run the main loop
    let mut app = App::new();
    let res = app.run(&mut tui);

    // Restore the terminal after the application is closed
    tui.exit()?;

    // Handle any errors that occurred during the application run
    if let Err(e) = res {
        eprintln!("Application Error: {e}");
    }
    Ok(())
}

// Safety guard to ensure terminal is restored even if the app panics
struct CleanupGuard;

impl Drop for CleanupGuard {
    fn drop(&mut self) {
        // Ignore errors during cleanup
        let _ = terminal::disable_raw_mode();
        let _ = crossterm::execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
    }
}