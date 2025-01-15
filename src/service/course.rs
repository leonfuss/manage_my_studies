use crate::domain::Course;
use crate::{cli::CourseCommands, StoreProvider};
use anyhow::anyhow;
use anyhow::Result;

use super::format::{DialogEntry, DialogOutput, FormatService};

pub(super) struct CourseService<'s, Store>
where
    Store: StoreProvider,
{
    store: &'s mut Store,
}

impl<'s, Store> CourseService<'s, Store>
where
    Store: StoreProvider,
{
    pub fn new(store: &'s mut Store) -> Self {
        Self { store }
    }

    pub fn run(&mut self, command: Option<CourseCommands>) -> Result<()> {
        let command = command.unwrap_or(CourseCommands::List);
        match command {
            CourseCommands::List => self.list(),
            CourseCommands::Add { name } => self.add(name),
            CourseCommands::Remove { name } => self.remove(name),
        }
    }

    fn list(&self) -> Result<()> {
        let semester = match self.store.current_semester() {
            Some(semester) => semester,
            None => {
                FormatService::error("No active semester found");
                FormatService::info(
                    "An active semester is required in order to list the corresponding courses",
                );
                return Ok(());
            }
        };

        let mut courses = semester
            .courses()
            .map(|course| course.name())
            .collect::<Vec<_>>();
        courses.sort();

        if courses.is_empty() {
            FormatService::info("No courses found");
            return Ok(());
        }

        // Find index of active semester
        let active_idx = self
            .store
            .current_semester()
            .and_then(|active_sem| courses.iter().position(|name| name == &active_sem.name()));

        // Create active checker function
        let is_active =
            move |idx: usize| -> bool { active_idx.map_or(false, |active| idx == active) };

        FormatService::active_item_table(courses, Box::new(is_active));
        Ok(())
    }

    fn add(&mut self, name: String) -> Result<()> {
        let semester = match self.store.current_semester() {
            Some(semester) => semester,
            None => {
                FormatService::error("No active semester found");
                FormatService::info("An active semester is required in order to add a new course");
                return Ok(());
            }
        };

        let course_path = semester.path().create_course_path(&name)?;
        // used to create course data file
        let _ = Course::from_path(course_path)?;
        Ok(())
    }

    fn remove(&mut self, name: String) -> Result<()> {
        let semester = match self.store.current_semester() {
            Some(semester) => semester,
            None => {
                FormatService::error("No active semester found");
                FormatService::info("An active semester is required in order to remove a course");
                return Ok(());
            }
        };

        let dialog = vec![
            DialogEntry::YesNoInput(format!("Are you sure that you want to permanently remove course '{}' with all its content? This action can not be reverted",name))
        ];
        let response = FormatService::dialog(dialog);
        if let Some(res) = response {
            let res = res
                .first()
                .ok_or_else(|| anyhow!("Dialog has not returned not the specified output"))?;
            let DialogOutput::YesNo(cond) = res else {
                FormatService::error("Invalid input");
                return Ok(());
            };

            if *cond {
                let course = semester
                    .course(&name)
                    .ok_or_else(|| anyhow!("Course '{}' could not be found", name))?;

                course.path().clone().remove()?;
                FormatService::success(&format!("Semester '{}' has been removed", name));
            } else {
                FormatService::info("Operation has been canceled");
            }
        } else {
            FormatService::info("Operation has been canceled");
        }
        Ok(())
    }
}
