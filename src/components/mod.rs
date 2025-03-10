use crate::tui::Frame;
use anyhow::Result;
use crossterm::event::KeyEvent;

pub mod home;
pub mod hospital;
pub mod login;
pub mod register;

pub trait Component {
    fn handle_input(&mut self, event: KeyEvent) -> Result<Option<crate::app::SelectedApp>>; // Modified return
    fn render(&self, frame: &mut Frame);
}
