//! Database module for Rustoria.
//!
//! This module provides functionality for interacting with the application's database.
//! It includes functions for initializing the database, managing users,
//! patients, and staff members.

use crate::models::{Gender, Patient, StaffMember, StaffRole};
use anyhow::{Context, Result};
use bcrypt::{hash, verify, DEFAULT_COST};
use rusqlite::{params, Connection};
use std::path::Path;

/// The name of the database file.
const DB_NAME: &str = "rustoria.db";

/// Initializes the database.
///
/// This function creates the database file if it doesn't exist and
/// executes the schema defined in `schema.sql`. It also creates a
/// default "root" user if one doesn't already exist.
///
/// # Errors
///
/// Returns an error if the database cannot be opened, the schema
/// cannot be executed, or the default user cannot be created.
pub fn init_db() -> Result<()> {
    // Open or create the database file
    let db_path = Path::new(DB_NAME);
    let conn = Connection::open(db_path)?;

    // Read the schema from the schema.sql file
    let schema = include_str!("schema.sql");

    // Execute the schema to create the tables
    conn.execute_batch(schema)
        .context("Failed to execute schema")?;

    // Check if the admin user already exists
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM users WHERE username = ?")?;
    let count: i64 = stmt.query_row(params!["root"], |row| row.get(0))?;

    // If the "root" user doesn't exist, create it with a default password
    if count == 0 {
        // Hash the default password
        let hashed_password = hash("root", DEFAULT_COST).context("Failed to hash password")?;
        // Insert the new user into the database
        conn.execute(
            "INSERT INTO users (username, password_hash) VALUES (?, ?)",
            params!["root", hashed_password],
        )?;
        println!("Created 'root' user with default password.");
    }

    Ok(())
}

/// Authenticates a user against the database.
///
/// This function verifies the provided username and password
/// against the stored credentials in the database.
///
/// # Arguments
///
/// * `username` - The username to authenticate.
/// * `password` - The password to authenticate.
///
/// # Returns
///
/// Returns `Ok(i64)` with the user ID if authentication is successful.
/// Returns an error if the username or password is invalid.
pub fn authenticate_user(username: &str, password: &str) -> Result<i64> {
    // Open the database connection
    let db_path = Path::new(DB_NAME);
    let conn = Connection::open(db_path)?;

    // Prepare and execute the SQL query to fetch the user's data
    let mut stmt = conn.prepare("SELECT id, password_hash FROM users WHERE username = ?")?;
    let (user_id, stored_hash): (i64, String) =
        stmt.query_row(params![username], |row| Ok((row.get(0)?, row.get(1)?)))?;

    // Verify the provided password against the stored hash
    if verify(password, &stored_hash).context("Failed to verify password")? {
        Ok(user_id)
    } else {
        Err(anyhow::anyhow!("Invalid credentials"))
    }
}

/// Creates a new user in the database.
///
/// This function creates a new user with the given username and password.
/// The password will be hashed before storing it.
///
/// # Arguments
///
/// * `username` - The username for the new user.
/// * `password` - The password for the new user.
///
/// # Errors
///
/// Returns an error if the database cannot be opened or the user cannot
/// be created.
pub fn create_user(username: &str, password: &str) -> Result<()> {
    // Open the database connection
    let db_path = Path::new(DB_NAME);
    let conn = Connection::open(db_path)?;

    // Hash the password *before* storing it!
    let hashed_password = hash(password, DEFAULT_COST).context("Failed to hash password")?;

    // Execute the SQL query to insert the new user
    conn.execute(
        "INSERT INTO users (username, password_hash) VALUES (?, ?)",
        params![username, hashed_password], // Use the *hashed* password
    )?;

    Ok(())
}

/// Retrieves a username from the database given a user ID.
///
/// # Arguments
///
/// * `user_id` - The ID of the user.
///
/// # Returns
///
/// Returns `Ok(String)` with the username if found.
/// Returns an error if the user ID is not found or if there is a database issue.
pub fn get_username(user_id: i64) -> Result<String> {
    // Open the database connection
    let db_path = Path::new(DB_NAME);
    let conn = Connection::open(db_path)?;

    // Prepare and execute the SQL query to retrieve the username
    let mut stmt = conn.prepare("SELECT username FROM users WHERE id = ?")?;
    let username: String = stmt.query_row(params![user_id], |row| row.get(0))?;

    Ok(username)
}

/// Creates a new patient in the database.
///
/// # Arguments
///
/// * `patient` - A reference to the `Patient` struct containing patient data.
///
/// # Errors
///
/// Returns an error if the database operation fails.
pub fn create_patient(patient: &Patient) -> Result<()> {
    // Open the database connection
    let db_path = Path::new(DB_NAME);
    let conn = Connection::open(db_path)?;

    // Execute the SQL query to insert the new patient
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
///
/// # Returns
///
/// Returns a `Result<Vec<Patient>>` containing a vector of `Patient` structs
/// if successful, or an error if the database operation fails.
pub fn get_all_patients() -> Result<Vec<Patient>> {
    // Open the database connection
    let db_path = Path::new(DB_NAME);
    let conn = Connection::open(db_path)?;

    // Prepare the SQL query to select all patients
    let mut stmt = conn.prepare("SELECT id, first_name, last_name, date_of_birth, gender, address, phone_number, email, medical_history, allergies, current_medications FROM patients")?;
    // Execute the query and map the results to Patient structs
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

    // Collect the patient data into a vector
    let mut patients = Vec::new();
    for patient_result in patient_iter {
        patients.push(patient_result?);
    }

    Ok(patients)
}

/// Deletes a patient from the database by ID.
///
/// # Arguments
///
/// * `patient_id` - The ID of the patient to delete.
///
/// # Errors
///
/// Returns an error if the database operation fails.
pub fn delete_patient(patient_id: i64) -> Result<()> {
    // Open the database connection
    let db_path = Path::new(DB_NAME);
    let conn = Connection::open(db_path)?;

    // Execute the SQL query to delete the patient
    conn.execute("DELETE FROM patients WHERE id = ?", params![patient_id])?;

    Ok(())
}

/// Updates an existing patient in the database.
///
/// # Arguments
///
/// * `patient` - A reference to the `Patient` struct containing the updated patient data.
///
/// # Errors
///
/// Returns an error if the database operation fails.
pub fn update_patient(patient: &Patient) -> Result<()> {
    // Open the database connection
    let db_path = Path::new(DB_NAME);
    let conn = Connection::open(db_path)?;

    // Execute the SQL query to update the patient
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
///
/// # Arguments
///
/// * `patient_id` - The ID of the patient to retrieve.
///
/// # Returns
///
/// Returns a `Result<Patient>` containing the `Patient` struct if found,
/// or an error if the patient is not found or the database operation fails.
pub fn get_patient(patient_id: i64) -> Result<Patient> {
    // Open the database connection
    let db_path = Path::new(DB_NAME);
    let conn = Connection::open(db_path)?;

    // Prepare the SQL query to select the patient by ID
    let mut stmt = conn.prepare("SELECT id, first_name, last_name, date_of_birth, gender, address, phone_number, email, medical_history, allergies, current_medications FROM patients WHERE id = ?")?;
    // Execute the query and map the result to a Patient struct
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

/// Creates a new staff member in the database.
///
/// # Arguments
///
/// * `staff_member` - A reference to the `StaffMember` struct containing the staff member data.
///
/// # Errors
///
/// Returns an error if the database operation fails.
pub fn create_staff_member(staff_member: &StaffMember) -> Result<()> {
    // Open the database connection
    let db_path = Path::new(DB_NAME);
    let conn = Connection::open(db_path)?;

    // Execute the SQL query to insert the new staff member
    conn.execute(
        "INSERT INTO staff (name, role, phone_number, email, address) VALUES (?, ?, ?, ?, ?)",
        params![
            staff_member.name,
            match staff_member.role {
                // Convert StaffRole to String for database storage
                StaffRole::Doctor => "Doctor",
                StaffRole::Nurse => "Nurse",
                StaffRole::Admin => "Admin",
                StaffRole::Technician => "Technician",
            },
            staff_member.phone_number,
            staff_member.email,
            staff_member.address,
        ],
    )?;

    Ok(())
}

/// Retrieves all staff member records from the database.
///
/// # Returns
///
/// Returns a `Result<Vec<StaffMember>>` containing a vector of `StaffMember` structs
/// if successful, or an error if the database operation fails.
pub fn get_all_staff() -> Result<Vec<StaffMember>> {
    // Open the database connection
    let conn = Connection::open("./rustoria.db")?;

    // Prepare the SQL query to select all staff members
    let mut stmt = conn.prepare(
        "SELECT id, name, role, phone_number, email, address
         FROM staff",
    )?;

    // Execute the query and map the results to StaffMember structs
    let staff = stmt
        .query_map([], |row| {
            Ok(StaffMember {
                id: row.get(0)?,
                name: row.get(1)?,
                role: match row.get::<_, String>(2)?.as_str() {
                    "Doctor" => StaffRole::Doctor,
                    "Nurse" => StaffRole::Nurse,
                    "Admin" => StaffRole::Admin,
                    "Technician" => StaffRole::Technician,
                    _ => StaffRole::Doctor, // Or perhaps a default, or handle the error
                },
                phone_number: row.get(3)?,
                email: row.get(4)?,
                address: row.get(5)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(staff)
}
