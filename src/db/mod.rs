//! Database module for Rustoria.
//!
//! This module provides functionality for interacting with the application's database.
//! It includes functions for initializing the database, managing users,
//! patients, and staff members. It encapsulates all database-related logic,
//! ensuring data persistence and retrieval. The primary types exposed are functions
//! for interacting with the database, such as `init_db`, `authenticate_user`, and
//! `create_patient`.

use crate::models::{Gender, MedicalRecord, Patient, StaffMember, StaffRole};
use anyhow::{anyhow, Context, Result};
use bcrypt::{hash, verify, DEFAULT_COST};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;
use time::{format_description, Date};

/// The name of the database file.
///
/// This constant defines the filename of the SQLite database used by the application.
/// It is used by functions in this module to open and interact with the database.
const DB_NAME: &str = "rustoria.db";

/// Initializes the database.
///
/// This function creates the database file if it doesn't exist and executes the schema
/// defined in `schema.sql` to set up the necessary tables. It also creates a default
/// "root" user if one doesn't already exist, using "root" as the default password.
///
/// # Errors
///
/// Returns an error if the database cannot be opened, the schema cannot be executed,
/// or the default user cannot be created. The errors are wrapped in `anyhow::Result`
/// for easy error handling.
///
/// # Side Effects
///
/// Creates a new database file if one doesn't exist.  Executes SQL commands to
/// create tables and insert data.
///
/// # Postconditions
///
/// The database file exists, and the necessary tables are created. If a "root"
/// user does not exist, it will be created with a default password.
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
/// This function verifies the provided username and password against the stored credentials
/// in the database. It retrieves the user's hashed password and uses `bcrypt` to compare
/// it with the provided password.
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
///
/// # Errors
///
/// Returns an error if the database cannot be opened, the user is not found, or the
/// password verification fails.
///
/// # Postconditions
///
/// Returns the user's ID if authentication is successful.
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
        Err(anyhow!("Invalid credentials"))
    }
}

/// Creates a new user in the database.
///
/// This function creates a new user with the given username and password.
/// The password will be hashed using bcrypt before storing it in the database
/// to ensure security.
///
/// # Arguments
///
/// * `username` - The username for the new user.
/// * `password` - The password for the new user.
///
/// # Errors
///
/// Returns an error if the database cannot be opened, the password hashing fails,
/// or the user creation fails.
///
/// # Side Effects
///
/// Adds a new user record to the `users` table in the database.
///
/// # Postconditions
///
/// A new user is created in the database with the provided username and a hashed version
/// of the password.
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
/// This function queries the database for a user with the specified ID and returns their username.
/// It's useful for displaying user-related information based on the user's ID.
///
/// # Arguments
///
/// * `user_id` - The ID of the user.
///
/// # Returns
///
/// Returns `Ok(String)` with the username if found.
/// Returns an error if the user ID is not found or if there is a database issue.
///
/// # Errors
///
/// Returns an error if the database cannot be opened, the user ID is not found, or
/// there is an issue querying the database.
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
/// This function inserts a new patient record into the `patients` table in the database.
/// It takes a `Patient` struct as input and extracts the relevant data to populate the
/// database record.
///
/// # Arguments
///
/// * `patient` - A reference to the `Patient` struct containing the patient's information.
///
/// # Errors
///
/// Returns an error if the database cannot be opened or the patient record cannot be created.
///
/// # Side Effects
///
/// Adds a new patient record to the `patients` table in the database.
///
/// # Postconditions
///
/// A new patient record is created in the database with the information provided in the
/// `patient` struct.
pub fn create_patient(patient: &Patient) -> Result<()> {
    let conn = Connection::open(DB_NAME)?;
    conn.execute(
        "INSERT INTO patients (first_name, last_name, date_of_birth, gender, address, phone_number, email, medical_history, allergies, current_medications) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        params![
            patient.first_name,
            patient.last_name,
            patient.date_of_birth,
            match patient.gender {
                Gender::Male => "Male",
                Gender::Female => "Female",
                Gender::Other => "Other",
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
/// This function queries the database for all patient records and returns them as a vector
/// of `Patient` structs.
///
/// # Returns
///
/// Returns `Ok(Vec<Patient>)` with a vector of `Patient` structs containing the information
/// for all patients in the database. Returns an error if there is an issue querying the
/// database or mapping the results to `Patient` structs.
///
/// # Errors
///
/// Returns an error if the database cannot be opened or the patient records cannot be retrieved.
pub fn get_all_patients() -> Result<Vec<Patient>> {
    let conn = Connection::open(DB_NAME)?;
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
                _ => {
                    return Err(rusqlite::Error::InvalidColumnType(
                        4,
                        String::from("Invalid gender value"),
                        rusqlite::types::Type::Text,
                    ))
                }
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

/// Retrieves a single patient by their ID.
///
/// This function queries the database for a patient with the specified ID and returns their information
/// as a `Patient` struct.
///
/// # Arguments
///
/// * `patient_id` - The ID of the patient to retrieve.
///
/// # Returns
///
/// Returns `Ok(Patient)` with the patient's information if found. Returns an error if the patient
/// is not found or if there is an issue querying the database.
///
/// # Errors
///
/// Returns an error if the database cannot be opened, the patient is not found, or there is an issue
/// querying the database.
pub fn get_patient(patient_id: i64) -> Result<Patient> {
    let conn = Connection::open(DB_NAME)?;
    let mut stmt = conn.prepare("SELECT id, first_name, last_name, date_of_birth, gender, address, phone_number, email, medical_history, allergies, current_medications FROM patients WHERE id = ?")?;

    let patient: Option<Patient> = stmt
        .query_row(params![patient_id], |row| {
            Ok(Patient {
                id: row.get(0)?,
                first_name: row.get(1)?,
                last_name: row.get(2)?,
                date_of_birth: row.get(3)?,
                gender: match &*row.get::<_, String>(4)? {
                    "Male" => Gender::Male,
                    "Female" => Gender::Female,
                    "Other" => Gender::Other,
                    _ => {
                        return Err(rusqlite::Error::InvalidColumnType(
                            4,
                            String::from("Invalid gender value"),
                            rusqlite::types::Type::Text,
                        ))
                    }
                },
                address: row.get(5)?,
                phone_number: row.get(6)?,
                email: row.get(7)?,
                medical_history: row.get(8)?,
                allergies: row.get(9)?,
                current_medications: row.get(10)?,
            })
        })
        .optional()?;

    patient.ok_or_else(|| anyhow!("Patient not found")) // Use the custom error
}

/// Updates an existing patient in the database.
///
/// This function updates the information for an existing patient in the `patients` table.
/// It takes a `Patient` struct as input and updates the corresponding record in the database
/// based on the patient's ID.
///
/// # Arguments
///
/// * `patient` - A reference to the `Patient` struct containing the updated patient information.
///
/// # Errors
///
/// Returns an error if the database cannot be opened or the patient record cannot be updated.
///
/// # Side Effects
///
/// Updates an existing patient record in the `patients` table in the database.
///
/// # Postconditions
///
/// The patient record in the database is updated with the information provided in the
/// `patient` struct.
pub fn update_patient(patient: &Patient) -> Result<()> {
    let conn = Connection::open(DB_NAME)?;
    conn.execute(
        "UPDATE patients SET first_name = ?, last_name = ?, date_of_birth = ?, gender = ?, address = ?, phone_number = ?, email = ?, medical_history = ?, allergies = ?, current_medications = ? WHERE id = ?",
        params![
            patient.first_name,
            patient.last_name,
            patient.date_of_birth,
           match patient.gender {
                Gender::Male => "Male",
                Gender::Female => "Female",
                Gender::Other => "Other",
            },
            patient.address,
            patient.phone_number,
            patient.email,
            patient.medical_history,
            patient.allergies,
            patient.current_medications,
            patient.id,
        ],
    )?;
    Ok(())
}

/// Deletes a patient record from the database by their ID.
///
/// This function deletes a patient record from the `patients` table in the database based on the
/// provided patient ID.
///
/// # Arguments
///
/// * `patient_id` - The ID of the patient to delete.
///
/// # Errors
///
/// Returns an error if the database cannot be opened or the patient record cannot be deleted.
///
/// # Side Effects
///
/// Deletes a patient record from the `patients` table in the database.
///
/// # Postconditions
///
/// The patient record with the specified ID is removed from the database.
pub fn delete_patient(patient_id: i64) -> Result<()> {
    let conn = Connection::open(DB_NAME)?;
    conn.execute("DELETE FROM patients WHERE id = ?", params![patient_id])?;
    Ok(())
}

/// Creates a new staff member in the database.
///
/// This function inserts a new staff member record into the `staff` table in the database.
/// It takes a `StaffMember` struct as input and extracts the relevant data to populate the
/// database record.
///
/// # Arguments
///
/// * `staff_member` - A reference to the `StaffMember` struct containing the staff member's information.
///
/// # Errors
///
/// Returns an error if the database cannot be opened or the staff member record cannot be created.
///
/// # Side Effects
///
/// Adds a new staff member record to the `staff` table in the database.
///
/// # Postconditions
///
/// A new staff member record is created in the database with the information provided in the
/// `staff_member` struct.
pub fn create_staff_member(staff_member: &StaffMember) -> Result<()> {
    let conn = Connection::open(DB_NAME)?;
    conn.execute(
        "INSERT INTO staff (name, role, phone_number, email, address) VALUES (?, ?, ?, ?, ?)",
        params![
            staff_member.name,
            match staff_member.role {
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

/// Retrieves all staff members from the database.
///
/// This function queries the database for all staff member records and returns them as a vector
/// of `StaffMember` structs.
///
/// # Returns
///
/// Returns `Ok(Vec<StaffMember>)` with a vector of `StaffMember` structs containing the information
/// for all staff members in the database. Returns an error if there is an issue querying the
/// database or mapping the results to `StaffMember` structs.
///
/// # Errors
///
/// Returns an error if the database cannot be opened or the staff member records cannot be retrieved.
pub fn get_all_staff() -> Result<Vec<StaffMember>> {
    let conn = Connection::open(DB_NAME)?;
    let mut stmt =
        conn.prepare("SELECT id, name, role, phone_number, email, address FROM staff")?;
    let staff_iter = stmt.query_map([], |row| {
        Ok(StaffMember {
            id: row.get(0)?,
            name: row.get(1)?,
            role: match row.get::<_, String>(2)?.as_str() {
                "Doctor" => StaffRole::Doctor,
                "Nurse" => StaffRole::Nurse,
                "Admin" => StaffRole::Admin,
                "Technician" => StaffRole::Technician,
                _ => {
                    return Err(rusqlite::Error::InvalidColumnType(
                        // Correct error handling
                        2, // Index of the 'role' column
                        String::from("Invalid role value"),
                        rusqlite::types::Type::Text,
                    ));
                }
            },
            phone_number: row.get(3)?,
            email: row.get(4)?,
            address: row.get(5)?,
        })
    })?;

    let mut staff = Vec::new();
    for staff_member in staff_iter {
        staff.push(staff_member?);
    }
    Ok(staff)
}

/// Retrieves a single staff member by their ID.
///
/// This function queries the database for a staff member with the specified ID and returns their information
/// as a `StaffMember` struct.
///
/// # Arguments
///
/// * `staff_id` - The ID of the staff member to retrieve.
///
/// # Returns
///
/// Returns `Ok(StaffMember)` with the staff member's information if found. Returns an error if the staff
/// member is not found or if there is an issue querying the database.
///
/// # Errors
///
/// Returns an error if the database cannot be opened, the staff member is not found, or there is an issue
/// querying the database.
pub fn get_staff(staff_id: i64) -> Result<StaffMember> {
    let conn = Connection::open(DB_NAME)?;
    let mut stmt = conn
        .prepare("SELECT id, name, role, phone_number, email, address FROM staff WHERE id = ?")?;

    let staff_member: Option<StaffMember> = stmt
        .query_row(params![staff_id], |row| {
            Ok(StaffMember {
                id: row.get(0)?,
                name: row.get(1)?,
                role: match row.get::<_, String>(2)?.as_str() {
                    "Doctor" => StaffRole::Doctor,
                    "Nurse" => StaffRole::Nurse,
                    "Admin" => StaffRole::Admin,
                    "Technician" => StaffRole::Technician,
                    _ => {
                        return Err(rusqlite::Error::InvalidColumnType(
                            // Correct error handling
                            2, // Index of the 'role' column
                            String::from("Invalid role value"),
                            rusqlite::types::Type::Text,
                        ));
                    }
                },
                phone_number: row.get(3)?,
                email: row.get(4)?,
                address: row.get(5)?,
            })
        })
        .optional()?; // Use optional to handle not found

    staff_member.ok_or_else(|| anyhow!("Staff member not found")) // Return a proper error
}

/// Updates an existing staff member in the database.
///
/// This function updates the information for an existing staff member in the `staff` table.
/// It takes a `StaffMember` struct as input and updates the corresponding record in the database
/// based on the staff member's ID.
///
/// # Arguments
///
/// * `staff_member` - A reference to the `StaffMember` struct containing the updated staff member information.
///
/// # Errors
///
/// Returns an error if the database cannot be opened or the staff member record cannot be updated.
///
/// # Side Effects
///
/// Updates an existing staff member record in the `staff` table in the database.
///
/// # Postconditions
///
/// The staff member record in the database is updated with the information provided in the
/// `staff_member` struct.
pub fn update_staff_member(staff_member: &StaffMember) -> Result<()> {
    let conn = Connection::open(DB_NAME)?;
    conn.execute(
        "UPDATE staff SET name = ?, role = ?, phone_number = ?, email = ?, address = ? WHERE id = ?",
        params![
            staff_member.name,
            match staff_member.role {
                StaffRole::Doctor => "Doctor",
                StaffRole::Nurse => "Nurse",
                StaffRole::Admin => "Admin",
                StaffRole::Technician => "Technician",
            },
            staff_member.phone_number,
            staff_member.email,
            staff_member.address,
            staff_member.id,
        ],
    )?;
    Ok(())
}

/// Assigns a shift to a staff member.
///
/// This function assigns a specific shift (Morning, Afternoon, Night) to a staff member
/// on a given date.  It inserts a new record into the `shifts` table of the database.
///
/// # Arguments
///
/// * `staff_id` - The ID of the staff member.
/// * `date` - The date of the shift.
/// * `shift` - The shift string ("Morning", "Afternoon", "Night").
///
/// # Returns
///
/// Returns a `Result` indicating success or an error.
///
/// # Errors
///
/// Returns an error if the database cannot be opened or the shift cannot be assigned.  This
/// includes issues with date formatting or invalid shift values.
///
/// # Side Effects
///
/// Inserts a new record into the `shifts` table.
///
/// # Postconditions
///
/// A new shift record is created in the `shifts` table associating the staff member with the
/// specified date and shift.
pub fn assign_staff_shift(staff_id: i64, date: &Date, shift: &str) -> Result<()> {
    let conn = Connection::open("rustoria.db")?;

    // Convert the `time::Date` to a string format suitable for SQLite
    let date_str = date
        .format(&format_description::parse("[year]-[month]-[day]").unwrap())
        .unwrap();

    conn.execute(
        "INSERT INTO shifts (staff_id, date, shift) VALUES (?, ?, ?)",
        params![staff_id, date_str, shift],
    )?;

    Ok(())
}

/// Deletes a staff member from the database by their ID.
///
/// This function deletes a staff member record from the `staff` table in the database based on the
/// provided staff member ID.
///
/// # Arguments
///
/// * `staff_id` - The ID of the staff member to delete.
///
/// # Errors
///
/// Returns an error if the database cannot be opened or the staff member record cannot be deleted.
///
/// # Side Effects
///
/// Deletes a staff member record from the `staff` table in the database.
///
/// # Postconditions
///
/// The staff member record with the specified ID is removed from the database.
pub fn delete_staff_member(staff_id: i64) -> Result<()> {
    let conn = Connection::open(DB_NAME)?;
    conn.execute("DELETE FROM staff WHERE id = ?", params![staff_id])?;
    Ok(())
}

/// Retrieves all assigned shifts for a staff member.
///
/// This function retrieves all assigned shifts for a given staff member from the `shifts` table.
/// It returns a vector of tuples, where each tuple contains the date and shift for an assigned shift.
///
/// # Arguments
///
/// * `staff_id` - The ID of the staff member.
///
/// # Returns
///
/// Returns a `Result` containing a vector of (Date, shift) tuples.
///
/// # Errors
///
/// Returns an error if the database cannot be opened or the shifts cannot be retrieved.  This includes
/// errors during date parsing from the database.
pub fn get_assigned_shifts_for_staff(staff_id: i64) -> Result<Vec<(Date, String)>> {
    let conn = Connection::open(DB_NAME)?;

    let mut stmt =
        conn.prepare("SELECT date, shift FROM shifts WHERE staff_id = ? ORDER BY date")?;

    let shifts_iter = stmt.query_map(params![staff_id], |row| {
        let date_str: String = row.get(0)?;
        let shift: String = row.get(1)?;

        // Parse the date string using split and conversion
        let parts: Vec<&str> = date_str.split('-').collect();
        if parts.len() != 3 {
            return Err(rusqlite::Error::InvalidColumnType(
                0,
                format!("Invalid date format: {}", date_str),
                rusqlite::types::Type::Text,
            ));
        }

        let year = parts[0].parse::<i32>().map_err(|_| {
            rusqlite::Error::InvalidColumnType(
                0,
                format!("Invalid year: {}", parts[0]),
                rusqlite::types::Type::Text,
            )
        })?;

        let month = parts[1].parse::<u8>().map_err(|_| {
            rusqlite::Error::InvalidColumnType(
                0,
                format!("Invalid month: {}", parts[1]),
                rusqlite::types::Type::Text,
            )
        })?;

        let day = parts[2].parse::<u8>().map_err(|_| {
            rusqlite::Error::InvalidColumnType(
                0,
                format!("Invalid day: {}", parts[2]),
                rusqlite::types::Type::Text,
            )
        })?;

        // Create the Date object
        let month = time::Month::try_from(month).map_err(|_| {
            rusqlite::Error::InvalidColumnType(
                0,
                format!("Invalid month number: {}", month),
                rusqlite::types::Type::Text,
            )
        })?;

        let date = time::Date::from_calendar_date(year, month, day).map_err(|e| {
            rusqlite::Error::InvalidColumnType(
                0,
                format!("Invalid date components: {}", e),
                rusqlite::types::Type::Text,
            )
        })?;

        Ok((date, shift))
    })?;

    // Collect results and handle errors
    let mut shifts = Vec::new();
    for shift in shifts_iter {
        shifts.push(shift?);
    }

    Ok(shifts)
}

/// Creates a new medical record in the database.
///
/// This function creates a new medical record for a patient, including doctor's notes, nurse's notes,
/// diagnosis, and prescription.  It stores this information in the `medical_records` table.
///
/// # Arguments
///
/// * `record` - A reference to a `MedicalRecord` struct containing the record's information.
///
/// # Errors
///
/// Returns an error if the database cannot be opened or the record cannot be inserted.
///
/// # Side Effects
///
/// Inserts a new record into the `medical_records` table.
///
/// # Postconditions
///
/// A new medical record is created in the database.
pub fn create_medical_record(record: &MedicalRecord) -> Result<()> {
    let conn = Connection::open("rustoria.db")?;
    conn.execute(
        "INSERT INTO medical_records (patient_id, doctor_notes, nurse_notes, diagnosis, prescription) VALUES (?, ?, ?, ?, ?)",
        params![
            record.patient_id,
            record.doctor_notes,
            record.nurse_notes,
            record.diagnosis,
            record.prescription
        ],
    )?;
    Ok(())
}

/// Retrieves all medical records from the database.
///
/// This function queries the database for all medical records in the `medical_records` table
/// and returns them as a `Vec<MedicalRecord>`.
///
/// # Returns
///
/// Returns a `Result` containing a vector of `MedicalRecord` structs.
///
/// # Errors
///
/// Returns an error if the database cannot be opened or the records cannot be retrieved.
pub fn get_all_medical_records() -> Result<Vec<MedicalRecord>> {
    let conn = Connection::open("rustoria.db")?;
    let mut stmt = conn.prepare("SELECT * FROM medical_records")?;
    let records = stmt
        .query_map([], |row| {
            Ok(MedicalRecord {
                id: row.get(0)?,
                patient_id: row.get(1)?,
                doctor_notes: row.get(2)?,
                nurse_notes: row.get(3)?,
                diagnosis: row.get(4)?,
                prescription: row.get(5)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?; // Collect into Result<Vec<>, _>
    Ok(records)
}

/// Retrieves a specific medical record by its ID.
///
/// This function queries the database for a single medical record with the specified ID
/// from the `medical_records` table.
///
/// # Arguments
///
/// * `record_id` - The ID of the medical record to retrieve.
///
/// # Returns
///
/// Returns a `Result` containing the `MedicalRecord` struct if found.
///
/// # Errors
///
/// Returns an error if the database cannot be opened, the record is not found, or the
/// retrieval fails.
pub fn get_medical_record(record_id: i64) -> Result<MedicalRecord> {
    let conn = Connection::open("rustoria.db")?;
    let mut stmt = conn.prepare("SELECT * FROM medical_records WHERE id = ?")?;
    let record = stmt.query_row(params![record_id], |row| {
        Ok(MedicalRecord {
            id: row.get(0)?,
            patient_id: row.get(1)?,
            doctor_notes: row.get(2)?,
            nurse_notes: row.get(3)?,
            diagnosis: row.get(4)?,
            prescription: row.get(5)?,
        })
    })?;
    Ok(record)
}

/// Updates an existing medical record in the database.
///
/// This function updates an existing medical record in the `medical_records` table.
///
/// # Arguments
///
/// * `record` - A reference to the `MedicalRecord` struct with the updated information.  The `id` field
///            is used to identify the record to update.
///
/// # Errors
///
/// Returns an error if the database cannot be opened or the update fails.
///
/// # Side Effects
///
/// Updates a row in the `medical_records` table.
///
/// # Postconditions
///
/// The medical record in the database is updated with the information from the provided `record`.
pub fn update_medical_record(record: &MedicalRecord) -> Result<()> {
    let conn = Connection::open("rustoria.db")?;
    conn.execute(
        "UPDATE medical_records SET patient_id = ?, doctor_notes = ?, nurse_notes = ?, diagnosis = ?, prescription = ? WHERE id = ?",
        params![
            record.patient_id,
            record.doctor_notes,
            record.nurse_notes,
            record.diagnosis,
            record.prescription,
            record.id
        ],
    )?;
    Ok(())
}

/// Deletes a medical record from the database.
///
/// This function deletes a specific medical record from the `medical_records` table.
///
/// # Arguments
///
/// * `record_id` - The ID of the medical record to delete.
///
/// # Errors
///
/// Returns an error if the database cannot be opened or the deletion fails.
///
/// # Side Effects
///
/// Deletes a row from the `medical_records` table.
///
/// # Postconditions
///
/// The medical record with the specified `record_id` is removed from the database.
pub fn delete_medical_record(record_id: i64) -> Result<()> {
    let conn = Connection::open("rustoria.db")?;
    conn.execute(
        "DELETE FROM medical_records WHERE id = ?",
        params![record_id],
    )?;
    Ok(())
}
