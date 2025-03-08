//! Medical Records management module for the Hospital application.
//!
//! This module provides functionalities for managing medical records, including
//! storing, retrieving, updating, and deleting records. It encapsulates the
//! different states and actions related to medical records management, ensuring
//! data integrity and efficient access to patient information. The module interacts
//! with the database layer to persist and retrieve record data.
//!
//! The primary type exposed by this module is `Records`, which encapsulates the
//! state and behavior of the medical records management component.

use crate::app::SelectedApp;
use crate::components::Component;
use crate::tui::Frame;
use anyhow::Result;
use crossterm::event::KeyEvent;

pub mod delete;
pub mod retrive;
pub mod store;
pub mod update;

/// Represents the different states of the medical records management component.
///
/// `RecordsState` defines the possible states the `Records` component can be in,
/// such as adding, retrieving, deleting, or updating medical records. The state
/// determines which sub-component is active and which UI is displayed. It's
/// used to manage the flow of operations within the Records component.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordsState {
    /// State for adding a new medical record.
    StoreRecord,
    /// State for listing medical records.
    RetrieveRecords,
    /// State for deleting a medical record.
    DeleteRecord,
    /// State for updating medical record information.
    UpdateRecord,
}

/// Manages the medical records-related functionalities within the hospital application.
///
/// The `Records` struct encapsulates the sub-components responsible for storing,
/// retrieving, deleting, and updating medical records. It manages the state transitions
/// between these sub-components and orchestrates the overall medical records
/// management workflow. It holds the current state of the Records component and
/// handles user input to transition between states.
pub struct Records {
    /// The component for adding a new medical record.
    pub store_record: store::StoreRecord,
    /// The component for listing medical records.
    pub retrieve_records: retrive::RetrieveRecords,
    /// The component for deleting a medical record, wrapped in `Option`.
    ///
    /// The `Option` wrapper indicates that the `DeleteRecord` component is only
    /// initialized when the user enters the `DeleteRecord` state. This allows for
    /// lazy initialization and reduces memory usage when not in use.
    pub delete_record: Option<delete::DeleteRecord>,
    /// The component for updating medical record, wrapped in `Option`.
    ///
    /// The `Option` wrapper indicates that the `UpdateRecord` component is only
    /// initialized when the user enters the `UpdateRecord` state. This allows for
    /// lazy initialization and reduces memory usage when not in use.
    pub update_record: Option<update::UpdateRecord>,
    /// The current state of the medical records management component.
    pub state: RecordsState,
}

impl Records {
    /// Creates a new `Records` component with the necessary subcomponents.
    ///
    /// This function initializes the `Records` component with its sub-components
    /// for storing, retrieving, deleting, and updating medical records. It sets
    /// the initial state to `RetrieveRecords` and fetches the initial list of
    /// records to be displayed to the user.
    ///
    /// # Returns
    ///
    /// A new instance of `Records`.
    pub fn new() -> Self {
        Self {
            store_record: store::StoreRecord::new(),
            retrieve_records: retrive::RetrieveRecords::new(),
            delete_record: None,
            update_record: None,
            state: RecordsState::RetrieveRecords, // Start in RetrieveRecords state
        }
    }

    /// Initializes the medical record list and loads patients for validation.
    ///
    /// This method is called when the component is first displayed or when
    /// the record list needs to be refreshed. It ensures that the record list
    /// is up-to-date and that the patient list is loaded for ID validation
    /// during record creation or update.
    ///
    /// # Errors
    ///
    /// Returns an error if fetching records or loading patients fails.
    pub fn initialize_list(&mut self) -> Result<()> {
        if self.state == RecordsState::RetrieveRecords {
            self.retrieve_records.fetch_records()?;
        }
        self.store_record.load_patients()?; // Load patients for ID validation.
        Ok(())
    }
}

impl Component for Records {
    /// Handles key input events for the medical records management component.
    ///
    /// Processes key events based on the current `RecordsState` and delegates input
    /// handling to the appropriate sub-component (store, retrieve, delete, update).
    /// Manages transitions between states as needed, updating the `state` field
    /// and initializing/deinitializing sub-components as necessary.
    ///
    /// # Parameters
    ///
    /// * `event`: The `KeyEvent` to handle.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(SelectedApp))`: If the event triggers a transition to another application.
    /// * `Ok(None)`: If the event is handled within the medical records component.
    /// * `Err(anyhow::Error)`: If an error occurs during input handling.
    fn handle_input(&mut self, event: KeyEvent) -> Result<Option<SelectedApp>> {
        match self.state {
            RecordsState::StoreRecord => {
                if let Some(selected_app) = self.store_record.handle_input(event)? {
                    match selected_app {
                        SelectedApp::None => {
                            self.state = RecordsState::RetrieveRecords;
                            self.initialize_list()?;
                            return Ok(Some(SelectedApp::None));
                        }
                        _ => {}
                    }
                }
            }
            RecordsState::RetrieveRecords => {
                if let Some(selected_app) = self.retrieve_records.handle_input(event)? {
                    match selected_app {
                        SelectedApp::None => return Ok(Some(SelectedApp::None)),
                        _ => {}
                    }
                }
            }
            RecordsState::DeleteRecord => {
                // Check if delete_record is initialized.
                if let Some(delete_record) = &mut self.delete_record {
                    if let Some(selected_app) = delete_record.handle_input(event)? {
                        match selected_app {
                            SelectedApp::None => {
                                // Transition back to RetrieveRecords and deinitialize delete_record.
                                self.state = RecordsState::RetrieveRecords;
                                self.delete_record = None;
                                self.initialize_list()?;
                                return Ok(Some(SelectedApp::None));
                            }
                            _ => {}
                        }
                    }
                }
            }
            RecordsState::UpdateRecord => {
                if let Some(update_record) = &mut self.update_record {
                    if let Some(selected_app) = update_record.handle_input(event)? {
                        match selected_app {
                            SelectedApp::None => {
                                self.state = RecordsState::RetrieveRecords;
                                self.update_record = None;
                                self.initialize_list()?;
                                return Ok(Some(SelectedApp::None));
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        Ok(None)
    }

    /// Renders the medical records management component to the terminal frame.
    ///
    /// Based on the current `RecordsState`, this function calls the `render` method of the
    /// appropriate sub-component (store, retrieve, delete, update) to display the UI.
    /// This ensures that the correct UI for the current state is presented to the user.
    ///
    /// # Parameters
    ///
    /// * `frame`: The `Frame` to render to.
    fn render(&self, frame: &mut Frame) {
        match self.state {
            RecordsState::StoreRecord => self.store_record.render(frame),
            RecordsState::RetrieveRecords => self.retrieve_records.render(frame),
            RecordsState::DeleteRecord => {
                // Safely unwrap the delete_record option before rendering.
                if let Some(delete_record) = &self.delete_record {
                    delete_record.render(frame);
                }
            }
            RecordsState::UpdateRecord => {
                // Safely unwrap the update_record option before rendering.
                if let Some(update_record) = &self.update_record {
                    update_record.render(frame);
                }
            }
        }
    }
}

impl Default for Records {
    /// Creates a default `Records` component.
    ///
    /// This implementation simply calls the `new` function to create a new
    /// `Records` instance with default values.
    fn default() -> Self {
        Self::new()
    }
}
