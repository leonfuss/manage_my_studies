use crate::{
    cli::SemesterCommands,
    domain::StudyCycle,
    service::{
        format::{DialogEntry, FormatAlignment, FormatService, IntoFormatType},
        ServiceResult,
    },
    table, StoreProvider,
};

use anyhow::{anyhow, bail, Context};

use super::format::DialogOutput;

pub(super) struct SemesterService<'s, Store>
where
    Store: StoreProvider,
{
    store: &'s mut Store,
}

impl<'s, Store> SemesterService<'s, Store>
where
    Store: StoreProvider,
{
    pub fn new(store: &'s mut Store) -> Self {
        Self { store }
    }

    pub fn run(&mut self, command: Option<SemesterCommands>) -> ServiceResult {
        let command = command.unwrap_or(SemesterCommands::List);
        match command {
            SemesterCommands::List => self.list(),
            SemesterCommands::Add {
                number,
                study_cycle,
            } => self.add(number, study_cycle.map(|it| StudyCycle::from_do(it))),
            SemesterCommands::Remove { name } => self.remove(name),
        }
    }

    fn list(&self) -> ServiceResult {
        // Collect and sort semester names
        let mut semester_names: Vec<String> = self
            .store
            .semesters()
            .map(|semester| semester.name())
            .collect();
        semester_names.sort();

        if semester_names.is_empty() {
            bail!("No semesters found!")
        }

        let res = if let Some(active_semester) = self.store.current_semester() {
            let active = semester_names
                .iter()
                .map(|course_name| {
                    if course_name == &active_semester.name() {
                        "*".to_string()
                    } else {
                        " ".to_string()
                    }
                })
                .collect::<Vec<_>>();

            table!("active", "courses" ; active, semester_names ; FormatAlignment::Center, FormatAlignment::Left)
        } else {
            table!("courses"; semester_names; FormatAlignment::Left)
        };
        Ok(res)
    }

    fn add(&mut self, number: u16, study_cycle: Option<StudyCycle>) -> ServiceResult {
        let study_cycle =
            study_cycle.or_else(|| self.store.current_semester().map(|it| it.study_cycle()));
        let Some(cycle) = study_cycle else {
            bail!("A study cycle must be provided as currently no semester is active.");
        };

        let path = self
            .store
            .entry_point()
            .create_semester_path(number, cycle)?;

        // make sure everything is set up correctly
        let sememester = self
            .store
            .get_semester(path.name())
            .ok_or_else(|| anyhow!("Failed to retrieve newly created semester"))?;
        Ok(format!("{} was created.", sememester.name()).success())
    }

    fn remove(&mut self, name: String) -> ServiceResult {
        let dialog = vec![
            DialogEntry::YesNoInput(format!("Are you sure that you want to permanently remove semester '{}' with all its courses? This action can not be reverted",name))
        ];
        let response = FormatService::dialog(dialog);
        if let Some(res) = response {
            let res = res
                .first()
                .ok_or_else(|| anyhow!("Dialog has not returned not the specified output"))?;
            let DialogOutput::YesNo(cond) = res else {
                bail!("Invalid Input")
            };

            if *cond {
                let semester = self
                    .store
                    .get_semester(&name)
                    .with_context(|| anyhow!("Semester could not be found"))?;
                semester.path().clone().remove()?;
                Ok(format!("Semester '{}' has been removed", name).success())
            } else {
                Ok("Operation has been canceled".info())
            }
        } else {
            Ok("Operation has been canceled".info())
        }
    }
}
