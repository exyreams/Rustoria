-- Database schema for Rustoria.

-- Create the users table
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL
);

-- Create the patients table
CREATE TABLE IF NOT EXISTS patients (
   id INTEGER PRIMARY KEY,
   first_name TEXT NOT NULL,
   last_name TEXT NOT NULL,
   date_of_birth TEXT NOT NULL,
   gender TEXT NOT NULL,
   address TEXT NOT NULL,
   phone_number TEXT NOT NULL,
   email TEXT,
   medical_history TEXT,
   allergies TEXT,
   current_medications TEXT
);

-- Create the staff table
CREATE TABLE IF NOT EXISTS shifts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    staff_id INTEGER NOT NULL,
    date TEXT NOT NULL,
    shift TEXT NOT NULL,
    FOREIGN KEY (staff_id) REFERENCES staff(id)
);