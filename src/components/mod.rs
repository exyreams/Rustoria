//! UI components for Rustoria.

use crate::tui::Frame;
use anyhow::Result;
use crossterm::event::KeyEvent;

// Re-export the components for easier access
pub mod home;
pub mod login;

/// A trait for UI components that can be rendered and handle input events.
pub trait Component {
    /// Handles input events for the component.
    ///
    /// # Arguments
    ///
    /// * `event`: The input event to handle.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.  If the component has
    /// handled the event and wants to signal a state change (e.g., in Login,
    /// a successful login), it should return `Ok(true)`.  If the event
    /// was handled but no major state change occurred, return `Ok(false)`.
    /// Return an error if a problem occurred.
    fn handle_input(&mut self, event: KeyEvent) -> Result<Option<crate::app::SelectedApp>>; // Modified return

    /// Renders the component to the screen.
    ///
    /// # Arguments
    ///
    /// * `frame`: The frame to render the component on.
    fn render(&self, frame: &mut Frame);
}