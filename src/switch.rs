use std::env;

use crate::{course::Course, semester::Semester, store::Store};
use anyhow::{anyhow, Context, Result};
use either::Either;

pub fn switch(store: &mut Store, reference: Option<String>) -> Result<()> {
    match reference {
        Some(it) => ref_switch(store, it),
        None => context_switch(store),
    }
}

pub fn context_switch(store: &mut Store) -> Result<()> {
    let env_exe = env::current_dir().context("Failed to retrieve current working directory")?;

    let w_dir = env_exe
        .canonicalize()
        .context("Failed to canonicalize current working directory")?;

    if w_dir == *store.entry_point() {
        store.set_active(None)?;
        return Ok(());
    }

    let index = w_dir
        .ancestors()
        .position(|anchestor| anchestor == store.entry_point())
        .ok_or_else(|| {
            anyhow!(
                "No semester or course found in the current environment.\n The current working directory must be a subfolder of the entry point ({})",
                store.entry_point().display()
            )
        })?;

    if index == 1 {
        let mut semester = Semester::new(&w_dir)?;
        store.set_active(Some(&semester))?;
        semester.set_active(None)?;
        return Ok(());
    }

    if index >= 2 {
        let semester_path = w_dir.ancestors().nth(index - 1).ok_or_else(|| {
            anyhow!(
                "Failed to access path anchestor at index: {}\npath:{}",
                index,
                w_dir.display()
            )
        });
        let mut semester = Semester::new(&semester_path?)?;
        store.set_active(Some(&semester))?;
        let course_path = w_dir.ancestors().nth(index - 2).ok_or_else(|| {
            anyhow!(
                "Failed to access path anchestor at index: {}\npath:{}",
                index,
                w_dir.display()
            )
        })?;
        let course = Course::new(course_path)?;
        semester.set_active(Some(&course))?;
    }

    Ok(())
}

pub fn ref_switch(store: &mut Store, reference: String) -> Result<()> {
    match store.get_by_reference(&reference) {
        Some(it) => match it {
            Either::Left(semester) => {
                store.set_active(Some(&semester))?;
                println!("Switched to semester: {}", semester.name());
            }
            Either::Right((mut semester, course)) => {
                store.set_active(Some(&semester))?;
                semester.set_active(Some(&course))?;
                println!("Switched to course: {}/{}", semester.name(), course.name());
            }
        },
        None => return Err(anyhow!("No semester found by reference: {}", reference)),
    }
    Ok(())
}
