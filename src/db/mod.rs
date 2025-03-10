use crate::models::{Gender, Invoice, MedicalRecord, Patient, StaffMember, StaffRole};
use anyhow::{anyhow, Context, Result};
use bcrypt::{hash, verify, DEFAULT_COST};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;
use time::{format_description, Date};

const DB_NAME: &str = "rustoria.db";

pub fn init_db() -> Result<()> {
    let db_path = Path::new(DB_NAME);
    let conn = Connection::open(db_path)?;

    let schema = include_str!("schema.sql");

    conn.execute_batch(schema)
        .context("Failed to execute schema")?;

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

pub fn authenticate_user(username: &str, password: &str) -> Result<i64> {
    let db_path = Path::new(DB_NAME);
    let conn = Connection::open(db_path)?;

    let mut stmt = conn.prepare("SELECT id, password_hash FROM users WHERE username = ?")?;
    let (user_id, stored_hash): (i64, String) =
        stmt.query_row(params![username], |row| Ok((row.get(0)?, row.get(1)?)))?;

    if verify(password, &stored_hash).context("Failed to verify password")? {
        Ok(user_id)
    } else {
        Err(anyhow!("Invalid credentials"))
    }
}

pub fn create_user(username: &str, password: &str) -> Result<()> {
    let db_path = Path::new(DB_NAME);
    let conn = Connection::open(db_path)?;

    let hashed_password = hash(password, DEFAULT_COST).context("Failed to hash password")?;

    conn.execute(
        "INSERT INTO users (username, password_hash) VALUES (?, ?)",
        params![username, hashed_password],
    )?;

    Ok(())
}

pub fn get_username(user_id: i64) -> Result<String> {
    let db_path = Path::new(DB_NAME);
    let conn = Connection::open(db_path)?;

    let mut stmt = conn.prepare("SELECT username FROM users WHERE id = ?")?;
    let username: String = stmt.query_row(params![user_id], |row| row.get(0))?;

    Ok(username)
}
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

    patient.ok_or_else(|| anyhow!("Patient not found"))
}

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

pub fn delete_patient(patient_id: i64) -> Result<()> {
    let conn = Connection::open(DB_NAME)?;
    conn.execute("DELETE FROM patients WHERE id = ?", params![patient_id])?;
    Ok(())
}

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
                        2,
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
                            2,
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
        .optional()?;

    staff_member.ok_or_else(|| anyhow!("Staff member not found"))
}

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

pub fn assign_staff_shift(staff_id: i64, date: &Date, shift: &str) -> Result<()> {
    let conn = Connection::open("rustoria.db")?;

    let date_str = date
        .format(&format_description::parse("[year]-[month]-[day]").unwrap())
        .unwrap();

    conn.execute(
        "INSERT INTO shifts (staff_id, date, shift) VALUES (?, ?, ?)",
        params![staff_id, date_str, shift],
    )?;

    Ok(())
}

pub fn delete_staff_member(staff_id: i64) -> Result<()> {
    let conn = Connection::open(DB_NAME)?;
    conn.execute("DELETE FROM staff WHERE id = ?", params![staff_id])?;
    Ok(())
}

pub fn get_assigned_shifts_for_staff(staff_id: i64) -> Result<Vec<(Date, String)>> {
    let conn = Connection::open(DB_NAME)?;

    let mut stmt =
        conn.prepare("SELECT date, shift FROM shifts WHERE staff_id = ? ORDER BY date")?;

    let shifts_iter = stmt.query_map(params![staff_id], |row| {
        let date_str: String = row.get(0)?;
        let shift: String = row.get(1)?;

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

    let mut shifts = Vec::new();
    for shift in shifts_iter {
        shifts.push(shift?);
    }

    Ok(shifts)
}

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
        .collect::<Result<Vec<_>, _>>()?;
    Ok(records)
}

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

pub fn delete_medical_record(record_id: i64) -> Result<()> {
    let conn = Connection::open("rustoria.db")?;
    conn.execute(
        "DELETE FROM medical_records WHERE id = ?",
        params![record_id],
    )?;
    Ok(())
}

pub fn create_invoice(invoice: &Invoice) -> Result<i64> {
    let conn = Connection::open("rustoria.db")?;
    let mut stmt = conn.prepare(
        "INSERT INTO invoices (patient_id, item, quantity, cost)
        VALUES (?, ?, ?, ?)",
    )?;
    stmt.execute((
        &invoice.patient_id,
        &invoice.item,
        &invoice.quantity,
        &invoice.cost,
    ))?;
    Ok(conn.last_insert_rowid())
}

pub fn get_invoice(id: i64) -> Result<Invoice> {
    let conn = Connection::open("rustoria.db")?;
    let mut stmt = conn.prepare("SELECT * FROM invoices WHERE id = ?")?;
    let invoice = stmt.query_row([id], |row| {
        Ok(Invoice {
            id: row.get(0)?,
            patient_id: row.get(1)?,
            item: row.get(2)?,
            quantity: row.get(3)?,
            cost: row.get(4)?,
        })
    })?;
    Ok(invoice)
}

pub fn get_all_invoices() -> Result<Vec<Invoice>> {
    let conn = Connection::open("rustoria.db")?;
    let mut stmt = conn.prepare("SELECT * FROM invoices")?;
    let invoices = stmt
        .query_map([], |row| {
            Ok(Invoice {
                id: row.get(0)?,
                patient_id: row.get(1)?,
                item: row.get(2)?,
                quantity: row.get(3)?,
                cost: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(invoices)
}

pub fn update_invoice(invoice: &Invoice) -> Result<()> {
    let conn = Connection::open("rustoria.db")?;
    conn.execute(
        "UPDATE invoices SET patient_id = ?, item = ?, quantity = ?, cost = ? 
         WHERE id = ?",
        (
            &invoice.patient_id,
            &invoice.item,
            &invoice.quantity,
            &invoice.cost,
            &invoice.id,
        ),
    )?;
    Ok(())
}
