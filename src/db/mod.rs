//! Database module for Rustoria.

use crate::models::{Gender, Patient};
use anyhow::{Context, Result};
use bcrypt::{hash, verify, DEFAULT_COST};
use rusqlite::{params, Connection};
use std::path::Path;

/// The name of the database file.
const DB_NAME: &str = "rustoria.db";

/// Initializes the database.
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

/// Creates a new patient in the database.
pub fn create_patient(patient: &Patient) -> Result<()> {
    let db_path = Path::new(DB_NAME);
    let conn = Connection::open(db_path)?;
    conn.execute(
        "INSERT INTO patients (first_name, last_name, date_of_birth, gender, address, phone_number, email, medical_history, allergies, current_medications) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        params![
            patient.first_name,
            patient.last_name,
            patient.date_of_birth,
            match patient.gender {
                crate::models::Gender::Male => "Male",
                crate::models::Gender::Female => "Female",
                crate::models::Gender::Other => "Other",
            },
            patient.address,
            patient.phone_number,
            patient.email,
            patient.medical_history,
            patient.allergies,
            patient.current_medications,
        ],
    )?;

    Ok(())
}

/// Retrieves all patients from the database.
pub fn get_all_patients() -> Result<Vec<Patient>> {
    let db_path = Path::new(DB_NAME);
    let conn = Connection::open(db_path)?;

    let mut stmt = conn.prepare("SELECT id, first_name, last_name, date_of_birth, gender, address, phone_number, email, medical_history, allergies, current_medications FROM patients")?;
    let patient_iter = stmt.query_map([], |row| {
        Ok(Patient {
            id: row.get(0)?,
            first_name: row.get(1)?,
            last_name: row.get(2)?,
            date_of_birth: row.get(3)?,
            gender: match &*row.get::<_, String>(4)? {
                "Male" => Gender::Male,
                "Female" => Gender::Female,
                "Other" => Gender::Other,
                _ => Gender::Other,
            },
            address: row.get(5)?,
            phone_number: row.get(6)?,
            email: row.get(7)?,
            medical_history: row.get(8)?,
            allergies: row.get(9)?,
            current_medications: row.get(10)?,
        })
    })?;

    let mut patients = Vec::new();
    for patient_result in patient_iter {
        patients.push(patient_result?);
    }

    Ok(patients)
}

/// Deletes a patient from the database by ID.
pub fn delete_patient(patient_id: i64) -> Result<()> {
    let db_path = Path::new(DB_NAME);
    let conn = Connection::open(db_path)?;

    conn.execute("DELETE FROM patients WHERE id = ?", params![patient_id])?;

    Ok(())
}

/// Updates an existing patient in the database.
pub fn update_patient(patient: &Patient) -> Result<()> {
    let db_path = Path::new(DB_NAME);
    let conn = Connection::open(db_path)?;

    conn.execute(
        "UPDATE patients SET first_name = ?, last_name = ?, date_of_birth = ?, gender = ?, address = ?, phone_number = ?, email = ?, medical_history = ?, allergies = ?, current_medications = ? WHERE id = ?",
        params![
            patient.first_name,
            patient.last_name,
            patient.date_of_birth,
            match patient.gender {
                crate::models::Gender::Male => "Male",
                crate::models::Gender::Female => "Female",
                crate::models::Gender::Other => "Other",
            },
            patient.address,
            patient.phone_number,
            patient.email,
            patient.medical_history,
            patient.allergies,
            patient.current_medications,
            patient.id, // The ID is the last parameter for the WHERE clause
        ],
    )?;

    Ok(())
}

/// Retrieves a single patient from the database by ID.
pub fn get_patient(patient_id: i64) -> Result<Patient> {
    let db_path = Path::new(DB_NAME);
    let conn = Connection::open(db_path)?;

    let mut stmt = conn.prepare("SELECT id, first_name, last_name, date_of_birth, gender, address, phone_number, email, medical_history, allergies, current_medications FROM patients WHERE id = ?")?;
    let patient = stmt.query_row(params![patient_id], |row| {
        Ok(Patient {
            id: row.get(0)?,
            first_name: row.get(1)?,
            last_name: row.get(2)?,
            date_of_birth: row.get(3)?,
            gender: match &*row.get::<_, String>(4)? {
                "Male" => Gender::Male,
                "Female" => Gender::Female,
                "Other" => Gender::Other,
                _ => Gender::Other, // Or handle the error appropriately
            },
            address: row.get(5)?,
            phone_number: row.get(6)?,
            email: row.get(7)?,
            medical_history: row.get(8)?,
            allergies: row.get(9)?,
            current_medications: row.get(10)?,
        })
    })?;

    Ok(patient)
}
