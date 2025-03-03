//! Terminal user interface (TUI) setup and management for Rustoria.

use anyhow::{Context, Result};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand, // Add this import
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, time::Duration};

/// Custom event type that includes both input events and "tick" events
#[derive(Debug, Clone)]
pub enum Event {
    Input(event::Event),
    Tick,
}

/// Type alias for a [`ratatui::Frame`] with a [`crossterm`] backend.
pub type Frame<'a> = ratatui::Frame<'a>;

/// The Terminal User Interface (TUI) for the application.
pub struct Tui {
    /// The terminal instance.
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    /// The framerate of the application which dictates how often the UI is redrawn.
    framerate: f64,
}

impl Tui {
    /// Creates a new `Tui` instance.
    pub fn new(terminal: Terminal<CrosstermBackend<io::Stdout>>) -> Self {
        Self {
            terminal,
            framerate: 30.0,
        }
    }

    /// Initializes the terminal.
    pub fn init(&mut self) -> Result<()> {
        terminal::enable_raw_mode()?;
        crossterm::execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
        self.terminal.hide_cursor()?;
        self.terminal.clear()?;
        self.set_min_size(95, 35)?; // Example: 95x35 minimum size
        Ok(())
    }

    /// Sets the minimum terminal size.
    pub fn set_min_size(&self, width: u16, height: u16) -> Result<()> {
        let mut stdout = io::stdout();
        let (current_width, current_height) = terminal::size()?;

        if current_width < width || current_height < height {
          stdout.execute(crossterm::terminal::SetSize(width, height))?;
        }
		
        Ok(())
    }

    /// Exits the terminal.
    pub fn exit(&mut self) -> Result<()> {
        self.terminal.show_cursor()?;
        terminal::disable_raw_mode()?;
        crossterm::execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
        Ok(())
    }

    /// Draws the UI to the terminal.
    pub fn draw(&mut self, render: impl FnOnce(&mut Frame)) -> Result<()> {
        self.terminal.draw(render)?;
        Ok(())
    }

    /// Reads the next input event from the terminal with a timeout.
    pub fn next_event(&self) -> Result<Event> {
        let timeout = Duration::from_secs_f64(1.0 / self.framerate);

        if event::poll(timeout)? {
            return Ok(Event::Input(event::read()?));
        }

        Ok(Event::Tick)
    }
}