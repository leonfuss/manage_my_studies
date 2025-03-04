#![feature(type_alias_impl_trait)]
#![feature(int_roundings)]

use anyhow::Result;

mod cli;
mod domain;
mod provider;
mod service;

use clap::Parser;
use cli::Cli;
use domain::{Config, Store};
pub(crate) use provider::*;
use service::Service;

fn main() -> Result<()> {
    let config = Config::new()?;
    let store = Store::new(config)?;
    let args = Cli::parse();
    let mut service = Service::new(store);

    service.run(args);

    Ok(())
}
