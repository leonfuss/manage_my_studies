use crate::{
    cli::{Cli, Commands},
    StoreProvider,
};

use super::{
    course::CourseService, format::FormatService, semester::SemesterService, status::StatusService,
};
use super::{switch::SwitchService, ServiceResult};

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

    pub fn run(&mut self, args: Cli) {
        let res: ServiceResult = match args.command {
            Commands::Semester { command } => SemesterService::new(&mut self.store).run(command),
            Commands::Course { command } => CourseService::new(&mut self.store).run(command),
            Commands::Switch { reference } => SwitchService::new(&mut self.store).run(reference),
            Commands::Status {} => StatusService::new(&self.store).run(),
            _ => todo!(),
        };

        FormatService::run(res);
    }
}
