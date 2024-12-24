use crate::{course::Course, semester::Semester};
use anyhow::{bail, Context, Result};
use std::path::PathBuf;

pub struct Config {
    entry_point: PathBuf,
    active_semester: Option<String>,
    active_course: Option<String>,
}

impl Config {
    pub fn new() -> Result<Config> {
        let config_path = validate_config_path()?;
        let entry_point = validate_entry_point(&config_path)?;
        let (active_semester, active_course) = validate_current(&entry_point)?;
        Ok(Config {
            entry_point,
            active_semester,
            active_course,
        })
    }

    pub fn entry(&self) -> &PathBuf {
        &self.entry_point
    }

    pub fn active_semester(&self) -> Option<Semester> {
        let path = self.entry_point.join(self.active_semester.as_ref()?);
        self.get_semester(path)
    }

    pub fn active_course(&self) -> Option<Course> {
        let path = self
            .entry()
            .join(self.active_semester.as_ref()?)
            .join(self.active_course.as_ref()?);
        self.get_course(path)
    }

    pub fn set_active_semester(&mut self, semester: &str, remove_course: bool) -> Result<()> {
        self.active_semester = Some(semester).map(|e| e.to_string());
        if remove_course {
            self.set_active_course(None)?;
        }
        self.write()
    }

    pub fn set_active_course(&mut self, course: Option<&str>) -> Result<()> {
        self.active_course = course.map(|e| e.to_string());
        self.write()
    }

    pub fn get_course<P>(&self, path: P) -> Option<Course>
    where
        P: AsRef<std::path::Path>,
    {
        let path = path.as_ref();
        let parent = path.parent()?;
        if self.get_semester(parent).is_none() {
            return None;
        }

        Course::from_path(path.to_path_buf())
    }

    pub fn get_semester<P>(&self, path: P) -> Option<Semester>
    where
        P: AsRef<std::path::Path>,
    {
        let path = path.as_ref();
        let parent = path.parent()?;
        if !path.is_dir() || parent != &self.entry_point {
            return None;
        }
        Semester::from_path(path.to_path_buf())
    }

    fn write(&self) -> Result<()> {
        let data_file = validate_dat_dir(&self.entry_point)?;
        let mut data = toml_edit::DocumentMut::new();

        if let Some(ref semester) = self.active_semester {
            data["semester"] = toml_edit::value(semester.clone());
        }

        if let Some(ref course) = self.active_course {
            data["course"] = toml_edit::value(course.clone());
        }
        std::fs::write(&data_file, data.to_string()).context("Failed to write data file")?;
        Ok(())
    }
}

// Platform-specific config directory paths
// Linux: $XDG_CONFIG_HOME or $HOME/.config
// macOS: $HOME/.config
// Windows: {FOLDERID_RoamingAppData}

fn validate_current(entry_point: &PathBuf) -> Result<(Option<String>, Option<String>)> {
    let data_file = validate_dat_dir(&entry_point)?;

    let data_content = std::fs::read_to_string(&data_file).context("Failed to read data file")?;
    let data: toml_edit::DocumentMut = data_content
        .parse::<toml_edit::DocumentMut>()
        .with_context(|| {
            format!(
                "Failed to parse the data file at: '{}'",
                &data_file.display(),
            )
        })?;

    let semester = data
        .get("semester")
        .and_then(|it| it.as_str().map(String::from));
    let course = data
        .get("course")
        .and_then(|it| it.as_str().map(String::from));
    Ok((semester, course))
}

fn validate_dat_dir(entry_point: &PathBuf) -> Result<PathBuf> {
    let data_file = entry_point.join(".mm");

    if data_file.is_file() {
        return Ok(data_file);
    }
    std::fs::write(&data_file, "").context("Failed to create data file")?;
    Ok(data_file)
}

fn validate_entry_point(config_path: &PathBuf) -> Result<PathBuf> {
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

fn validate_config_path() -> Result<PathBuf> {
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
