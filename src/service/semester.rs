use crate::{
    cli::SemesterCommands, domain::StudyCycle, service::format::DialogEntry, StoreProvider,
};
use anyhow::{anyhow, Context, Result};

use super::format::{DialogOutput, FormatService};

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

    pub fn run(&mut self, command: Option<SemesterCommands>) -> Result<()> {
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

    fn list(&self) -> Result<()> {
        // Collect and sort semester names
        let mut semester_names: Vec<String> = self
            .store
            .semesters()
            .map(|semester| semester.name())
            .collect();
        semester_names.sort();

        if semester_names.is_empty() {
            FormatService::info("No semesters found");
            return Ok(());
        }

        // Find index of active semester
        let active_idx = self.store.current_semester().and_then(|active_sem| {
            semester_names
                .iter()
                .position(|name| name == &active_sem.name())
        });

        // Create active checker function
        let is_active =
            move |idx: usize| -> bool { active_idx.map_or(false, |active| idx == active) };

        FormatService::active_item_table(semester_names, Box::new(is_active));
        Ok(())
    }

    fn add(&mut self, number: u16, study_cycle: Option<StudyCycle>) -> Result<()> {
        let study_cycle =
            study_cycle.or_else(|| self.store.current_semester().map(|it| it.study_cycle()));
        let Some(cycle) = study_cycle else {
            FormatService::error(
                "A study cycle must be provided as currently no semester is active.",
            );
            return Ok(());
        };

        let path = match self.store.entry_point().create_semester_path(number, cycle) {
            Ok(path) => path,
            Err(e) => {
                FormatService::error(&e.to_string());
                return Ok(());
            }
        };

        // make sure everything is set up correctly
        self.store
            .get_semester(path.name())
            .ok_or_else(|| anyhow!("Failed to retrieve newly created semester"))?;
        Ok(())
    }

    fn remove(&mut self, name: String) -> Result<()> {
        let dialog = vec![
            DialogEntry::YesNoInput(format!("Are you sure that you want to permanently remove semester '{}' with all its courses? This action can not be reverted",name))
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
                let semester = self
                    .store
                    .get_semester(&name)
                    .with_context(|| anyhow!("Semester could not be found"))?;
                semester.path().clone().remove()?;
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
