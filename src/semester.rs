use std::path::PathBuf;

use anyhow::bail;
use anyhow::Result;
use walkdir::WalkDir;

use crate::cli::StudyCycle;
use crate::config::Config;
use crate::SemesterCommands;

pub fn handle_semester(config: &mut Config, command: Option<SemesterCommands>) -> Result<()> {
    let command = command.unwrap_or(SemesterCommands::List);
    match command {
        SemesterCommands::List => list_semesters(config),
        SemesterCommands::Add {
            number,
            study_cycle,
        } => add_semester(config, number, study_cycle),
        SemesterCommands::Remove { name } => remove_semester(config, name),
    }
}

fn list_semesters(config: &Config) -> Result<()> {
    let semesters = get_semester(config.entry());
    if semesters.is_empty() {
        bail!("No Semesters found");
    }
    for semester in semesters {
        if let Some(active_semester) = config.active_semester() {
            if semester == active_semester {
                print!("*");
            } else {
                print!(" ");
            }
        }
        println!(" {}", semester);
    }
    Ok(())
}

fn add_semester(config: &Config, number: u16, study_cycle: Option<StudyCycle>) -> Result<()> {
    let study_cycle = if let Some(semester) = config.active_semester() {
        study_cycle.or(Some(semester.study_cycle()))
    } else {
        study_cycle
    };

    let study_cycle = study_cycle.ok_or_else(|| anyhow::anyhow!("Study cycle is required"))?;
    let path = config
        .entry()
        .join(format!("{}{:02}", study_cycle.abbreviation(), number));
    if path.exists() {
        bail!("Semester already exists");
    }
    let semester = Semester {
        number,
        study_cycle,
        path,
    };
    std::fs::create_dir_all(semester.path())?;
    println!("Added semester: {}", semester);
    Ok(())
}

fn remove_semester(config: &Config, name: String) -> Result<()> {
    // Ask for confirmation before removal
    println!(
        "Are you sure you want to remove the semester '{}'? (y/N)",
        name
    );
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    if input.trim().to_lowercase() == "y" {
        let path = config.entry().join(&name);
        if path.exists() {
            std::fs::remove_dir_all(&path)?;
            println!("Removed semester: {}", name);
        } else {
            bail!("Semester '{}' does not exist", name);
        }
    } else {
        println!("Aborted removal of semester: {}", name);
    }
    Ok(())
}

#[derive(Debug)]
pub struct Semester {
    number: u16,
    study_cycle: StudyCycle,
    path: PathBuf,
}

impl Semester {
    pub fn from_path(path: PathBuf) -> Option<Semester> {
        let name = path.file_name()?.to_str()?;
        let (study_cycle, number_str) = match name.chars().next()? {
            'b' => (StudyCycle::Bachelor, &name[1..]),
            'm' => (StudyCycle::Master, &name[1..]),
            'd' => (StudyCycle::Doctorate, &name[1..]),
            _ => return None,
        };
        let number = number_str.parse::<u16>().ok()?;
        Some(Semester {
            number,
            study_cycle,
            path,
        })
    }

    pub fn file_name(&self) -> String {
        format!("{}{:02}", self.study_cycle.abbreviation(), self.number)
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }
    pub fn study_cycle(&self) -> StudyCycle {
        self.study_cycle
    }
}

impl PartialEq for Semester {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl Eq for Semester {}

impl PartialOrd for Semester {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(
            (self.number * self.study_cycle.to_base())
                .cmp(&(&other.number * other.study_cycle.to_base())),
        )
    }
}

impl Ord for Semester {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.number * self.study_cycle.to_base())
            .cmp(&(&other.number * other.study_cycle.to_base()))
    }
}

use std::fmt;

impl fmt::Display for Semester {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02}. {}", self.number, self.study_cycle)
    }
}

fn get_semester(entry_point: &PathBuf) -> Vec<Semester> {
    let mut semesters = WalkDir::new(entry_point)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_dir())
        .filter_map(|entry| Semester::from_path(entry.path().to_path_buf()))
        .collect::<Vec<Semester>>();
    semesters.sort();
    semesters
}
