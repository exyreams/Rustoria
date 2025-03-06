//! Data models for Rustoria.
//!
//! This module defines the core data structures used in the Rustoria hospital management system.
//! It includes entities such as patients, with serialization support for database operations
//! and API communications.

use serde::{Deserialize, Serialize}; // Import

/// Represents the gender identity of a patient.
///
/// This enum is used for demographic information and may be relevant
/// for certain medical treatments or procedures.
#[derive(Debug, Clone, Serialize, Deserialize)] // Add Serialize, Deserialize
pub enum Gender {
    /// Identifies as male
    Male,
    /// Identifies as female
    Female,
    /// Identifies as a gender not covered by the other options
    Other,
}

/// Represents a patient in the hospital management system.
///
/// Contains personal information, contact details, and basic medical information
/// necessary for patient identification and care.
#[derive(Debug, Clone, Serialize, Deserialize)] // Add Serialize, Deserialize
pub struct Patient {
    /// Unique identifier for the patient, compatible with database systems
    pub id: i64, // Use i64 for database compatibility
    /// Patient's first/given name
    pub first_name: String,
    /// Patient's last/family name
    pub last_name: String,
    /// Patient's date of birth in string format (recommended format: YYYY-MM-DD)
    pub date_of_birth: String,
    /// Patient's gender identity
    pub gender: Gender,
    /// Patient's residential address
    pub address: String,
    /// Patient's contact phone number
    pub phone_number: String,
    /// Patient's email address, if available
    pub email: Option<String>,
    /// Summary of patient's relevant medical history
    pub medical_history: Option<String>,
    /// List of patient's known allergies
    pub allergies: Option<String>,
    /// Medications the patient is currently taking
    pub current_medications: Option<String>,
}
