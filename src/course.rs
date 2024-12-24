use std::path::PathBuf;

use crate::{config::Config, CourseCommands};
use anyhow::{bail, Context, Result};
use walkdir::WalkDir;

pub fn handle_course(config: &mut Config, command: Option<CourseCommands>) -> Result<()> {
    let command = command.unwrap_or(CourseCommands::List);
    match command {
        CourseCommands::List => list_courses(config),
        CourseCommands::Add { name } => add_course(config, name),
        CourseCommands::Remove { name } => remove_course(config, name),
        CourseCommands::Move { from, to } => move_course(config, from, to),
    }
}

fn list_courses(config: &Config) -> Result<()> {
    let active_semester = config
        .active_semester()
        .ok_or_else(|| anyhow::anyhow!("An active semester is required to list courses"))?;

    let courses = get_course(active_semester.path());
    if courses.is_empty() {
        println!("No courses found for the active semester.");
    } else {
        for course in courses {
            if let Some(active_course) = config.active_course() {
                if course.name == active_course.name {
                    print!("* ");
                } else {
                    print!("  ");
                }
            } else {
                print!("  ");
            }
            println!("{} - {}", course.name, course.long_name);
        }
    }
    Ok(())
}

fn add_course(config: &Config, name: String) -> Result<()> {
    let active_semester = config
        .active_semester()
        .ok_or_else(|| anyhow::anyhow!("An active semester is required to add a course"))?;

    let path = active_semester.path().join(&name);

    if path.exists() {
        bail!("Course with same name already exists");
    }

    std::fs::create_dir_all(&path)
        .with_context(|| format!("Failed to create course folder at: {}", path.display()))?;

    let course_toml_path = path.join(".course.toml");
    std::fs::write(&course_toml_path, include_str!("../course.toml")).with_context(|| {
        format!(
            "Failed to create .course.toml file at: {}",
            course_toml_path.display()
        )
    })
}

fn remove_course(config: &Config, name: String) -> Result<()> {
    let active_semester = config
        .active_semester()
        .ok_or_else(|| anyhow::anyhow!("An active semester is required to remove a courses"))?;

    // Ask for confirmation before removal
    println!(
        "Are you sure you want to remove the course '{}/{}'? (y/N)",
        active_semester, name
    );
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    if input.trim().to_lowercase() == "y" {
        let path = active_semester.path().join(&name);
        if path.exists() {
            std::fs::remove_dir_all(&path)?;
            println!("Removed course: {}", name);
        } else {
            bail!("Course '{}' does not exist", name);
        }
    } else {
        println!("Aborted removal of course: {}", name);
    }
    Ok(())
}

fn move_course(config: &Config, from: Option<String>, to: String) -> Result<()> {
    let active_semester = config
        .active_semester()
        .ok_or_else(|| anyhow::anyhow!("An active semester is required to move a courses"))?;

    let course = match (from, config.active_course()) {
        (Some(from), _) => from,
        (None, Some(active_course)) => active_course.name,
        (None, None) => bail!("An active course or a 'from' course is required to move a course"),
    };

    let from_path = active_semester.path().join(&course);
    let to_path = active_semester.path().join(&to);

    if !from_path.exists() {
        bail!("The course '{}' does not exist", course);
    }

    if to_path.exists() {
        bail!("A course with the name '{}' already exists", to);
    }

    std::fs::rename(&from_path, &to_path)
        .with_context(|| format!("Failed to move course from '{}' to '{}'", course, to))?;
    println!("Moved course from '{}' to '{}'", course, to);
    Ok(())
}

fn get_course(semester_entry: &PathBuf) -> Vec<Course> {
    let mut courses = WalkDir::new(semester_entry)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_dir())
        .filter_map(|entry| Course::from_path(entry.path().to_path_buf()))
        .collect::<Vec<Course>>();
    courses.sort_by(|a, b| a.name.cmp(&b.name));
    courses
}

pub struct Course {
    name: String,
    long_name: String,
}

impl Course {
    pub fn from_path(path: PathBuf) -> Option<Course> {
        let name = path.file_name()?.to_str()?.to_string();
        let course_toml_path = path.join(".course.toml");
        let long_name = if course_toml_path.exists() {
            let content = std::fs::read_to_string(course_toml_path).ok()?;
            let value = content.parse::<toml_edit::DocumentMut>().ok()?;
            value.get("long_name")?.as_str()?.to_string()
        } else {
            println!("'course.toml' could not be found for {}", path.display());
            String::new()
        };
        Some(Course { name, long_name })
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
