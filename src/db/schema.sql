-- Database schema for Rustoria.

-- Create the users table.
-- The `users` table stores user credentials for authentication and authorization purposes.
-- Each user has a unique username and a password hash for secure authentication.
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY, -- Unique identifier for the user.
    username TEXT NOT NULL UNIQUE, -- Username for login. Must be unique.
    password_hash TEXT NOT NULL -- Hashed password for security.
);

-- Create the patients table.
-- The `patients` table stores detailed information about each patient.
-- This includes personal details, contact information, and medical history.
CREATE TABLE IF NOT EXISTS patients (
   id INTEGER PRIMARY KEY, -- Unique identifier for the patient.
   first_name TEXT NOT NULL, -- Patient's first name.
   last_name TEXT NOT NULL, -- Patient's last name.
   date_of_birth TEXT NOT NULL, -- Patient's date of birth.
   gender TEXT NOT NULL, -- Patient's gender.
   address TEXT NOT NULL, -- Patient's address.
   phone_number TEXT NOT NULL, -- Patient's phone number.
   email TEXT, -- Patient's email address (optional).
   medical_history TEXT, -- Patient's medical history.
   allergies TEXT, -- Patient's allergies.
   current_medications TEXT -- Patient's current medications.
);

-- Create the staff table
-- The `shifts` table stores shift data of each staff member.
-- This table is designed to record the schedules of staff, linking each shift to a specific staff member and date.
CREATE TABLE IF NOT EXISTS shifts (
    id INTEGER PRIMARY KEY AUTOINCREMENT, -- Unique identifier for the shift. Automatically increments.
    staff_id INTEGER NOT NULL, -- Foreign key referencing the `staff` table.
    date TEXT NOT NULL, -- Date of the shift.
    shift TEXT NOT NULL,  -- Morning shift, Evening shift etc.
    FOREIGN KEY (staff_id) REFERENCES staff(id) -- Establishes a foreign key relationship with the `staff` table.
);

-- Create the medical_records table
-- The `medical_records` table stores patient medical information.
-- Contains records of doctor's notes, diagnoses, prescriptions, and nurse's notes (if available) for each patient.
CREATE TABLE IF NOT EXISTS medical_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT, -- Unique identifier for the record. Automatically increments.
    patient_id INTEGER NOT NULL, -- Foreign key referencing the `patients` table.
    doctor_notes TEXT NOT NULL, -- Notes from the doctor.
    nurse_notes TEXT, -- Notes from the nurse (optional).
    diagnosis TEXT NOT NULL, -- The diagnosis.
    prescription TEXT, -- Prescribed medications.
    FOREIGN KEY (patient_id) REFERENCES patients(id) -- Establishes a foreign key relationship with the `patients` table.
);