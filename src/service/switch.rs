use std::env;

use crate::StoreProvider;
use anyhow::{anyhow, bail, Context, Result};

use super::format::FormatService;

pub(super) struct SwitchService<'s, Store>
where
    Store: StoreProvider,
{
    store: &'s mut Store,
}

impl<'s, Store> SwitchService<'s, Store>
where
    Store: StoreProvider,
{
    pub fn new(store: &'s mut Store) -> SwitchService<'s, Store> {
        SwitchService { store }
    }

    pub fn run(&mut self, reference: Option<String>) -> Result<()> {
        match reference {
            Some(it) => self.reference_switch(it),
            None => self.context_switch(),
        }
    }

    fn reference_switch(&mut self, reference: String) -> Result<()> {
        let split = reference.split('/').collect::<Vec<&str>>();
        match split.len() {
            0 => bail!("Invalid reference"),
            1 => {
                // Check if reference is a semester
                if let Some(semester) = self.store.get_semester(split[0]) {
                    self.store.set_current_semester(Some(&semester))?;
                    FormatService::success(&format!("Switched to semester: {}", semester.name()));
                    return Ok(());
                }

                // Check if reference is a course in the active semester
                if let Some(mut active_semester) = self.store.current_semester() {
                    if let Some(course) = active_semester.course(split[0]) {
                        self.store
                            .set_current_course(&mut active_semester, Some(&course))?;
                        FormatService::success(&format!(
                            "Switched to course: {}/{}",
                            active_semester.name(),
                            course.name()
                        ));
                        return Ok(());
                    }
                }

                // Check if reference is a course in any semester
                let courses: Vec<_> = self.store.courses().collect();
                if let Some(course) = courses.iter().find(|course| course.name() == split[0]) {
                    let semesters: Vec<_> = self.store.semesters().collect();
                    if let Some(mut semester) = semesters
                        .into_iter()
                        .find(|semester| semester.course(&course.name()).is_some())
                    {
                        self.store.set_current_semester(Some(&semester))?;
                        self.store.set_current_course(&mut semester, Some(course))?;
                        FormatService::success(&format!(
                            "Switched to course: {}/{}",
                            semester.name(),
                            course.name()
                        ));
                        return Ok(());
                    }
                    bail!("No semester found for course: {}", course.name());
                }
                bail!("No course found by reference: {}", reference)
            }
            2 => {
                let mut semester = self.store.get_semester(split[0]).ok_or_else(|| {
                    anyhow!(
                        "No semester found matching the reference semester part '{}' of '{}'",
                        split[0],
                        reference
                    )
                })?;
                let course = semester.course(split[1]).ok_or_else(|| {
                    anyhow!(
                        "No Course found matchin the reference course part '{}' of '{}'",
                        split[1],
                        reference
                    )
                })?;
                self.store.set_current_semester(Some(&semester))?;
                self.store
                    .set_current_course(&mut semester, Some(&course))?;
                Ok(())
            }
            _ => bail!("Please provide a valid reference"),
        }
    }

    fn context_switch(&mut self) -> Result<()> {
        let env_exe = env::current_dir().context("Failed to retrieve current working directory")?;
        let entry = self.store.entry_point();

        let w_dir = env_exe
            .canonicalize()
            .context("Failed to canonicalize current working directory")?;

        if w_dir == *entry {
            self.store.set_current_semester(None)?;
            return Ok(());
        }

        let index = match w_dir.ancestors().position(|anchestor| anchestor == *entry) {
            Some(it) => it,
            None => {
                FormatService::error(&format!("No semester or course found in the current environment.\n The current working directory must be a subfolder of the entry point ({})", entry.display()));
                return Ok(());
            }
        };

        if index == 1 {
            let name = w_dir.file_name().ok_or_else(|| {
                anyhow!(
                    "Failed to retrieve file name from path: {}",
                    w_dir.display()
                )
            })?;
            let semester = self
                .store
                .get_semester(&name.to_string_lossy().to_string())
                .ok_or_else(|| anyhow!("Current directory is not a subdirectory of a semester"))?;
            self.store.set_current_semester(Some(&semester))?;
            return Ok(());
        }

        if index >= 2 {
            let semester_path = w_dir.ancestors().nth(index - 1).ok_or_else(|| {
                anyhow!(
                    "Failed to access path anchestor at index: {}\npath:{}",
                    index,
                    w_dir.display()
                )
            })?;
            let semester_name = semester_path.file_name().ok_or_else(|| {
                anyhow!(
                    "Failed to retrieve file name from path: {}",
                    w_dir.display()
                )
            })?;
            let mut semester = self
                .store
                .get_semester(&semester_name.to_string_lossy().to_string())
                .ok_or_else(|| anyhow!("Current directory is not a subdirectory of a semester"))?;
            self.store.set_current_semester(Some(&semester))?;

            let course_path = w_dir.ancestors().nth(index - 2).ok_or_else(|| {
                anyhow!(
                    "Failed to access path anchestor at index: {}\npath:{}",
                    index,
                    w_dir.display()
                )
            })?;
            let course_name = course_path.file_name().ok_or_else(|| {
                anyhow!(
                    "Failed to retrieve file name from path: {}",
                    w_dir.display()
                )
            })?;

            let course = semester
                .course(&course_name.to_string_lossy().to_string())
                .ok_or_else(|| anyhow!("Current directory is not a subdirectory of a course"))?;
            self.store
                .set_current_course(&mut semester, Some(&course))?;
        }

        Ok(())
    }
}
