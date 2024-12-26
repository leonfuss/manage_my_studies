use crate::{store::Store, CourseCommands};
use anyhow::{anyhow, Context, Result};

pub fn course(store: &mut Store, command: Option<CourseCommands>) -> Result<()> {
    let command = command.unwrap_or(CourseCommands::List);
    match command {
        CourseCommands::List => list(store),
        CourseCommands::Add { name } => add(store, name),
        CourseCommands::Remove { name } => remove(store, name),
        CourseCommands::Move { to, from } => from_to(store, to, from),
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
    let semester = store
        .active_semester_mut()
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
    let semester = store
        .active_semester_mut()
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
