use std::{
    ops::Deref,
    path::{Path, PathBuf},
};

use crate::{
    course::Course,
    store::{EnsureExistance, Files, ReadWriteData, SemesterDataFile, Store},
    SemesterCommands, StudyCycle,
};
use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

#[derive(Debug, Deserialize, Serialize)]
pub struct Semester {
    #[serde(skip)]
    semester_number: u16,
    #[serde(skip)]
    #[serde(default = "placeholder_study_cycle")]
    cycle: StudyCycle,
    #[serde(skip)]
    path: PathBuf,
    #[serde(skip)]
    name: String,
    active_course: Option<String>,
}

fn placeholder_study_cycle() -> StudyCycle {
    StudyCycle::Bachelor
}

impl ReadWriteData for SemesterDataFile {
    type Object = Semester;
    fn read(&self) -> Result<Self::Object> {
        let content = std::fs::read_to_string(self.deref())
            .with_context(|| anyhow!("Failed to read file at: {}", self.deref().display()))?;
        let mut semester: Semester = toml_edit::de::from_str(&content).with_context(|| {
            anyhow!(
                "Failed to parse Semester data from: {}",
                self.deref().display()
            )
        })?;
        semester.path = (*self).to_path_buf();
        Ok(semester)
    }

    fn write(&self, object: &Self::Object) -> Result<()> {
        let data = toml_edit::ser::to_string(&object).with_context(|| {
            anyhow!(
                "Failed to serialize Semester data to toml for: {}",
                self.deref().display()
            )
        })?;
        std::fs::write(self.deref(), data).with_context(|| {
            anyhow!(
                "Failed to write Semester data to file: {}",
                self.deref().display()
            )
        })?;
        Ok(())
    }
}

impl Semester {
    pub fn new<P>(path: P) -> Result<Semester>
    where
        P: AsRef<Path>,
    {
        let (cycle, semester_number, name) = Semester::deserialize_path(path.as_ref())?;
        let data = path.semester_data_file();
        data.ensure_exists()?;
        let mut semester = data.read()?;
        semester.semester_number = semester_number;
        semester.cycle = cycle;
        semester.path = path.as_ref().to_path_buf();
        semester.name = name;

        Ok(semester)
    }

    fn deserialize_path(path: &Path) -> Result<(StudyCycle, u16, String)> {
        if !path.exists() {
            return Err(anyhow::anyhow!(
                "Semester '{}' does not exist",
                path.display()
            ));
        }
        let name = path
            .file_name()
            .ok_or_else(|| anyhow!("filename could not be extracted"))
            .map(|it| it.to_string_lossy().to_string())?;
        let (cycle, number) = match name
            .chars()
            .next()
            .ok_or_else(|| anyhow!("Semester name is empty"))?
        {
            'b' => (StudyCycle::Bachelor, &name[1..]),
            'm' => (StudyCycle::Master, &name[1..]),
            'd' => (StudyCycle::Doctorate, &name[1..]),
            _ => return Err(anyhow::anyhow!("Invalid semester name '{}'", name)),
        };
        let number = number.parse::<u16>().with_context(|| {
            anyhow!(
                "Failed to parse number: {} in semester name {}",
                number,
                name
            )
        })?;
        Ok((cycle, number, name))
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn get_course(&self, name: &str) -> Result<Course> {
        let path = self.path().join(name);
        if !path.exists() {
            return Err(anyhow!("Course '{}' does not exist", name));
        }
        Course::new(path)
    }

    pub fn is_active(&self, coures: &Course) -> bool {
        self.active_course
            .as_ref()
            .map(|it| self.path.join(it) == *coures.path())
            .unwrap_or(false)
    }

    pub fn courses(&self) -> impl Iterator<Item = Course> + 'static {
        WalkDir::new(&self.path)
            .min_depth(1)
            .max_depth(1)
            .sort_by_file_name()
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().is_dir())
            .filter_map(|entry| Course::new(entry.path()).ok())
    }
    pub fn add_course(&self, name: &str) -> Result<Course> {
        let path = self.path().join(name);
        if path.exists() {
            return Err(anyhow!("Course '{}' already exists", name));
        }

        std::fs::create_dir(&path)?;
        let course_toml_path = path.join(".course.toml");
        std::fs::write(&course_toml_path, include_str!("../course.toml"))?;
        Course::new(path)
    }

    pub fn remove_course(&mut self, course: Course) -> Result<()> {
        if self.is_active(&course) {
            self.active_course = None;
        }
        std::fs::remove_dir_all(course.path())?;
        Ok(())
    }

    pub fn move_course(&mut self, course: Course, to: &str) -> Result<Course> {
        let to_path = self.path().join(&to);
        if to_path.exists() {
            return Err(anyhow!("A course with the name '{}' already exists", to));
        }
        std::fs::rename(course.path(), &to_path)?;
        let new_course = Course::new(to_path)?;
        if self.is_active(&course) {
            self.active_course = Some(new_course.name().to_string());
        }
        Ok(new_course)
    }
    pub fn active(&self) -> Option<Course> {
        self.active_course
            .as_ref()
            .map(|it| Course::new(self.path.join(it)).ok())
            .flatten()
    }
    pub fn set_active(&mut self, course: Option<&Course>) -> Result<()> {
        self.active_course = course.map(|it| it.name().to_owned());
        self.write()
    }

    fn write(&self) -> Result<()> {
        let path = self.path.semester_data_file();
        path.ensure_exists()?;
        path.write(self)
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }
    pub fn study_cycle(&self) -> StudyCycle {
        self.cycle
    }

    pub fn format_name(cycle: StudyCycle, semester_number: u16) -> String {
        format!("{}{:02}", cycle.abbreviation(), semester_number)
    }
}

pub fn semester(store: &mut Store, command: Option<SemesterCommands>) -> Result<()> {
    let command = command.unwrap_or(SemesterCommands::List);
    match command {
        SemesterCommands::List => list(store),
        SemesterCommands::Add {
            number,
            study_cycle,
        } => add(store, number, study_cycle),
        SemesterCommands::Remove { name } => remove(store, name),
    }
}

fn list(store: &Store) -> Result<()> {
    let semesters = store.semesters();
    let mut run = false;
    for semester in semesters {
        run = true;
        if store.is_active(&semester) {
            print!("*");
        } else {
            print!(" ");
        }
        println!(" {}", semester.name());
    }
    if !run {
        println!("No semesters found");
    }
    Ok(())
}

fn add(store: &Store, number: u16, study_cycle: Option<StudyCycle>) -> Result<()> {
    let semester = store.add_semester(number, study_cycle)?;
    println!("Added semester: {}", semester.name());
    Ok(())
}

fn remove(store: &mut Store, name: String) -> Result<()> {
    let semster = store
        .get_semester(&name)
        .with_context(|| anyhow!("Semester could not be found"))?;
    use std::io::{self, Write};

    print!("Do you really want to delete semester '{}'? (y/N): ", name);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    if input.trim().to_lowercase() != "y" {
        println!("Aborted removal of semester: {}", name);
        return Ok(());
    }

    store.remove_semester(semster)?;
    println!("Removed semester: {}", name);
    Ok(())
}
