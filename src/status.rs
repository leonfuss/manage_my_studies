use crate::config::Config;
use anyhow::Result;

pub fn handle_status(config: &Config) -> Result<()> {
    match (config.active_semester(), config.active_course()) {
        (Some(active_semester), Some(active_course)) => {
            println!(
                "On course: {}/{}",
                active_semester.file_name(),
                active_course.name()
            );
        }
        (Some(active_semseter), None) => {
            println!("On semester: {}", active_semseter);
        }
        _ => {
            println!("No active semester or course set");
        }
    }
    Ok(())
}
