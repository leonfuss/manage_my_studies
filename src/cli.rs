use std::fmt;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(name = "mm")]
#[command(about = "Manage my studies")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Status {},
    Switch {
        reference: Option<String>,
    },
    Semester {
        #[command(subcommand)]
        command: Option<SemesterCommands>,
    },
    Course {
        #[command(subcommand)]
        command: Option<CourseCommands>,
    },
    Exercise {
        #[command(subcommand)]
        command: ExerciseCommands,
    },
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

    pub fn to_base(&self) -> u16 {
        match self {
            StudyCycle::Bachelor => 100,
            StudyCycle::Master => 200,
            StudyCycle::Doctorate => 300,
        }
    }
}
