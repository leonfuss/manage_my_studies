use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use either::Either;
use walkdir::WalkDir;

use crate::{util, StudyCycle};

#[derive(Debug)]
pub struct Store {
    entry_point: PathBuf,
    active_semester: Option<Semester>,
}

#[derive(Debug, Clone)]
pub struct Semester {
    semester_number: u16,
    cycle: StudyCycle,
    path: PathBuf,
    active_course: Option<Course>,
}

#[derive(Debug, Clone)]
pub struct Course {
    name: String,
    path: PathBuf,
}

impl Store {
    pub fn new(entry_point: PathBuf) -> Result<Store> {
        if !entry_point.exists() {
            return Err(anyhow::anyhow!("Store path does not exist"));
        }
        let data_file = entry_point.join(".mm");
        let active_semester = Store::get_active_semester_from_file(&data_file)?.and_then(|name| {
            match Store::_get_semester(&name, &entry_point) {
                Ok(semester) => Some(semester),
                Err(e) => {
                    eprintln!("Failed to load active semester: {}", e);
                    None
                }
            }
        });

        let store = Store {
            entry_point,
            active_semester,
        };
        Ok(store)
    }

    pub fn active_semester(&self) -> Option<&Semester> {
        self.active_semester.as_ref()
    }

    pub fn active_semester_mut(&mut self) -> Option<&mut Semester> {
        self.active_semester.as_mut()
    }

    fn get_active_semester_from_file(path: &PathBuf) -> Result<Option<String>> {
        util::enusure_file_exists(path)?;
        let toml = util::toml_from_path(path)?;
        let semester = toml
            .get("semester")
            .and_then(|it| it.as_str())
            .map(String::from);
        Ok(semester)
    }

    fn _get_semester(name: &str, entry_point: &PathBuf) -> Result<Semester> {
        let path = entry_point.join(name);
        if !path.exists() {
            return Err(anyhow::anyhow!("Semester '{}' does not exist", name));
        }
        let (cycle, number) = match name
            .chars()
            .next()
            .ok_or_else(|| anyhow!("Semester name is empty"))?
        {
            'b' => (StudyCycle::Bachelor, &name[1..]),
            'm' => (StudyCycle::Master, &name[1..]),
            'd' => (StudyCycle::Doctorate, &name[1..]),
            _ => return Err(anyhow::anyhow!("Invalid semester name '{}'", name)),
        };
        let number = number.parse::<u16>().with_context(|| {
            anyhow!(
                "Failed to parse number: {} in semester name {}",
                number,
                name
            )
        })?;
        let semester = Semester::new(path, number, cycle)?;
        Ok(semester)
    }

    pub fn semesters(&self) -> impl Iterator<Item = Semester> + '_ {
        WalkDir::new(&self.entry_point)
            .min_depth(1)
            .max_depth(1)
            .sort_by_file_name()
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter(|entry| self.is_semester(entry.path()))
            .filter_map(|entry| {
                let name = entry.file_name().to_string_lossy().to_string();
                Store::_get_semester(&name, &self.entry_point).ok()
            })
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

    pub fn get_semester(&self, name: &str) -> Option<Semester> {
        Store::_get_semester(name, &self.entry_point).ok()
    }

    pub fn get_by_reference(
        &self,
        reference: &str,
    ) -> Option<Either<Semester, (Semester, Course)>> {
        let split = reference.split('/').collect::<Vec<&str>>();
        match split.len() {
            0 => return None,
            1 => {
                if let Some(semester) = self.get_semester(reference) {
                    return Some(Either::Left(semester));
                } else {
                    if let Some(active_semester) = &self.active_semester {
                        return active_semester
                            .get_course(reference)
                            .ok()
                            .map(|course| Either::Right((active_semester.clone(), course)));
                    };

                    self.semesters()
                        .flat_map(|semester| {
                            let semester_clone = semester.clone();
                            semester
                                .courses()
                                .map(move |course| (semester_clone.clone(), course))
                        })
                        .find(|(_, course)| course.name == reference)
                        .map(Either::Right)
                }
            }
            2 => {
                let semester = self.get_semester(split[0])?;
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
            .or_else(|| self.active_semester.as_ref().map(|it| it.study_cycle()))
            .ok_or_else(|| {
                anyhow!("A study cycle must be provided as currently no semester is active.")
            })?;
        let name = Semester::file_name(cycle, semester_number);
        let path = self.entry_point.join(&name);
        if path.exists() {
            return Err(anyhow!("Semester '{}' already exists", name));
        }
        std::fs::create_dir(&path)?;
        Semester::new(path, semester_number, cycle)
    }

    pub fn remove_semester(&mut self, semester: Semester) -> Result<()> {
        if self.is_active(&semester) {
            self.active_semester = None;
        }
        std::fs::remove_dir_all(semester.path())?;
        Ok(())
    }

    pub fn is_active(&self, semester: &Semester) -> bool {
        self.active_semester
            .as_ref()
            .map(|it| it.path == semester.path)
            .unwrap_or(false)
    }

    pub fn set_active(&mut self, semester: Option<&Semester>) -> Result<()> {
        self.active_semester = semester.cloned();
        self.write()
    }

    fn write(&self) -> Result<()> {
        let path = self.entry_point.join(".mm");
        let mut data = util::toml_from_path(&path)?;
        if let Some(ref semester) = self.active_semester {
            data["semester"] = toml_edit::value(semester.name());
        }
        std::fs::write(&path, data.to_string())?;
        Ok(())
    }
}

impl Semester {
    pub fn new(path: PathBuf, semester_number: u16, cycle: StudyCycle) -> Result<Semester> {
        let data_path = path.join(".mm");
        util::enusure_file_exists(&data_path)?;
        let toml = util::toml_from_path(&data_path)?;
        let active_course = toml
            .get("course")
            .and_then(|it| it.as_str())
            .map(String::from)
            .and_then(|name| {
                let path = path.join(&name);
                if path.exists() {
                    Some(Course::new(path, name))
                } else {
                    None
                }
            });

        let semester = Semester {
            semester_number,
            path,
            cycle,
            active_course,
        };
        Ok(semester)
    }
    pub fn get_course(&self, name: &str) -> Result<Course> {
        let path = self.path().join(name);
        if !path.exists() {
            return Err(anyhow!("Course '{}' does not exist", name));
        }
        let course = Course::new(path, name.into());
        Ok(course)
    }

    pub fn is_active(&self, coures: &Course) -> bool {
        self.active_course
            .as_ref()
            .map(|it| it.path == coures.path)
            .unwrap_or(false)
    }

    pub fn courses(&self) -> impl Iterator<Item = Course> + 'static {
        WalkDir::new(&self.path)
            .min_depth(1)
            .max_depth(1)
            .sort_by_file_name()
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().is_dir())
            .filter_map(|entry| {
                let name = entry.file_name().to_string_lossy().to_string();
                Some(Course::new(entry.path().to_path_buf(), name))
            })
    }
    pub fn add_course(&self, name: &str) -> Result<Course> {
        let path = self.path().join(name);
        if path.exists() {
            return Err(anyhow!("Course '{}' already exists", name));
        }

        std::fs::create_dir(&path)?;
        let course_toml_path = path.join(".course.toml");
        std::fs::write(&course_toml_path, include_str!("../course.toml"))?;
        let course = Course::new(path, name.into());
        Ok(course)
    }

    pub fn remove_course(&mut self, course: Course) -> Result<()> {
        if self.is_active(&course) {
            self.active_course = None;
        }
        std::fs::remove_dir_all(course.path())?;
        Ok(())
    }

    pub fn move_course(&mut self, course: Course, to: &str) -> Result<Course> {
        let to_path = self.path().join(&to);
        if to_path.exists() {
            return Err(anyhow!("A course with the name '{}' already exists", to));
        }
        std::fs::rename(course.path(), &to_path)?;
        let new_course = Course::new(to_path, to.to_owned());
        if self.is_active(&course) {
            self.active_course = Some(new_course.clone());
        }
        Ok(new_course)
    }
    pub fn active(&self) -> Option<&Course> {
        self.active_course.as_ref()
    }
    pub fn set_active(&mut self, course: Option<&Course>) -> Result<()> {
        self.active_course = course.cloned();
        self.write()
    }

    fn write(&self) -> Result<()> {
        let path = self.path().join(".mm");
        let mut data = util::toml_from_path(&path)?;
        if let Some(ref course) = self.active_course {
            data["course"] = toml_edit::value(course.name.clone());
        }
        std::fs::write(&path, data.to_string())?;
        Ok(())
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }
    pub fn study_cycle(&self) -> StudyCycle {
        self.cycle
    }

    pub fn name(&self) -> String {
        Semester::file_name(self.cycle, self.semester_number)
    }
    pub fn file_name(cycle: StudyCycle, semester_number: u16) -> String {
        format!("{}{:02}", cycle.abbreviation(), semester_number)
    }
}

impl Course {
    pub fn new(path: PathBuf, name: String) -> Course {
        Course { path, name }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn semester(&self, store: &Store) -> Result<Semester> {
        let parent = self
            .path
            .parent()
            .ok_or_else(|| anyhow!("Course path has no parent"))?;
        let name = parent
            .file_name()
            .ok_or_else(|| anyhow!("Course path has no file name"))?
            .to_string_lossy()
            .to_string();
        Store::_get_semester(&name, &store.entry_point)
    }
}
