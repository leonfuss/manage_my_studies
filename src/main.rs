use anyhow::Result;

use clap::Parser;
use cli::*;
use config::Config;
use course::handle_course;
use semester::handle_semester;
use status::handle_status;
use switch::handle_switch;

mod cli;
mod config;
mod course;
mod semester;
mod status;
mod switch;

fn main() -> Result<()> {
    let mut config = Config::new()?;

    let args = Cli::parse();

    match args.command {
        Commands::Semester { command } => handle_semester(&mut config, command)?,
        Commands::Switch { reference } => handle_switch(&mut config, reference)?,
        Commands::Course { command } => handle_course(&mut config, command)?,
        Commands::Status {} => handle_status(&config)?,
        _ => {}
    }

    Ok(())
}
