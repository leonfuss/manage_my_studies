use std::env;

use crate::store::Store;
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
    let mut current_path = Some(env_exe.as_path());

    while let Some(path) = current_path {
        if let Some(course) = find_course_in_path(store, path) {
            let semester = course.semester(store)?;
            store.set_active(Some(&semester))?;
            let active_semester = store.active_semester_mut().unwrap();
            active_semester.set_active(Some(&course))?;
            println!("Switched to course: {}/{}", semester.name(), course.name());
            return Ok(());
        }

        if let Some(semester) = store.get_semester(path.file_name().unwrap().to_str().unwrap()) {
            store.set_active(Some(&semester))?;
            println!("Switched to semester: {}", semester.name());
            let active_semester = store.active_semester_mut().unwrap();
            active_semester.set_active(None)?;
            return Ok(());
        }

        current_path = path.parent();
    }

    Err(anyhow!(
        "No semester or course found in the current environment"
    ))
}

fn find_course_in_path(store: &Store, path: &std::path::Path) -> Option<crate::store::Course> {
    let course_name = path.file_name()?.to_str()?;
    store
        .semesters()
        .flat_map(|semester| semester.get_course(course_name).ok())
        .next()
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
