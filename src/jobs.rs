use std::process::Child;
use std::io::{Write, stdout};

use crate::Buffers;

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

impl JobState {
    fn as_str(&self) -> &str {
        match self {
            JobState::Done => "Done",
            JobState::Running => "Running",
        }
    }
}

impl Job {
    fn orig(& self) -> &str {
        match self.state {
            JobState::Done => &self.origin[..self.origin.len()-1],
            JobState::Running => &self.origin,
        }
    }
}

fn job_marker<'a>(i: usize, len: usize) -> &'a str {
    match i {
        x if x == len-1 => "+",
        x if x == len-2 => "-",
        _ => " ",
    }
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
            writeln!(
                stdout(),
                "[{}]{}  {:<24}{}",
                job.id,
                job_marker(i, len),
                "Done",
                job.orig()
            ).unwrap();
        }
    }
    jobs.retain(|job| job.state != JobState::Done);
}

pub fn jobs_cmd(jobs: &mut Vec<Job>, buffers: &mut Buffers) {
    if jobs.is_empty() {
        return;
    }

    let len = jobs.len();
    for (i, job) in jobs.iter_mut().enumerate() {
        match job.child.try_wait() {
            Ok(Some(_)) => job.state = JobState::Done,
            _ => (),
        };
        writeln!(
            buffers.out(),
            "[{}]{}  {:<24}{}",
            job.id,
            job_marker(i, len),
            job.state.as_str(),
            job.orig()
        ).unwrap();
    }
    jobs.retain(|job| job.state != JobState::Done);
}

pub fn add_job(jobs: &mut Vec<Job>, child: Child, name: String, origin: String) {
        let pid = child.id();
        let id = jobs
            .iter()
            .max_by_key(|job| job.id)
            .map_or(0, |job| job.id) + 1;
        println!("[{id}] {pid}");
        jobs.push(Job {
            id: id,
            name,
            origin,
            pid: pid,
            state: JobState::Running,
            child,
        });
}
