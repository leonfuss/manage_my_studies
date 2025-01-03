use std::{
    ops::Deref,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context, Result};
use either::Either;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::{course::Course, semester::Semester, StudyCycle};

#[derive(Debug, Deserialize, Serialize)]
pub struct Store {
    #[serde(skip)]
    #[serde(default)]
    entry_point: PathBuf,
    active_semester: Option<String>,
}

impl Store {
    pub fn new(entry_point: PathBuf) -> Result<Store> {
        if !entry_point.exists() {
            return Err(anyhow!("Store path does not exist"));
        }

        let file = entry_point.store_data_file();
        file.ensure_exists()?;
        let mut store = file.read()?;
        store.entry_point = entry_point;
        Ok(store)
    }

    pub fn active_semester(&self) -> Option<Semester> {
        if let Some(ref name) = self.active_semester {
            self.get_semester(name).ok()
        } else {
            None
        }
    }

    pub fn entry_point(&self) -> &PathBuf {
        &self.entry_point
    }
}

impl Store {
    pub fn get_semester(&self, name: &str) -> Result<Semester> {
        Semester::new(self.entry_point.join(name))
    }

    pub fn semesters(&self) -> impl Iterator<Item = Semester> + '_ {
        WalkDir::new(&self.entry_point)
            .min_depth(1)
            .max_depth(1)
            .sort_by_file_name()
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter(|entry| self.is_semester(entry.path()))
            .filter_map(|entry| Semester::new(entry.path()).ok())
    }

    fn is_semester<P>(&self, path: P) -> bool
    where
        P: AsRef<std::path::Path>,
    {
        let path = path.as_ref();
        let parent = path.parent();
        if !path.is_dir() || parent != Some(&self.entry_point) {
            return false;
        }
        match path.file_name().and_then(|it| it.to_str()) {
            Some(name) => match name.chars().next() {
                Some('b') | Some('m') | Some('d') => name.chars().skip(1).all(|c| c.is_digit(10)),
                _ => false,
            },
            None => false,
        }
    }

    pub fn get_by_reference(
        &self,
        reference: &str,
    ) -> Option<Either<Semester, (Semester, Course)>> {
        let split = reference.split('/').collect::<Vec<&str>>();
        match split.len() {
            0 => return None,
            1 => {
                if let Ok(semester) = self.get_semester(reference) {
                    return Some(Either::Left(semester));
                } else {
                    if let Some(active_semester) = self.active_semester() {
                        return active_semester
                            .get_course(reference)
                            .ok()
                            .map(|course| Either::Right((active_semester, course)));
                    };

                    self.semesters()
                        .filter_map(|semester| {
                            let course =
                                semester.courses().find(|course| course.name() == reference);
                            course.map(|course| (semester, course))
                        })
                        .map(Either::Right)
                        .next()
                }
            }
            2 => {
                let semester = self.get_semester(split[0]).ok()?;
                let course = semester.get_course(split[1]).ok()?;
                return Some(Either::Right((semester, course)));
            }
            _ => return None,
        }
    }

    pub fn add_semester(
        &self,
        semester_number: u16,
        cycle: Option<StudyCycle>,
    ) -> Result<Semester> {
        let cycle = cycle
            .or_else(|| self.active_semester().map(|it| it.study_cycle()))
            .ok_or_else(|| {
                anyhow!("A study cycle must be provided as currently no semester is active.")
            })?;
        let name = Semester::format_name(cycle, semester_number);
        let path = self.entry_point.join(&name);
        if path.exists() {
            return Err(anyhow!("Semester '{}' already exists", name));
        }
        std::fs::create_dir(&path)?;
        Semester::new(path)
    }

    pub fn remove_semester(&mut self, semester: Semester) -> Result<()> {
        if self.is_active(&semester) {
            self.active_semester = None;
        }
        std::fs::remove_dir_all(semester.path())?;
        Ok(())
    }

    pub fn is_active(&self, semester: &Semester) -> bool {
        self.active_semester()
            .map(|it| it.path() == semester.path())
            .unwrap_or(false)
    }

    pub fn set_active(&mut self, semester: Option<&Semester>) -> Result<()> {
        self.active_semester = semester.map(|it| it.name().to_owned());
        self.write()
    }

    fn write(&self) -> Result<()> {
        let path = self.entry_point.store_data_file();
        path.ensure_exists()?;
        path.write(&self)
    }
}

trait FileMarker {}

pub struct StoreDataFile(PathBuf);
pub struct SemesterDataFile(PathBuf);
pub struct CourseDataFile(PathBuf);

impl FileMarker for StoreDataFile {}
impl FileMarker for SemesterDataFile {}

pub trait Files {
    fn store_data_file(&self) -> StoreDataFile;
    fn semester_data_file(&self) -> SemesterDataFile;
    fn course_data_file(&self) -> CourseDataFile;
}

pub trait ReadWriteData {
    type Object;
    fn read(&self) -> Result<Self::Object>;
    fn write(&self, object: &Self::Object) -> Result<()>;
}

pub trait EnsureExistance {
    fn ensure_exists(&self) -> Result<()>;
}

impl<F> EnsureExistance for F
where
    F: Deref<Target = PathBuf> + FileMarker,
{
    fn ensure_exists(&self) -> Result<()> {
        let path: &Path = self.deref().as_ref();
        if !path.exists() {
            std::fs::write(path, "")
                .with_context(|| anyhow!("Failed to create data file at: {}", path.display()))?;
        }
        Ok(())
    }
}

impl EnsureExistance for CourseDataFile {
    fn ensure_exists(&self) -> Result<()> {
        let path: &Path = self.deref().as_ref();
        if !path.exists() {
            std::fs::write(path, include_str!("../course.toml"))
                .with_context(|| anyhow!("Failed to create data file at: {}", path.display()))?;
        }
        Ok(())
    }
}

impl ReadWriteData for StoreDataFile {
    type Object = Store;

    fn read(&self) -> Result<Self::Object> {
        let content = std::fs::read_to_string(self.deref())
            .with_context(|| anyhow!("Failed to read file at: {}", self.deref().display()))?;
        let store: Store = toml_edit::de::from_str(&content).with_context(|| {
            anyhow!(
                "Failed to parse Store data from: {}",
                self.deref().display()
            )
        })?;
        Ok(store)
    }

    fn write(&self, object: &Self::Object) -> Result<()> {
        let data = toml_edit::ser::to_string(&object).with_context(|| {
            anyhow!(
                "Failed to serialize Store data to toml for: {}",
                self.deref().display()
            )
        })?;
        std::fs::write(self.deref(), data).with_context(|| {
            anyhow!(
                "Failed to write Store data to file: {}",
                self.deref().display()
            )
        })?;
        Ok(())
    }
}

impl<P> Files for P
where
    P: AsRef<Path>,
{
    /// Should only be called on the entry point
    fn store_data_file(&self) -> StoreDataFile {
        StoreDataFile(self.as_ref().join(".mm"))
    }

    /// Should only be called on a valid semester folder
    fn semester_data_file(&self) -> SemesterDataFile {
        SemesterDataFile(self.as_ref().join(".mm"))
    }

    /// Should only be called on a valid course folder
    fn course_data_file(&self) -> CourseDataFile {
        CourseDataFile(self.as_ref().join("course.toml"))
    }
}

impl Deref for StoreDataFile {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for SemesterDataFile {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for CourseDataFile {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
