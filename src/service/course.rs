use crate::domain::Course;
use crate::service::format::FormatAlignment;
use crate::table;
use crate::{cli::CourseCommands, StoreProvider};
use anyhow::{anyhow, bail};

use super::format::{DialogEntry, DialogOutput, FormatService, IntoFormatType};
use super::ServiceResult;

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

    pub fn run(&mut self, command: Option<CourseCommands>) -> ServiceResult {
        let command = command.unwrap_or(CourseCommands::List);
        match command {
            CourseCommands::List => self.list(),
            CourseCommands::Add { name } => self.add(name),
            CourseCommands::Remove { name } => self.remove(name),
        }
    }

    fn list(&self) -> ServiceResult {
        let semester = match self.store.current_semester() {
            Some(semester) => semester,
            None => {
                let error = "No active semester found".error();
                let info =
                    "An active semester is required in order to list the corresponding courses"
                        .info();

                return Ok(error.chain(info));
            }
        };

        let mut courses = semester
            .courses()
            .map(|course| course.name())
            .collect::<Vec<_>>();
        courses.sort();

        if courses.is_empty() {
            let msg = "No courses found".info();
            return Ok(msg);
        }

        let active_idx = self.store.current_semester().map(|active_sem| {
            (&courses)
                .iter()
                .map(|course| {
                    if course == &active_sem.name() {
                        return "*".into();
                    }
                    return " ".into();
                })
                .collect()
        });

        let table = match active_idx {
            Some(active) => {
                table!("Active", "Courses"; active, courses; FormatAlignment::Right, FormatAlignment::Right)
            }
            None => table!("Courses"; courses; FormatAlignment::Right),
        };
        Ok(table)
    }

    fn add(&mut self, name: String) -> ServiceResult {
        let semester = match self.store.current_semester() {
            Some(semester) => semester,
            None => {
                let error = "No active semester found".error();
                let info = "An active semester is required in order to add a new course".info();
                return Ok(error.chain(info));
            }
        };

        let course_path = semester.path().create_course_path(&name)?;
        // used to create course data file
        let _ = Course::from_path(course_path)?;
        let msg = format!("Course '{}' has been added", name).success();
        Ok(msg)
    }

    fn remove(&mut self, name: String) -> ServiceResult {
        let semester = match self.store.current_semester() {
            Some(semester) => semester,
            None => {
                let error = "No active semester found".error();
                let info = "An active semester is required in order to remove a course".info();
                return Ok(error.chain(info));
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
                bail!("Invalid input");
            };

            if *cond {
                let course = semester
                    .course(&name)
                    .ok_or_else(|| anyhow!("Course '{}' could not be found", name))?;

                course.path().clone().remove()?;
                let msg = format!("Semester '{}' has been removed", name).success();
                return Ok(msg);
            } else {
                return Ok("Operation has been canceled".info());
            }
        } else {
            return Ok("Operation has been canceled".info());
        }
    }
}
