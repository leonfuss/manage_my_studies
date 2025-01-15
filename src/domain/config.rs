use anyhow::{anyhow, bail, Context, Result};
use regex::Regex;
use std::path::{Path, PathBuf};

use crate::ConfigProvider;

use super::{
    paths::{EntryPoint, MaybeSymLinkable},
    semester::StudyCycle,
};

#[derive(Debug, serde::Deserialize)]
struct ConfigDO {
    entry_point: String,
    semster_names: Option<String>,
    study_cycle_mapping: Option<StudyCycleMappingDO>,
    semester_link: Option<PathBuf>,
    course_link: Option<PathBuf>,
}

#[derive(Debug, serde::Deserialize, Clone)]
struct StudyCycleMappingDO {
    bachelor: Option<String>,
    master: Option<String>,
    doctorate: Option<String>,
}

pub(crate) struct Config {
    /// The path to the directory where the university data is stored.
    entry_point: EntryPoint,
    /// A regex pattern to match the names of the semesters.
    semester_names: SemesterNames,
    /// Path to optional symlink to the current semester folder.
    semester_link: MaybeSymLinkable,
    /// Path to optional symlink to the current course folder.
    course_link: MaybeSymLinkable,
}

/// [SemesterNames] defines the relationship between the folder names and the study cycle as well es semester number.
/// The regex pattern is used to validate the folder names and extract the study cycle and semester number. A valid regex
/// must contain the named capture groups "study_cycle" and "semester_number". "semester_number" must be numeric. And is
/// expected to run from 1 to ... for each study cycle. The study cycle is mapped to the [StudyCycle] enum using the following:
/// - "b" -> [StudyCycle::Bachelor]
/// - "m" -> [StudyCycle::Master]
/// - "d" -> [StudyCycle::Doctorate]
/// A custom mapping can be provided using the StudyCycleMapping Table [StudyCycleMappingDO]
///
/// If no regex is provided it defaults to: `r"^(?P<study_cycle>[bmd])(?P<semester_number>\d{2})$"`
#[derive(Debug, Clone)]
pub(crate) struct SemesterNames {
    regex: Regex,
    study_cycle_mapping: Vec<(String, StudyCycle)>,
}

impl SemesterNames {
    pub fn is_name(&self, name: &str) -> bool {
        self.regex.is_match(name)
    }

    pub fn deserialize(&self, name: &str) -> Result<(u16, StudyCycle)> {
        let captures = self
            .regex
            .captures(name)
            .ok_or_else(|| anyhow!("Provided name is not a valid semester name"))?;
        let semester_number = captures
            .name("semester_number")
            .ok_or_else(|| anyhow!("Failed to extract semester number"))?
            .as_str()
            .parse::<u16>()
            .with_context(|| anyhow!("Failed to parse semester number"))?;
        let study_cycle = captures
            .name("study_cycle")
            .ok_or_else(|| anyhow!("Failed to extract study cycle"))?
            .as_str()
            .to_string();
        let (_, study_cycle) = self.study_cycle_mapping.iter().filter(|(it, _)| it == &study_cycle).next().
            with_context(|| anyhow!("semester name capute (<study_cylce>: {}) could not be matched to study cycle: \nSemester name: {}", study_cycle, name))?;
        Ok((semester_number, study_cycle.clone()))
    }
}

impl Config {
    /// Loads the configuration from the default config file location or creates a new one if it does not exist.
    ///
    /// Platform-specific config directory paths
    /// Linux: $XDG_CONFIG_HOME or $HOME/.config/mm/config.toml
    /// macOS: $HOME/.config/mm/config.toml
    /// Windows: {FOLDERID_RoamingAppData}\mm\config.toml
    pub fn new() -> Result<Config> {
        let config_path = Self::config_path()?.join("mm").join("config.toml");
        if !config_path.is_file() {
            Self::create_default_config_file()?;
            bail!(
                "A new config file has been created at: '{}'. Please set the 'entry_point' in the config file and try again.",
                &config_path.display()
            )
        }
        Config::from_path(config_path)
    }

    pub fn from_path<P>(path: P) -> Result<Config>
    where
        P: AsRef<Path>,
    {
        let file =
            std::fs::read_to_string(path).with_context(|| anyhow!("Failed to open config file"))?;
        let config_do = toml_edit::de::from_str::<ConfigDO>(&file)
            .with_context(|| anyhow!("Could not read Config from toml"))?;

        let entry_point = EntryPoint::new(&config_do.entry_point)?;
        let semester_names =
            SemesterNames::new(config_do.semster_names, config_do.study_cycle_mapping)?;
        let course_link = MaybeSymLinkable::new(config_do.course_link)?;
        let semester_link = MaybeSymLinkable::new(config_do.semester_link)?;

        let config = Config {
            entry_point,
            semester_names,
            course_link,
            semester_link,
        };
        Ok(config)
    }

    pub fn create_default_config_file() -> Result<()> {
        let path = Self::config_path()?;
        let parent = path
            .parent()
            .context("Failed to load parent of config file")?;
        std::fs::create_dir_all(parent)?;
        let config_content = include_str!("../../config.toml");
        std::fs::write(path, config_content).context("Failed to create config file")?;
        Ok(())
    }

    pub fn config_path() -> Result<PathBuf> {
        if cfg!(target_os = "macos") {
            let home_dir =
                dirs::home_dir().context("Failed to find home directory on your system")?;
            Ok(home_dir.join(".config"))
        } else {
            dirs::config_dir().context("Failed to find config directory on your system.")
        }
    }
}

impl ConfigProvider for Config {
    fn entry_point(&self) -> EntryPoint {
        self.entry_point.clone()
    }

    fn current_course_link(&self) -> MaybeSymLinkable {
        self.course_link.clone()
    }

    fn current_semester_link(&self) -> MaybeSymLinkable {
        self.semester_link.clone()
    }

    fn semester_names(&self) -> SemesterNames {
        self.semester_names.clone()
    }
}

impl SemesterNames {
    pub(self) fn new(
        regex: Option<String>,
        study_cylce_mapping: Option<StudyCycleMappingDO>,
    ) -> Result<SemesterNames> {
        let capture_groups = vec!["study_cycle", "semester_number"];
        let default_regex = r"^(?P<study_cycle>[bmd])(?P<semester_number>\d{2})";
        let default_map = StudyCycleMappingDO {
            bachelor: Some("b".into()),
            master: Some("m".into()),
            doctorate: Some("d".into()),
        };

        let regex = match regex {
            Some(rx) => validate::semester_regex(&rx, &capture_groups)?,
            None => {
                let regex = validate::semester_regex(default_regex, &capture_groups)?;
                let study_cycle_mapping = validate::study_cycle_mapping(None, default_map)?;
                let semester_names = SemesterNames {
                    regex,
                    study_cycle_mapping,
                };
                return Ok(semester_names);
            }
        };

        let study_cycle_mapping = validate::study_cycle_mapping(study_cylce_mapping, default_map)?;
        let semester_names = SemesterNames {
            regex,
            study_cycle_mapping,
        };
        Ok(semester_names)
    }
}

mod validate {
    use std::collections::HashSet;

    use crate::domain::semester::StudyCycle;

    use super::*;

    pub(super) fn semester_regex(regex: &str, capture_groups: &Vec<&str>) -> Result<Regex> {
        let regex = Regex::new(regex)
            .with_context(|| anyhow!("Failed to build semester-folder regex: {}", regex))?;
        let caputure_names: HashSet<&str> = regex.capture_names().flatten().collect();

        for capture in capture_groups {
            if !caputure_names.contains(capture) {
                bail!(
                    "Semester-folder regex does not contain capture group: {}",
                    capture
                )
            }
        }
        Ok(regex)
    }

    pub(super) fn study_cycle_mapping(
        mapping: Option<StudyCycleMappingDO>,
        default_map: StudyCycleMappingDO,
    ) -> Result<Vec<(String, StudyCycle)>> {
        fn fill(input: Option<String>, default: Option<String>) -> Result<String> {
            let out = input
                .or(default)
                .ok_or_else(|| anyhow!("Study-cycle default mapping does [None] values"))?;
            Ok(out)
        }

        let mapping = mapping.unwrap_or_else(|| default_map.clone());
        let bachelor = fill(mapping.bachelor, default_map.bachelor)?;
        let master = fill(mapping.master, default_map.master)?;
        let doctorate = fill(mapping.doctorate, default_map.doctorate)?;
        let mapping = vec![
            (bachelor, StudyCycle::Bachelor),
            (master, StudyCycle::Master),
            (doctorate, StudyCycle::Doctorate),
        ];
        Ok(mapping)
    }
}
