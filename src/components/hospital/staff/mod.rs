//! Staff management module within the Hospital application.

use self::add::AddStaff;
use crate::components::Component;
use crate::tui::Frame;
use anyhow::Result;
use crossterm::event::KeyEvent;

pub mod add;
// pub mod list;  // Future modules
// pub mod assign_shift;
// pub mod remove;

/// Represents the overall state of the Staff section.  For now, it just
/// contains the AddStaff component.  Later, it will manage switching
/// between different staff-related views.
pub struct Staff {
    add_staff: AddStaff, // Component for adding staff
}

impl Staff {
    pub fn new() -> Self {
        Self {
            add_staff: AddStaff::new(),
        }
    }
}

impl Component for Staff {
    fn handle_input(&mut self, event: KeyEvent) -> Result<Option<crate::app::SelectedApp>> {
        // Delegate input handling to the currently active component (AddStaff for now)
        if let Some(action) = self.add_staff.handle_input(event)? {
            return Ok(Some(action)); // Return the action
        }
        Ok(None)
    }

    fn render(&self, frame: &mut Frame) {
        // Render the currently active component (AddStaff for now)
        self.add_staff.render(frame);
    }
}
