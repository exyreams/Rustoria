use crate::db;
use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

pub fn login(credentials: Credentials) -> Result<i64> {
    db::authenticate_user(&credentials.username, &credentials.password)
        .context("⚠️ Authentication failed")
}
