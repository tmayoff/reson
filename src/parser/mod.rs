mod tokens;

use anyhow::Result;
use std::{fs::read_to_string, path::PathBuf};

pub fn parse_file(path: &PathBuf) -> Result<()> {
    let content = read_to_string(path)?;

    parse(&content)?;

    Ok(())
}

fn parse(input: &str) -> Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {}
