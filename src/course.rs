use std::{
    ops::Deref,
    path::{Path, PathBuf},
};

use crate::{
    semester::Semester,
    store::{CourseDataFile, EnsureExistance, Files, ReadWriteData, Store},
    CourseCommands,
};
use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Course {
    #[serde(rename = "name")]
    long_name: Option<String>,
    #[serde(skip)]
    name: String,
    #[serde(skip)]
    path: PathBuf,
    grade: Option<f32>,
    ects: Option<u8>,
    degrees: Option<Vec<String>>,
    #[serde(rename = "übK")]
    uebk: Option<bool>,
}

impl ReadWriteData for CourseDataFile {
    type Object = Course;
    fn read(&self) -> Result<Self::Object> {
        let content = std::fs::read_to_string(self.deref())
            .with_context(|| anyhow!("Failed to read file at: {}", self.deref().display()))?;
        let mut course: Course = toml_edit::de::from_str(&content).with_context(|| {
            anyhow!(
                "Failed to parse Course data from: {}",
                self.deref().display()
            )
        })?;
        course.path = (*self).to_path_buf();
        Ok(course)
    }

    fn write(&self, object: &Self::Object) -> Result<()> {
        let data = toml_edit::ser::to_string(&object).with_context(|| {
            anyhow!(
                "Failed to serialize Course data to toml for: {}",
                self.deref().display()
            )
        })?;
        std::fs::write(self.deref(), data).with_context(|| {
            anyhow!(
                "Failed to write Course data to file: {}",
                self.deref().display()
            )
        })?;
        Ok(())
    }
}

pub fn course(store: &mut Store, command: Option<CourseCommands>) -> Result<()> {
    let command = command.unwrap_or(CourseCommands::List);
    match command {
        CourseCommands::List => list(store),
        CourseCommands::Add { name } => add(store, name),
        CourseCommands::Remove { name } => remove(store, name),
        CourseCommands::Move { to, from } => from_to(store, to, from),
    }
}

impl Course {
    pub fn new<P>(path: P) -> Result<Course>
    where
        P: AsRef<Path>,
    {
        let data = path.course_data_file();
        data.ensure_exists()?;
        let mut course = data.read()?;
        course.name = path
            .as_ref()
            .file_name()
            .ok_or_else(|| anyhow!("Course path has no file name"))?
            .to_string_lossy()
            .to_string();
        course.path = path.as_ref().to_path_buf();
        Ok(course)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn ects(&self) -> Option<u8> {
        self.ects
    }

    pub fn grade(&self) -> Option<f32> {
        self.grade
    }

    pub fn uebk(&self) -> Option<bool> {
        self.uebk
    }

    pub fn degrees(&self) -> impl Iterator<Item = String> + '_ {
        self.degrees.iter().flatten().cloned()
    }

    pub fn semester(&self) -> Result<Semester> {
        let parent = self
            .path
            .parent()
            .ok_or_else(|| anyhow!("Course path has no parent"))?;
        Semester::new(&parent)
    }
}

fn list(store: &Store) -> Result<()> {
    let semester = store
        .active_semester()
        .ok_or_else(|| anyhow!("No active semester"))?;

    let mut run = false;
    for course in semester.courses() {
        run = true;
        if semester.is_active(&course) {
            print!("*");
        } else {
            print!(" ");
        }
        println!("{}", course.name());
    }
    if !run {
        println!("No courses found");
    }
    Ok(())
}

fn add(store: &Store, name: String) -> Result<()> {
    let semester = store
        .active_semester()
        .ok_or_else(|| anyhow!("No active semester"))?;

    let course = semester.add_course(&name)?;
    println!("Added course: {}", course.name());
    Ok(())
}

fn remove(store: &mut Store, name: String) -> Result<()> {
    let mut semester = store
        .active_semester()
        .ok_or_else(|| anyhow!("No active semester"))?;

    let course = semester
        .get_course(&name)
        .with_context(|| anyhow!("Course could not be found"))?;

    use std::io::{self, Write};

    print!("Do you really want to delete course '{}'? (y/N): ", name);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    if input.trim().to_lowercase() != "y" {
        println!("Aborted removal of course: {}", name);
        return Ok(());
    }

    semester.remove_course(course)?;
    println!("Removed course: {}", name);
    Ok(())
}

fn from_to(store: &mut Store, to: String, from: Option<String>) -> Result<()> {
    let mut semester = store
        .active_semester()
        .ok_or_else(|| anyhow!("No active semester"))?;
    let from_course = from.unwrap_or_else(|| {
        semester
            .active()
            .map(|course| course.name().to_string())
            .unwrap_or_else(|| anyhow!("No active course").to_string())
    });

    let from_course = semester
        .get_course(&from_course)
        .with_context(|| anyhow!("FROM ({}) could not be found", from_course))?;

    semester.move_course(from_course, &to)?;
    println!(
        "Course {} was sucessful moved to {}/{}",
        semester.name(),
        semester.name(),
        to
    );
    Ok(())
}
