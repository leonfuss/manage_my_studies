use std::collections::HashMap;

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
    println!("== Performance == ");
    println!("  Overall Average:          {:.2}", average(store));
    println!("  Overall Weighted Average: {:.2}", weighted_average(store));
    println!("  By Degree: ");
    let weighted_averages = weighted_average_by_degree(store);
    if weighted_averages.is_empty() {
        println!("    No courses found");
    } else {
        for (degree, average) in weighted_average_by_degree(store) {
            println!("    {}: {:.2}", degree, average);
        }
    }

    Ok(())
}

// Unweighted average accross all degrees and course types (übK included)
// Only coures with a defined grade are considered.
pub fn average(store: &Store) -> f32 {
    let (sum, count) = store
        .semesters()
        .flat_map(|semester| semester.courses())
        .filter_map(|course| course.grade())
        .fold((0f32, 0), |(sum, count), grade| (sum + grade, count + 1));
    let average = if count > 0 { sum / (count as f32) } else { 0.0 };
    average
}

// Weighted average accross all degrees and course types (übK included)
// Only coures with a defined grade and ects are considered.
pub fn weighted_average(store: &Store) -> f32 {
    let (sum, count) = store
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
pub fn weighted_average_by_degree(store: &Store) -> HashMap<String, f32> {
    let mut degrees: HashMap<String, Vec<(Option<f32>, Option<u8>)>> = HashMap::new();
    store
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
