use std::{
    ops::Deref,
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::{anyhow, bail, Context, Ok, Result};
use serde::{de::DeserializeOwned, Serialize};
use walkdir::WalkDir;

use super::{config::SemesterNames, StudyCycle};

/// The entry point to the university data.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct EntryPoint(PathBuf);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StoreDataFile(PathBuf);

impl EntryPoint {
    pub fn new(path: &str) -> Result<EntryPoint> {
        let path = PathBuf::from_str(path)?;
        Self::from_path(path)
    }

    pub fn from_path<P>(path: P) -> Result<EntryPoint>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        if path.exists() && path.is_dir() {
            Ok(EntryPoint(path.to_path_buf()))
        } else {
            bail!(
                "The entry point '{}' is not a valid directory.",
                path.display()
            )
        }
    }

    /// Returns the path to the store data file.
    /// If the file does not exist, it will be created.
    pub fn data_file(&self) -> Result<StoreDataFile> {
        let path = self.0.join(".mm");
        if !path.exists() && !path.is_file() {
            std::fs::write(&path, "").with_context(|| {
                anyhow!("Failed to create store data file at: {}", path.display())
            })?;
        }
        Ok(StoreDataFile(path))
    }

    pub fn semester_path(
        &self,
        name: &str,
        semester_names: &SemesterNames,
    ) -> Option<SemesterPath> {
        if semester_names.is_name(name) {
            let path = self.0.join(name);
            if path.exists() && path.is_dir() {
                return Some(SemesterPath(path, name.to_string()));
            }
        }
        None
    }

    pub fn create_semester_path(
        &self,
        semester_number: u16,
        study_cycle: StudyCycle,
    ) -> Result<SemesterPath> {
        let name = format!("{}{}", study_cycle, semester_number);
        let path = self.0.join(&name);
        if path.exists() {
            bail!("The semester path '{}' already exists.", path.display());
        }
        std::fs::create_dir(&path)
            .with_context(|| anyhow!("Failed to create semester path at: {}", path.display()))?;
        Ok(SemesterPath(path, name))
    }

    pub fn semester_paths<'a>(
        &'a self,
        semester_names: &'a SemesterNames,
    ) -> impl Iterator<Item = SemesterPath> + 'a {
        WalkDir::new(&self.0)
            .max_depth(1)
            .min_depth(1)
            .into_iter()
            .filter_map(move |entry| {
                let entry = entry.ok()?;
                let name = entry.file_name().to_string_lossy().to_string();
                if semester_names.is_name(&name) {
                    Some(SemesterPath(entry.path().to_path_buf(), name))
                } else {
                    None
                }
            })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SemesterPath(PathBuf, String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SemesterDataFile(PathBuf);

impl SemesterPath {
    pub fn name(&self) -> &str {
        &self.1
    }

    pub fn path(&self) -> &PathBuf {
        &self.0
    }

    pub fn data_file(&self) -> Result<SemesterDataFile> {
        let path = self.0.join(".mm");
        if !path.exists() && !path.is_file() {
            std::fs::write(&path, "").with_context(|| {
                anyhow!("Failed to create semester data file at: {}", path.display())
            })?;
        }
        Ok(SemesterDataFile(path))
    }

    pub fn course_paths(&self) -> impl Iterator<Item = CoursePath> {
        WalkDir::new(&self.0)
            .max_depth(1)
            .min_depth(1)
            .into_iter()
            .filter_map(|entry| {
                let entry = entry.ok()?;
                if entry.file_type().is_dir() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    Some(CoursePath(entry.path().to_path_buf(), name))
                } else {
                    None
                }
            })
    }

    pub fn course_path(&self, name: &str) -> Option<CoursePath> {
        let path = self.0.join(name);
        if path.exists() && path.is_dir() {
            Some(CoursePath(path, name.to_string()))
        } else {
            None
        }
    }

    pub fn remove(self) -> Result<()> {
        std::fs::remove_dir_all(&self.0)
            .with_context(|| anyhow!("Failed to remove semester path at: {}", self.0.display()))?;
        Ok(())
    }

    pub fn create_course_path(&self, name: &str) -> Result<CoursePath> {
        let path = self.0.join(&name);
        if path.exists() {
            bail!("The course path '{}' already exists.", path.display());
        }
        std::fs::create_dir(&path)
            .with_context(|| anyhow!("Failed to create semester path at: {}", path.display()))?;

        Ok(CoursePath(path, name.into()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CoursePath(PathBuf, String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CourseDataFile(PathBuf);

impl CoursePath {
    pub fn data_file(&self) -> Result<CourseDataFile> {
        let path = self.0.join("course.toml");
        if !path.exists() && !path.is_file() {
            std::fs::write(&path, include_str!("../../course.toml")).with_context(|| {
                anyhow!("Failed to create course data file at: {}", path.display())
            })?;
        }
        Ok(CourseDataFile(path))
    }

    pub fn remove(self) -> Result<()> {
        std::fs::remove_dir_all(&self.0)
            .with_context(|| anyhow!("Failed to remove course path at: {}", self.0.display()))?;
        Ok(())
    }

    pub fn name(&self) -> &str {
        &self.1
    }
}

impl Deref for EntryPoint {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for CoursePath {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A path that may can be turned into a symlink.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct MaybeSymLinkable(Option<PathBuf>);

impl MaybeSymLinkable {
    pub fn new<P>(path: Option<P>) -> Result<MaybeSymLinkable>
    where
        P: AsRef<Path>,
    {
        let path = path.map(|p| p.as_ref().to_path_buf());

        if let Some(p) = &path {
            if p.exists() && p.is_symlink() {
                return Ok(MaybeSymLinkable(path));
            } else {
                bail!(
                    "The path '{}' already exists and is not a symblink",
                    p.display()
                )
            }
        } else {
            Ok(MaybeSymLinkable(None))
        }
    }

    pub fn link_from<P>(&self, original: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        self.remove_link()?;
        if let Some(path) = &self.0 {
            #[cfg(unix)]
            {
                std::os::unix::fs::symlink(&original, path)?;
            }

            #[cfg(windows)]
            {
                std::os::windows::fs::symlink_dir(&original, path)?;
            }
        }
        Ok(())
    }

    pub fn remove_link(&self) -> Result<()> {
        if let Some(path) = &self.0 {
            if path.is_symlink() {
                std::fs::remove_file(path)?;
            }
        }
        Ok(())
    }
}

pub(crate) trait ReadWriteDO: Deref<Target = PathBuf> {
    type Object: DeserializeOwned + Serialize;
    fn read(&self) -> Result<Self::Object> {
        let content = std::fs::read_to_string(self.deref())
            .with_context(|| anyhow!("Failed to read file at: {}", self.deref().display()))?;
        let it: Self::Object = toml_edit::de::from_str::<Self::Object>(&content)
            .with_context(|| anyhow!("Failed to parse data from: {}", self.deref().display()))?;
        Ok(it)
    }

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

impl Deref for StoreDataFile {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
