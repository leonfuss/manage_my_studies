use std::fmt;

use clap::{Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};

#[derive(Debug, Parser)]
#[command(name = "mm")]
#[command(about = "Manage my studies", version = "0.2.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[command(about = "Show the current active semester or course")]
    #[command(alias = "s")]
    Status {},
    #[command(about = "Switch to a semester or course")]
    #[command(alias = "sw")]
    Switch { reference: Option<String> },
    #[command(about = "Manage semesters")]
    #[command(alias = "se")]
    Semester {
        #[command(subcommand)]
        command: Option<SemesterCommands>,
    },
    #[command(about = "Manage courses")]
    #[command(alias = "co")]
    Course {
        #[command(subcommand)]
        command: Option<CourseCommands>,
    },
    #[command(about = "exercises")]
    #[command(alias = "ex")]
    Exercise {
        #[command(subcommand)]
        command: ExerciseCommands,
    },
    #[command(about = "Change configuration (to be implemented)")]
    Config {
        #[command(subcommand)]
        command: Option<ConfigCommands>,
    },
}

#[derive(Debug, Subcommand)]
pub enum SemesterCommands {
    List,
    Add {
        number: u16,
        study_cycle: Option<StudyCycleDO>,
    },
    Remove {
        name: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum CourseCommands {
    List,
    Add {
        #[arg(value_name = "COURSE_NAME")]
        name: String,
    },
    Remove {
        #[arg(value_name = "COURSE_NAME")]
        name: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum ExerciseCommands {
    List,
    Add { name: Option<String> },
    Remove { name: String },
    Move { from: Option<String>, to: String },
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommands {
    List,
    Set { key: String, value: String },
    Remove { key: String },
}

#[derive(Debug, Serialize, Deserialize, ValueEnum, Clone, Copy, PartialEq, Eq)]
pub enum StudyCycleDO {
    Bachelor,
    Master,
    Doctorate,
}

impl fmt::Display for StudyCycleDO {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let cycle_str = match self {
            StudyCycleDO::Bachelor => "Bachelor",
            StudyCycleDO::Master => "Master",
            StudyCycleDO::Doctorate => "Doctorate",
        };
        write!(f, "{}", cycle_str)
    }
}
