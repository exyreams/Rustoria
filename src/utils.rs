//! Utility functions for Rustoria.
use anyhow::Result;
use std::io::{self, Write}; // Import Write

/// Flushes the standard output.
///
/// This function is used to ensure that all output is written to the console
/// before the program exits.
pub fn flush_stdout() -> Result<(), io::Error> {
    io::stdout().flush()?; // flush() is now available
    Ok(())
}