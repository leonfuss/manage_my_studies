use anyhow::Result;

use crate::{
    cli::{Cli, Commands},
    StoreProvider,
};

use super::switch::SwitchService;
use super::{course::CourseService, semester::SemesterService, status::StatusService};

pub struct Service<Store>
where
    Store: StoreProvider,
{
    store: Store,
}

impl<Store> Service<Store>
where
    Store: StoreProvider,
{
    pub fn new(store: Store) -> Service<Store> {
        Service { store }
    }

    pub fn run(&mut self, args: Cli) -> Result<()> {
        match args.command {
            Commands::Semester { command } => SemesterService::new(&mut self.store).run(command)?,
            Commands::Course { command } => CourseService::new(&mut self.store).run(command)?,
            Commands::Switch { reference } => SwitchService::new(&mut self.store).run(reference)?,
            Commands::Status {} => StatusService::new(&self.store).run()?,
            _ => {}
        }
        Ok(())
    }
}
