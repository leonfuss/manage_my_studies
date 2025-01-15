use anyhow::Result;

use crate::domain::{Course, EntryPoint, MaybeSymLinkable, Semester, SemesterNames};

pub(crate) trait StoreProvider: Sized {
    fn semesters(&self) -> impl Iterator<Item = Semester>;
    fn courses(&self) -> impl Iterator<Item = Course>;
    fn semester_courses(&self, semester: Semester) -> impl Iterator<Item = Course>;
    fn get_semester(&self, name: &str) -> Option<Semester>;
    fn current_semester(&self) -> Option<Semester>;
    fn current_course(&self) -> Option<Course>;
    fn set_current_semester(&mut self, semester: Option<&Semester>) -> Result<()>;
    fn set_current_course(&self, semester: &mut Semester, course: Option<&Course>) -> Result<()>;
    fn entry_point(&self) -> EntryPoint;
}

pub(crate) trait ConfigProvider {
    fn entry_point(&self) -> EntryPoint;
    fn current_course_link(&self) -> MaybeSymLinkable;
    fn current_semester_link(&self) -> MaybeSymLinkable;
    fn semester_names(&self) -> SemesterNames;
}
