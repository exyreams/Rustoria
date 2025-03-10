use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, time::Duration};

#[derive(Debug, Clone)]
pub enum Event {
    Input(event::Event),
    Tick,
}

pub type Frame<'a> = ratatui::Frame<'a>;

pub struct Tui {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    framerate: f64,
}

impl Tui {
    pub fn new(terminal: Terminal<CrosstermBackend<io::Stdout>>) -> Self {
        Self {
            terminal,
            framerate: 30.0,
        }
    }

    pub fn init(&mut self) -> Result<()> {
        terminal::enable_raw_mode()?;
        crossterm::execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
        self.terminal.hide_cursor()?;
        self.terminal.clear()?;
        self.set_min_size(95, 35)?;
        Ok(())
    }

    pub fn set_min_size(&self, width: u16, height: u16) -> Result<()> {
        let mut stdout = io::stdout();
        let (current_width, current_height) = terminal::size()?;

        if current_width < width || current_height < height {
            stdout.execute(crossterm::terminal::SetSize(width, height))?;
        }

        Ok(())
    }

    pub fn exit(&mut self) -> Result<()> {
        self.terminal.show_cursor()?;
        terminal::disable_raw_mode()?;
        crossterm::execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
        Ok(())
    }

    pub fn draw(&mut self, render: impl FnOnce(&mut Frame)) -> Result<()> {
        self.terminal.draw(render)?;
        Ok(())
    }

    pub fn next_event(&self) -> Result<Event> {
        let timeout = Duration::from_secs_f64(1.0 / self.framerate);

        if event::poll(timeout)? {
            return Ok(Event::Input(event::read()?));
        }

        Ok(Event::Tick)
    }
}
