use std::{env, path::PathBuf};

use crate::config::Config;
use anyhow::{anyhow, bail, Context, Result};
use walkdir::WalkDir;

pub fn handle_switch(config: &mut Config, reference: Option<String>) -> Result<()> {
    match reference {
        Some(reference) => reference_switch(config, reference),
        None => context_switch(config),
    }
}

pub fn context_switch(config: &mut Config) -> Result<()> {
    let env_exe = env::current_dir().context("Failed to retrieve current working directory")?;

    let mut current_path = Some(env_exe.as_path());

    while let Some(path) = current_path {
        if !path.exists() || path == config.entry() {
            break;
        }
        if let Some(course) = config.get_course(path) {
            config.set_active_course(Some(course.name().into()))?;
            if let Some(semester) = config.get_semester(path) {
                config.set_active_semester(&semester.file_name(), false)?;
                println!("Switched course to '{}/{}'.", semester, course.name());
                return Ok(());
            }
        }
        current_path = path.parent();
    }

    if let Some(semester) = config.get_semester(&env_exe) {
        config.set_active_semester(&semester.file_name(), true)?;
        println!("Switched semester to '{}'.", semester.file_name());
        return Ok(());
    }

    bail!("No course or semester found in the curent path: {:?}\nPlease give an explicit reference to switch context.", env_exe);
}

pub fn reference_switch(config: &mut Config, reference: String) -> Result<()> {
    if reference.is_empty() {
        bail!("Empty reference given to switch context.");
    }

    let split = reference.split('/').collect::<Vec<&str>>();

    if split.len() == 2 {
        return path_switch(config, reference);
    }

    let walk = |path: &PathBuf, level: usize| {
        WalkDir::new(path)
            .max_depth(level)
            .into_iter()
            .filter_map(|it| it.ok())
            .filter(|it| {
                if let Some(file_name) = it.file_name().to_str() {
                    file_name == &reference
                } else {
                    false
                }
            })
            .into_iter()
            .next()
    };

    if let Some(semester) = config.active_semester() {
        if let Some(entry) = walk(semester.path(), 1) {
            let name = entry
                .file_name()
                .to_str()
                .expect("Failed to convert filename to string")
                .to_string();
            config.set_active_course(Some(&name))?;
            println!("Switched course to '{}'.", name);
            return Ok(());
        }
    };

    if let Some(entry) = walk(config.entry(), 2) {
        let name = entry
            .file_name()
            .to_str()
            .expect("Failed to convert filename to string")
            .to_string();

        if entry.depth() == 1 {
            config.set_active_semester(&name, true)?;
            println!("Switched semster to '{}'.", name);
            return Ok(());
        }

        if entry.depth() == 2 {
            let parent = entry
                .path()
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .ok_or_else(|| anyhow!("Failed to retrieve parent directory name"))?
                .to_string();
            config.set_active_semester(&parent, true)?;
            config.set_active_course(Some(&name))?;
            println!("Switched course to '{}/{}'.", parent, name);
            return Ok(());
        }

        return Ok(());
    }

    bail!("No course or semester found with the reference: {}\nPlease give a valid reference to switch context.", reference);
}

fn path_switch(config: &mut Config, reference: String) -> Result<()> {
    let split = reference.split('/').collect::<Vec<&str>>();
    let semester = split[0];
    let course = split[1];

    let semester = config.entry().join(semester);

    if let Some(semester) = config.get_semester(semester) {
        config.set_active_semester(&semester.file_name(), true)?;
        let course = semester.path().join(course);
        if let Some(course) = config.get_course(course) {
            config.set_active_course(Some(course.name()))?;
            println!("Switched course to '{}/{}'.", semester, course.name());
            return Ok(());
        }
    }

    bail!("No course or semester found with the reference: {}\nPlease give a valid reference to switch context.", reference);
}
