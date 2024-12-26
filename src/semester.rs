use crate::{store::Store, SemesterCommands, StudyCycle};
use anyhow::{anyhow, Context, Result};

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
