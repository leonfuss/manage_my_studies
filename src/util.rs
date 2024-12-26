use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use toml_edit::DocumentMut;

pub fn toml_from_path(path: &PathBuf) -> Result<DocumentMut> {
    let content = std::fs::read_to_string(path)
        .with_context(|| anyhow!("Failed to read file at: {}", path.display()))?;
    let toml = content
        .parse::<DocumentMut>()
        .with_context(|| format!("Failed to parse the file at: '{}'", path.display(),))?;
    Ok(toml)
}

pub fn enusure_file_exists(path: &PathBuf) -> Result<()> {
    if !path.exists() {
        std::fs::write(path, "")
            .with_context(|| anyhow!("Failed to create data file at: {}", path.display()))?;
    }
    Ok(())
}
