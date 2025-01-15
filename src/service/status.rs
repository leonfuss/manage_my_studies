use crate::StoreProvider;
use anyhow::Result;
use std::collections::HashMap;

pub(super) struct StatusService<'s, Store>
where
    Store: StoreProvider,
{
    store: &'s Store,
}

impl<'s, Store> StatusService<'s, Store>
where
    Store: StoreProvider,
{
    pub fn new(store: &'s Store) -> StatusService<'s, Store> {
        StatusService { store }
    }

    pub fn run(&self) -> Result<()> {
        self.status()
    }

    fn status(&self) -> Result<()> {
        match self.store.current_semester() {
            Some(semester) => match semester.active_course() {
                Some(course) => println!(
                    "Active on course: {}/{}\n{:?}\n{:?}",
                    semester.name(),
                    course.name(),
                    semester,
                    course
                ),
                None => println!("Active on: {}/", semester.name()),
            },
            None => println!("No active semester or course"),
        }
        println!("== Performance == ");
        println!("  Overall Average:          {:.2}", self.average());
        println!("  Overall Weighted Average: {:.2}", self.weighted_average());
        println!("  By Degree: ");
        let weighted_averages = self.weighted_average_by_degree();
        if weighted_averages.is_empty() {
            println!("    No courses found");
        } else {
            for (degree, average) in weighted_averages {
                println!("    {}: {:.2}", degree, average);
            }
        }

        Ok(())
    }

    // Unweighted average accross all degrees and course types (übK included) // Only coures with a defined grade are considered.
    pub fn average(&self) -> f32 {
        let (sum, count) = self
            .store
            .semesters()
            .flat_map(|semester| semester.courses())
            .filter_map(|course| course.grade())
            .fold((0f32, 0), |(sum, count), grade| (sum + grade, count + 1));
        let average = if count > 0 { sum / (count as f32) } else { 0.0 };
        average
    }

    // Weighted average accross all degrees and course types (übK included)
    // Only coures with a defined grade and ects are considered.
    pub fn weighted_average(&self) -> f32 {
        let (sum, count) = self
            .store
            .semesters()
            .flat_map(|semester| semester.courses())
            .filter_map(|course| course.grade().zip(course.ects()))
            .fold((0f32, 0), |(sum, count), (grade, ects)| {
                (sum + grade * (ects as f32), count + ects)
            });
        let average = if count > 0 { sum / (count as f32) } else { 0.0 };
        average
    }

    // Calculates the weighted average by degree. This does not include coures marked with üBK
    pub fn weighted_average_by_degree(&self) -> HashMap<String, f32> {
        let mut degrees: HashMap<String, Vec<(Option<f32>, Option<u8>)>> = HashMap::new();
        self.store
            .semesters()
            .flat_map(|semester| semester.courses())
            .for_each(|course| {
                for d in course.degrees() {
                    if course.uebk().unwrap_or(false) {
                        continue;
                    }
                    degrees
                        .entry(d.to_string())
                        .or_insert(vec![])
                        .push((course.grade(), course.ects()));
                }
            });

        let weighted_averages: HashMap<String, f32> = degrees
            .into_iter()
            .map(|(degree, courses)| {
                let (sum, count) = courses
                    .iter()
                    .filter_map(|course| course.0.zip(course.1))
                    // Calculates the weighted average by degree. This does not include coures marked with üBK
                    .fold((0f32, 0), |(sum, count), (grade, ects)| {
                        (sum + grade * (ects as f32), count + ects)
                    });
                let average = if count > 0 { sum / (count as f32) } else { 0.0 };
                (degree, average)
            })
            .collect();
        weighted_averages
    }
}
