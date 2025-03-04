use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{ConfigProvider, StoreProvider};

use super::{
    config::SemesterNames,
    course::Course,
    paths::{EntryPoint, MaybeSymLinkable, ReadWriteDO, SemesterPath, StoreDataFile},
    semester::Semester,
};

#[derive(Debug)]
pub(crate) struct Store {
    active_semester: Option<SemesterPath>,
    entry_point: EntryPoint,
    semester_names: SemesterNames,
    current_semester_link: MaybeSymLinkable,
    current_course_link: MaybeSymLinkable,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct StoreDO {
    active_semester: Option<String>,
}

impl Store {
    pub fn new<Config>(config: Config) -> Result<Store>
    where
        Config: ConfigProvider,
    {
        let entry_point = config.entry_point();
        let semester_names = config.semester_names();
        let current_semester_link = config.current_semester_link();
        let current_course_link = config.current_course_link();

        let file = entry_point.data_file()?;
        let store_do = file.read()?;

        let active_semester = store_do
            .active_semester
            .map(|name| entry_point.semester_path(&name, &semester_names))
            .flatten();

        let store = Store {
            entry_point,
            semester_names,
            current_course_link,
            current_semester_link,
            active_semester,
        };
        Ok(store)
    }
}

impl StoreProvider for Store {
    fn semesters(&self) -> impl Iterator<Item = Semester> {
        self.entry_point
            .semester_paths(&self.semester_names)
            .filter_map(|path| Semester::from_path(path, &self.semester_names).ok())
    }

    fn courses(&self) -> impl Iterator<Item = Course> {
        self.entry_point
            .semester_paths(&self.semester_names)
            .flat_map(|path| path.course_paths())
            .filter_map(|path| Course::from_path(path).ok())
    }

    fn semester_courses(&self, semester: Semester) -> impl Iterator<Item = Course> {
        semester.courses()
    }

    fn get_semester(&self, name: &str) -> Option<Semester> {
        self.entry_point
            .semester_path(name, &self.semester_names)
            .map(|path| Semester::from_path(path, &self.semester_names).ok())
            .flatten()
    }

    fn current_semester(&self) -> Option<Semester> {
        self.active_semester
            .as_ref()
            .map(|it| Semester::from_path(it.clone(), &self.semester_names).ok())
            .flatten()
    }

    fn current_course(&self) -> Option<Course> {
        self.current_semester()
            .map(|semester| semester.active_course())
            .flatten()
    }

    fn set_current_semester(&mut self, semester: Option<&Semester>) -> Result<()> {
        self.active_semester = semester.as_ref().map(|it| it.path().clone());
        let store_do = StoreDO {
            active_semester: semester.map(|it| it.path().name().to_string()),
        };
        self.entry_point.data_file()?.write(&store_do)?;
        if let Some(semester) = self.active_semester.as_ref() {
            self.current_semester_link.link_from(semester.path())?;
        } else {
            self.current_semester_link.remove_link()?;
            self.current_course_link.remove_link()?;
        }
        Ok(())
    }

    fn set_current_course(&self, semester: &mut Semester, course: Option<&Course>) -> Result<()> {
        semester.set_active(course)?;
        if let Some(course) = course.as_ref() {
            self.current_course_link.link_from(course.path().as_path())
        } else {
            self.current_course_link.remove_link()
        }
    }

    fn entry_point(&self) -> EntryPoint {
        self.entry_point.clone()
    }
}

impl ReadWriteDO for StoreDataFile {
    type Object = StoreDO;
}
