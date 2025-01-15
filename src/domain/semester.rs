use core::fmt;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::cli::StudyCycleDO;

use super::{
    config::SemesterNames,
    course::Course,
    paths::{CoursePath, ReadWriteDO, SemesterDataFile, SemesterPath},
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct Semester {
    semester_number: u16,
    study_cycle: StudyCycle,
    path: SemesterPath,
    active_course: Option<CoursePath>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SemesterDO {
    active_course: Option<String>,
}

impl Semester {
    pub fn from_path(path: SemesterPath, semester_names: &SemesterNames) -> Result<Semester> {
        let data_file = path.data_file()?;
        let semester_do = data_file.read()?;
        let active_course = semester_do
            .active_course
            .map(|it| path.course_path(&it))
            .flatten();
        let (semester_number, study_cycle) = semester_names.deserialize(path.name())?;
        let semester = Semester {
            semester_number,
            study_cycle,
            path,
            active_course,
        };
        Ok(semester)
    }

    pub fn active_course(&self) -> Option<Course> {
        self.active_course
            .as_ref()
            .map(|it| Course::from_path(it.clone()).ok())
            .flatten()
    }

    pub fn courses(&self) -> impl Iterator<Item = Course> {
        self.path
            .course_paths()
            .filter_map(|path| Course::from_path(path).ok())
    }

    pub fn course(&self, name: &str) -> Option<Course> {
        self.path
            .course_path(name)
            .map(|path| Course::from_path(path).ok())
            .flatten()
    }

    /// Does not perform symlink operations.
    /// Call via store to ensure symlink operations are performed.
    pub(super) fn set_active(&mut self, course: Option<&Course>) -> Result<()> {
        self.active_course = course.map(|it| it.path().clone());
        self.path.data_file()?.write(&self.to_do())
    }

    fn to_do(&self) -> SemesterDO {
        let active_course = self.active_course.as_ref().map(|it| it.name().to_string());
        SemesterDO { active_course }
    }

    pub fn path(&self) -> &SemesterPath {
        &self.path
    }

    pub fn name(&self) -> String {
        format!(
            "{}{:02}",
            self.study_cycle.abbreviation(),
            self.semester_number
        )
    }

    pub fn study_cycle(&self) -> StudyCycle {
        self.study_cycle
    }
}

impl SemesterDO {}

impl ReadWriteDO for SemesterDataFile {
    type Object = SemesterDO;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StudyCycle {
    Bachelor,
    Master,
    Doctorate,
}

impl StudyCycle {
    pub fn from_do(study_cycle: StudyCycleDO) -> StudyCycle {
        match study_cycle {
            StudyCycleDO::Bachelor => StudyCycle::Bachelor,
            StudyCycleDO::Master => StudyCycle::Master,
            StudyCycleDO::Doctorate => StudyCycle::Doctorate,
        }
    }
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
