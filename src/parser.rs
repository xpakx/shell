use core::option::Option;
use std::path::PathBuf;
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;


enum ParseMode {
    Normal,
    SingleQuote,
    DoubleQuote,
}

pub struct CommandLine {
    pub cmd: Cmd,
    pub tokens: Vec<String>,
    pub origin: String,
    pub run_in_bg: bool,
    pub has_next_in_pipeline: bool,
}

impl CommandLine {
    pub fn new(mut tokens: Vec<String>, origin: &str, has_next: bool) -> CommandLine {

        if tokens.is_empty() {
            return CommandLine{
                cmd: Cmd::Unknown(String::new()),
                tokens,
                run_in_bg: false,
                origin: String::from(origin),
                has_next_in_pipeline: has_next,
            }
        }

        let command = tokens.remove(0);
        let command = Cmd::resolve(&command);
        CommandLine {
            cmd: command,
            tokens,
            run_in_bg: false,
            origin: String::from(origin),
            has_next_in_pipeline: has_next,
        }
    }

    pub fn find_flag(&self, flag: &str) -> Option<&String> {
        self.tokens.iter()
            .position(|x| x.as_str() == flag)
            .and_then(|index| self.tokens.get(index + 1))
    }

    pub fn find_flag_double(&self, flag: &str) -> Option<(& String, &String)> {
        let index = self.tokens.iter().position(|x| x.as_str() == flag)?;
        Some((self.tokens.get(index + 1)?, self.tokens.get(index + 2)?))
    }

    #[allow(dead_code)]
    pub fn find_bool_flag(&self, flag: &str) -> bool {
        self.tokens.iter() .position(|x| x.as_str() == flag).is_some()
    }

    pub fn enable_bg(&mut self) {
        self.run_in_bg = match self.tokens.last() {
            Option::Some(x) if x == "&" => {
                self.tokens.pop();
                true
            },
            _ => false,
        }
    }
}

pub fn split_commands(tokens: Vec<String>, command: &str) -> Vec<CommandLine> {
    let mut cmds: Vec<CommandLine> = Vec::new();
    let mut origs = command.split('|');
    let mut curr: Vec<String> = Vec::new();
    for token in tokens.iter() {
        if token == "|" {
            if !curr.is_empty() {
                let orig = origs.next().unwrap_or("");
                let cmd = CommandLine::new(curr, orig, true);
                cmds.push(cmd);
                curr = Vec::new();
            }
        } else {
            curr.push(token.clone());
        }
    }
    if !curr.is_empty() {
        let orig = origs.next().unwrap_or("");
        let cmd = CommandLine::new(curr, orig, false);
        cmds.push(cmd);
    }
    cmds
}


pub fn parse_command(command: &str) -> Vec<String> {
    let home = std::env::home_dir()
        .map(|path| path.to_string_lossy().into_owned())
        .unwrap_or_default();

    let mut args: Vec<String> = Vec::new();
    let mut last = ' ';
    let mut arg = String::new();
    let mut chars = command.trim().chars().peekable();
    let mut mode = ParseMode::Normal;

    while let Some(c) = chars.next() {
        match mode {
            ParseMode::Normal => match c {
                ' ' => match last {
                    ' ' => (),
                    _ => {
                        if !arg.is_empty() {
                            args.push(arg);
                            arg = String::new();
                        }
                    }
                },
                '\'' => mode = ParseMode::SingleQuote,
                '"' => mode = ParseMode::DoubleQuote,
                '\\' => if let Some(next) = chars.next() {
                    arg.push(next);
                },
                '~' => arg.push_str(&home),
                '>' => {
                    let have_io_number =  arg == "1" || arg == "2";
                    if !have_io_number && !arg.is_empty() {
                        args.push(arg);
                        arg = String::new();
                    }
                    arg.push(c);
                    if let Some(&next) = chars.peek() {
                        if next == '>' {
                            chars.next();
                            arg.push(c);
                        }
                    }
                    args.push(arg);
                    arg = String::new();
                },
                '<' => {
                    let have_io_number =  arg == "0";
                    if !have_io_number && !arg.is_empty() {
                        args.push(arg);
                        arg = String::new();
                    }
                    arg.push(c);
                    if let Some(&next) = chars.peek() {
                        if next == '<' {
                            chars.next();
                            arg.push(c);
                        }
                    }
                    args.push(arg);
                    arg = String::new();
                },
                '|' => {
                    if !arg.is_empty() {
                        args.push(arg);
                        arg = String::new();
                    }
                    arg.push(c);
                    args.push(arg);
                    arg = String::new();
                },
                _ => arg.push(c),
            },
            ParseMode::SingleQuote => match c {
                '\'' => mode = ParseMode::Normal,
                _ => arg.push(c),
            }
            ParseMode::DoubleQuote => match c {
                '"' => mode = ParseMode::Normal,
                '\\' => if let Some(next) = chars.next() {
                    arg.push(next);
                },
                _ => arg.push(c),
            }
        }
        last = c;
    }
    if arg != "" {
        args.push(arg);
    }
    args
}


pub enum Cmd {
    Builtin(Builtin),
    External(Executable),
    Unknown(String),
}

pub enum Builtin {
    Exit,
    Echo,
    Type,
    Pwd,
    Cd,
    Complete,
    Jobs,
}

pub struct Executable {
    pub name: String,
    pub path: PathBuf,
}

impl Cmd {
    pub fn resolve(command: &str) -> Self {
        match command {
            "exit" => Cmd::Builtin(Builtin::Exit),
            "echo" => Cmd::Builtin(Builtin::Echo),
            "type" => Cmd::Builtin(Builtin::Type),
            "pwd" => Cmd::Builtin(Builtin::Pwd),
            "cd" => Cmd::Builtin(Builtin::Cd),
            "complete" => Cmd::Builtin(Builtin::Complete),
            "jobs" => Cmd::Builtin(Builtin::Jobs),
            _ => match cmd_from_path(command) {
                Option::None => Cmd::Unknown(command.to_string()),
                Some(data) => Cmd::External(data),
            }
        }
    }
}

fn cmd_from_path(command: &str) -> Option<Executable> {
    let path = env::var("PATH").unwrap();
    let mut paths = env::split_paths(&path);
    while let Some(path) = paths.next() {
        let full_path = path.join(&command);
        if full_path.is_file() {
            if let Ok(metadata) = fs::metadata(&full_path) {
                if metadata.permissions().mode() & 0o111 != 0 {
                    return Some(
                        Executable {
                            name: command.to_string(),
                            path: full_path,
                        }
                    )
                }
            }
        }
    }
    None
}
