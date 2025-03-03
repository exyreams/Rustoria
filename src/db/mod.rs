//! Database module for Rustoria.

use anyhow::{Context, Result};
use bcrypt::{hash, verify, DEFAULT_COST};
use rusqlite::{params, Connection};
use std::path::Path;

/// The name of the database file.
const DB_NAME: &str = "rustoria.db";

/// Initializes the database.
/// Creates the database file if it doesn't exist and applies the schema.
pub fn init_db() -> Result<()> {
    let db_path = Path::new(DB_NAME);
    let conn = Connection::open(db_path)?;

    // Read the schema from the schema.sql file
    let schema = include_str!("schema.sql");

    // Execute the schema
    conn.execute_batch(schema)
        .context("Failed to execute schema")?;

    // Check if the admin user already exists (same as before)
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM users WHERE username = ?")?;
    let count: i64 = stmt.query_row(params!["root"], |row| row.get(0))?;

    if count == 0 {
        let hashed_password = hash("root", DEFAULT_COST).context("Failed to hash password")?;
        conn.execute(
            "INSERT INTO users (username, password_hash) VALUES (?, ?)",
            params!["root", hashed_password],
        )?;
        println!("Created 'root' user with default password.");
    }

    Ok(())
}

/// Authenticates a user against the database.
pub fn authenticate_user(username: &str, password: &str) -> Result<i64> {
    let db_path = Path::new(DB_NAME);
    let conn = Connection::open(db_path)?;

    let mut stmt = conn.prepare("SELECT id, password_hash FROM users WHERE username = ?")?;
    let (user_id, stored_hash): (i64, String) =
        stmt.query_row(params![username], |row| Ok((row.get(0)?, row.get(1)?)))?;

    if verify(password, &stored_hash).context("Failed to verify password")? {
        Ok(user_id)
    } else {
        Err(anyhow::anyhow!("Invalid credentials"))
    }
}

/// Creates a new user in the database.
pub fn create_user(username: &str, password: &str) -> Result<()> {
    let db_path = Path::new(DB_NAME);
    let conn = Connection::open(db_path)?;

    // Hash the password *before* storing it!
    let hashed_password = hash(password, DEFAULT_COST).context("Failed to hash password")?;

    conn.execute(
        "INSERT INTO users (username, password_hash) VALUES (?, ?)",
        params![username, hashed_password], // Use the *hashed* password
    )?;

    Ok(())
}

/// Retrieves a username from the database given a user ID.
pub fn get_username(user_id: i64) -> Result<String> {
    let db_path = Path::new(DB_NAME);
    let conn = Connection::open(db_path)?;

    let mut stmt = conn.prepare("SELECT username FROM users WHERE id = ?")?;
    let username: String = stmt.query_row(params![user_id], |row| row.get(0))?;

    Ok(username)
}
