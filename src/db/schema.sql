-- Database schema for Rustoria.

-- Create the users table.  This table stores user credentials.
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY,  -- Unique user ID (auto-incrementing)
    username TEXT NOT NULL UNIQUE, -- User's chosen username (must be unique)
    password_hash TEXT NOT NULL    -- Hashed password (using bcrypt)
);