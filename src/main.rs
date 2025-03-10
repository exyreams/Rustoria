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
    event::DisableMouseCapture,
    terminal::{self, LeaveAlternateScreen},
};
use ratatui::prelude::{CrosstermBackend, Terminal};
use std::io;
use tui::Tui;

fn main() -> Result<()> {
    let _guard = CleanupGuard;

    db::init_db()?;

    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    terminal.clear()?;

    let mut tui = Tui::new(terminal);
    tui.init()?;

    let mut app = App::new();
    let res = app.run(&mut tui);

    tui.exit()?;

    if let Err(e) = res {
        eprintln!("Application Error: {e}");
    }
    Ok(())
}

struct CleanupGuard;

impl Drop for CleanupGuard {
    fn drop(&mut self) {
        // Ignore errors during cleanup
        let _ = terminal::disable_raw_mode();
        let _ = crossterm::execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
    }
}
