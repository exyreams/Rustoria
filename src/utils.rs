use anyhow::Result;
use std::io::{self, Write};

#[allow(dead_code)]
pub fn flush_stdout() -> Result<(), io::Error> {
    io::stdout().flush()?;
    Ok(())
}
