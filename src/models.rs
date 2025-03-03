//! Data models for Rustoria.

/// Represents a patient in the hospital management system.
#[derive(Debug, Clone)]
pub struct Patient {
    /// The patient's unique ID.
    pub id: u64,
    /// The patient's name.
    pub name: String,
    /// The patient's age.
    pub age: u8,
    // Add other relevant fields, e.g., condition, room number, etc.
}

/// Represents a pharmaceutical drug in the inventory.
#[derive(Debug, Clone)]
pub struct Drug {
    /// The drug's unique ID.
    pub id: u64,
    /// The drug's name.
    pub name: String,
    /// The current stock level.
    pub stock: u32,
    // Add other relevant fields, e.g., supplier, expiry date, etc.
}
