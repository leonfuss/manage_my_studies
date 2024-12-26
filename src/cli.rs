use std::fmt;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(name = "mm")]
#[command(about = "Manage my studies", version = "0.1.0")]
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
    #[command(about = "Chnage configuration (to be implemented)")]
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
        study_cycle: Option<StudyCycle>,
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
    Move {
        #[arg(value_name = "TO")]
        to: String,
        #[arg(value_name = "FROM")]
        from: Option<String>,
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

#[derive(Debug, ValueEnum, Clone, Copy, PartialEq, Eq)]
pub enum StudyCycle {
    Bachelor,
    Master,
    Doctorate,
}

impl fmt::Display for StudyCycle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let cycle_str = match self {
            StudyCycle::Bachelor => "Bachelor",
            StudyCycle::Master => "Master",
            StudyCycle::Doctorate => "Doctorate",
        };
        write!(f, "{}", cycle_str)
    }
}

impl StudyCycle {
    pub fn abbreviation(&self) -> &'static str {
        match self {
            StudyCycle::Bachelor => "b",
            StudyCycle::Master => "m",
            StudyCycle::Doctorate => "d",
        }
    }
}
