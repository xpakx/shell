use std::process::Child;
use std::io::{Write, stdout};

#[derive(PartialEq)]
pub enum JobState {
    Running,
    Done,
}

pub struct Job {
    pub id: usize,
    #[allow(dead_code)]
    pub name: String,
    #[allow(dead_code)]
    pub pid: u32,
    pub origin: String,
    pub state: JobState,
    pub child: Child,
}


pub fn reap_jobs(jobs: &mut Vec<Job>) {
    if jobs.is_empty() {
        return;
    }
    let len = jobs.len();
    for (i, job) in jobs.iter_mut().enumerate() {
        match job.child.try_wait() {
            Ok(Some(_)) => job.state = JobState::Done,
            _ => (),
        };
        if job.state == JobState::Done {
            let marker = match i {
                x if x == len-1 => "+",
                x if x == len-2 => "-",
                _ => " ",
            };
            let origin = &job.origin[..job.origin.len()-1];
            writeln!(stdout(), "[{}]{}  {:<24}{}", job.id, marker, "Done", origin).unwrap();
        }
    }
    jobs.retain(|job| job.state != JobState::Done);
}
