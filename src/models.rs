//! Data models for Rustoria.

use serde::{Deserialize, Serialize}; // Import

/// Represents a patient in the hospital management system.
#[derive(Debug, Clone, Serialize, Deserialize)] // Add Serialize, Deserialize
pub enum Gender {
    Male,
    Female,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)] // Add Serialize, Deserialize
pub struct Patient {
    pub id: i64, // Use i64 for database compatibility
    pub first_name: String,
    pub last_name: String,
    pub date_of_birth: String,
    pub gender: Gender,
    pub address: String,
    pub phone_number: String,
    pub email: Option<String>,
    pub medical_history: Option<String>,
    pub allergies: Option<String>,
    pub current_medications: Option<String>,
}

/// Represents a pharmaceutical drug in the inventory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Drug {
    /// The drug's unique ID.
    pub id: u64,
    /// The drug's name.
    pub name: String,
    /// The current stock level.
    pub stock: u32,
}
