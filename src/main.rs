use anyhow::Result;

use clap::Parser;
use cli::*;

mod cli;
mod config;
mod course;
mod semester;
mod status;
mod store;
mod switch;
mod util;

fn main() -> Result<()> {
    let config_path = config::get_config_path()?;
    let entry_point = config::get_entry_point(&config_path)?;

    let mut store = store::Store::new(entry_point)?;

    let args = Cli::parse();

    match args.command {
        Commands::Semester { command } => semester::semester(&mut store, command)?,
        Commands::Switch { reference } => switch::switch(&mut store, reference)?,
        Commands::Course { command } => course::course(&mut store, command)?,
        Commands::Status {} => status::status(&store)?,
        _ => {}
    }

    Ok(())
}
