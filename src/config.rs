use anyhow::{bail, Context, Result};
use std::path::PathBuf;

pub fn get_entry_point(config_path: &PathBuf) -> Result<PathBuf> {
    let config_content =
        std::fs::read_to_string(config_path).context("Failed to read config file")?;
    let config: toml_edit::DocumentMut = config_content
        .parse::<toml_edit::DocumentMut>()
        .with_context(|| {
            format!(
                "Failed to parse the config file at: '{}'",
                &config_path.display(),
            )
        })?;

    if let Some(entry_point) = config.get("entry_point") {
        let entry_point_path = PathBuf::from(
            entry_point
                .as_str()
                .context("entry_point is not a valid string")?,
        );
        if entry_point_path.is_dir() {
            Ok(entry_point_path)
        } else {
            Err(anyhow::anyhow!(
                "'{}' is not a valid directory.\nPlease set the 'entry_point' in the config file at: '{}' and try again. Make sure you provide an absolute path from the mount point.",
                &entry_point_path.display(),
                &config_path.display()
            ))
        }
    } else {
        Err(anyhow::anyhow!(
            "The key 'entry_point' could not be found in the config file. Please set the 'entry_point' in the config file at: '{}' and try again.",
            &config_path.display()
        ))
    }
}

/// Platform-specific config directory paths
/// Linux: $XDG_CONFIG_HOME or $HOME/.config/mm/config.toml
/// macOS: $HOME/.config/mm/config.toml
/// Windows: {FOLDERID_RoamingAppData}\mm\config.toml
pub fn get_config_path() -> Result<PathBuf> {
    let config_path = get_config_dir()?.join("mm").join("config.toml");

    if config_path.is_file() {
        return Ok(config_path);
    }

    println!(
        "Error: Config file could not be found at: {}",
        &config_path.display()
    );
    let config_dir = get_config_dir()?.join("mm");
    std::fs::create_dir_all(&config_dir)?;
    let config_file_path = config_dir.join("config.toml");
    let config_content = include_str!("../config.toml");
    std::fs::write(&config_file_path, config_content).context("Failed to create config file")?;
    bail!(
        "A new config file has been created at: '{}'. Please set the 'entry_point' in the config file and try again.",
        &config_file_path.display()
    )
}

fn get_config_dir() -> Result<PathBuf> {
    if cfg!(target_os = "macos") {
        let home_dir = dirs::home_dir().context("Failed to find home directory on your system")?;
        Ok(home_dir.join(".config"))
    } else {
        dirs::config_dir().context("Failed to find config directory on your system.")
    }
}
