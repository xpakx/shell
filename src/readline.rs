use core::cell::RefCell;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use std::rc::Rc;

use rustyline::completion::Completer;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper, Result};


pub struct CommandHelper {
    commands: RefCell<Vec<String>>,
    path: RefCell<String>,
    completions: Rc<RefCell<HashMap<String, String>>>,
}

impl CommandHelper {
    pub fn new(completions: Rc<RefCell<HashMap<String, String>>>) -> Self {
        CommandHelper { 
            commands: RefCell::new(Vec::new()),
            path: RefCell::new(String::new()),
            completions,
        }
    }

    pub fn update_commands(&self) {
        let path = env::var("PATH").unwrap();
        {
            let mut old_path = self.path.borrow_mut();
            if path == *old_path {
                return
            }
            *old_path = path.clone(); 
        }
        let commands = prepare_commands(&path);

        let mut old_cmds = self.commands.borrow_mut();
        *old_cmds = commands;
    }

    pub fn complete_command(&self, prefix: &str) -> Vec<String> {
            self.update_commands();
            let mut matches = vec![];
            let commands = self.commands.borrow();

            // TODO: less naive approach; a Trie?
            for cmd in &*commands {
                if cmd.starts_with(prefix) {
                    matches.push(format!("{} ", cmd));
                }
            }
            matches
    }

    pub fn complete_file(&self, path: &str, prefix: &str) -> Vec<String> {
            let mut matches = vec![];

            let Ok(cwd) = env::current_dir() else {
                return matches
            };

            let path = if path.is_empty() {
                cwd
            } else {
                let raw_path = Path::new(path);
                if raw_path.is_absolute() {
                    raw_path.to_path_buf()
                } else {
                    cwd.join(raw_path)
                }
            };


            match fs::read_dir(path) {
                Ok(entries) => {
                    for entry in entries {
                        match entry {
                            Ok(entry) => {
                                let file_name = entry.file_name().to_string_lossy().to_string();
                                if file_name.starts_with(prefix) {
                                    if entry.path().is_file() {
                                        matches.push(format!("{} ", entry.file_name().display()));
                                    } else if  entry.path().is_dir() {
                                        matches.push(format!("{}/", entry.file_name().display()));
                                    }
                                }
                            },
                            Err(_) => (),
                        }
                    }
                },
                Err(_) => (),
            }
            matches.sort();
            matches
    }


    pub fn complete_completer(
        &self,
        path: &str,
        line: &str,
        cmd_end: usize,
        cursor: usize,
    ) -> Vec<String> {
        let cmd_name = &line[0..cmd_end];
        let curr_start = match (&line[..cursor]).rfind(' ') {
            Option::Some(i) => i+1,
            Option::None => 0,
        };
        let curr_end = match (&line[cursor..]).find(' ') {
            Option::Some(i) => i+cursor,
            Option::None => cursor,
        };
        let curr = &line[curr_start..curr_end];
        let last_end = match curr_start {
            0 => 0,
            i => i-1,
        };
        let mut last_start = match (&line[..last_end]).rfind(' ') {
            Option::Some(i) => i+1,
            Option::None => 0,
        };
        if last_start <= cmd_end {
            last_start = last_end;
        }
        let last = &line[last_start..last_end];

        let mut cmd = Command::new(path);
        cmd.args([cmd_name, curr, last]);
        match cmd.output() {
            Ok(output) => String::from_utf8_lossy(&output.stdout)
                .split_whitespace()
                .map(|s| format!("{} ", s))
                .collect(),
            Err(_) => Vec::new(),
        }
    }
}


fn prepare_commands(path: &str) -> Vec<String> {
    let mut commands: Vec<String> = vec!["echo", "exit", "type", "pwd", "cd", "complete"]
        .into_iter()
        .map(String::from)
        .collect();
    let execs = executables(path); 
    commands.extend(execs);
    commands.sort();
    commands.dedup();
    commands
}


fn executables(path: &str) -> Vec<String> {
    let mut paths = env::split_paths(&path);
    let mut executables = Vec::new();
    while let Some(path) = paths.next() {
        match fs::read_dir(path) {
            Ok(entries) => {
                for entry in entries {
                    match entry {
                        Ok(entry) => {
                            if entry.path().is_file() {
                                if let Ok(metadata) = fs::metadata(&entry.path()) {
                                    if metadata.permissions().mode() & 0o111 != 0 {
                                        executables.push(entry.file_name().to_string_lossy().to_string());
                                    }
                                }
                            }
                        },
                        Err(_) => (),
                    }
                }
            },
            Err(_) => (),
        }
    }
    executables
}

impl Helper for CommandHelper {}

impl Completer for CommandHelper {
    type Candidate = String;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<String>)> {
        // TODO: use proper parsing instead of spaces
        let mut matches = vec![];
        let prefix = &line[..pos];  // TODO: to next space?

        let mut start = match prefix.rfind(' ') {
            Option::Some(i) => if i >= pos {i} else {i+1},
            Option::None => 0,
        };
        let prefix = &prefix[start..pos];

        // TODO: spaces at the beginning
        matches = if start > 0 {
            let completer = match line.find(' ') {
                Option::Some(i) => {
                    let path = self.completions.borrow().get(&line[0..i]).cloned();
                    match path {
                        Option::Some(path) => Some((path, i)),
                        Option::None => None,
                    }
                },
                Option::None => None,
            };

            match completer {
                Option::Some((path, idx)) => {
                    self.complete_completer(&path, &line, idx, pos)
                },
                Option::None => {
                    let start_curr = match prefix.rfind('/') {
                        Option::Some(i) => if i >= prefix.len() {Some(i)} else {Some(i+1)},
                        Option::None => None,
                    };
                    let (path, prefix, new_start) = match start_curr {
                        Option::Some(i) => (&prefix[..i], &prefix[i..], start+i),
                        Option::None => ("", prefix, start),

                    };
                    start = new_start;
                    self.complete_file(&path, &prefix)
                }

            }
        } else {
            self.complete_command(&prefix)
        };

        Ok((start, matches))
    }
}

impl Hinter for CommandHelper {
    type Hint = String;
    fn hint(&self, _line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<String> {
        None
    }
}

impl Highlighter for CommandHelper {}
impl Validator for CommandHelper {}
