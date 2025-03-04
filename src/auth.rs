//! Authentication module for Rustoria.
use crate::db;
use anyhow::{Context, Result};

/// Represents user credentials.
#[derive(Debug, Clone)]
pub struct Credentials {
    /// The username.
    pub username: String,
    /// The password.
    pub password: String,
}

/// Performs the login check using the database.
///
/// # Arguments
///
/// * `credentials`: The user credentials to check.
///
/// # Returns
///
/// A `Result` indicating success or failure.  Returns `Ok(())` if
/// the credentials are valid, otherwise returns an error.
pub fn login(credentials: Credentials) -> Result<i64> {
    db::authenticate_user(&credentials.username, &credentials.password)
        .context("⚠️ Authentication failed")
}
