use crate::store::Store;
use anyhow::Result;

pub fn status(store: &Store) -> Result<()> {
    match store.active_semester() {
        Some(semester) => match semester.active() {
            Some(course) => println!("Active on course: {}/{}", semester.name(), course.name()),
            None => println!("Active on: {}/", semester.name()),
        },
        None => println!("No active semester or course"),
    }
    Ok(())
}
