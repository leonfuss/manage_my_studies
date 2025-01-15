use std::ops::Deref;

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use super::paths::{CourseDataFile, CoursePath, ReadWriteDO};

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct Course {
    path: CoursePath,
    grade: Option<f32>,
    ects: Option<u8>,
    name: Option<String>,
    degrees: Option<Vec<String>>,
    uebk: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CourseDO {
    name: Option<String>,
    grade: Option<f32>,
    ects: Option<u8>,
    degrees: Option<Vec<String>>,
    #[serde(rename = "übK")]
    uebk: Option<bool>,
}

impl Course {
    pub fn from_path(path: CoursePath) -> Result<Course> {
        let data = path.data_file()?;
        let course_do = data.read()?;
        let course = Course {
            path,
            grade: course_do.grade,
            ects: course_do.ects,
            name: course_do.name,
            uebk: course_do.uebk,
            degrees: course_do.degrees,
        };
        Ok(course)
    }

    pub fn path(&self) -> &CoursePath {
        &self.path
    }

    pub fn name(&self) -> String {
        self.name
            .as_deref()
            .map(str::to_owned)
            .unwrap_or_else(|| format!("[{}]", self.path().name()))
    }

    pub fn grade(&self) -> Option<f32> {
        self.grade
    }

    pub fn ects(&self) -> Option<u8> {
        self.ects
    }

    pub fn degrees(&self) -> &Vec<String> {
        static EMPTY: Vec<String> = Vec::new();
        self.degrees.as_ref().unwrap_or(&EMPTY)
    }

    pub fn uebk(&self) -> Option<bool> {
        self.uebk
    }
}

impl ReadWriteDO for CourseDataFile {
    type Object = CourseDO;

    fn write(&self, object: &Self::Object) -> Result<()> {
        let data = toml_edit::ser::to_string(&object).with_context(|| {
            anyhow!(
                "Failed to serialize data to toml for: {}",
                self.deref().display()
            )
        })?;
        std::fs::write(self.deref(), data)
            .with_context(|| anyhow!("Failed to write data to file: {}", self.deref().display()))?;
        Ok(())
    }
}
