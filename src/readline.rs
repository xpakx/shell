use core::cell::RefCell;
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;

use rustyline::completion::Completer;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper, Result};


pub struct CommandHelper {
    commands: RefCell<Vec<String>>,
    path: RefCell<String>,
}

impl CommandHelper {
    pub fn new() -> Self {
        CommandHelper { 
            commands: RefCell::new(Vec::new()),
            path: RefCell::new(String::new()),
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
}


fn prepare_commands(path: &str) -> Vec<String> {
    let mut commands: Vec<String> = vec!["echo", "exit", "type", "pwd", "cd"]
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
        let mut matches = vec![];
        let prefix = &line[..pos];

        self.update_commands();
        let commands = self.commands.borrow();

        // TODO: less naive approach; a Trie?
        for cmd in &*commands {
            if cmd.starts_with(prefix) {
                matches.push(format!("{} ", cmd));
            }
        }

        Ok((0, matches))
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
