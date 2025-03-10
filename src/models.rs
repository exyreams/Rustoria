use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Gender {
    Male,
    Female,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Patient {
    pub id: i64,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StaffRole {
    Doctor,
    Nurse,
    Admin,
    Technician,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaffMember {
    pub id: i64,
    pub name: String,
    pub role: StaffRole,
    pub phone_number: String,
    pub email: Option<String>,
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedicalRecord {
    pub id: i64,
    pub patient_id: i64,
    pub doctor_notes: String,
    pub nurse_notes: Option<String>,
    pub diagnosis: String,
    pub prescription: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    pub id: i64,
    pub patient_id: i64,
    pub item: String,
    pub quantity: i32,
    pub cost: f64,
}
