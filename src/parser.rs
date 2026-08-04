use std::path::PathBuf;
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;


enum ParseMode {
    Normal,
    SingleQuote,
    DoubleQuote,
}


pub fn parse_command(command: &str) -> (Cmd, Vec<String>) {
    let home = std::env::home_dir()
        .map(|path| path.to_string_lossy().into_owned())
        .unwrap_or_default();

    let mut args: Vec<String> = Vec::new();
    let mut last = ' ';
    let mut arg = String::with_capacity(command.len());
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
                        arg = String::with_capacity(command.len());
                    }
                    arg.push(c);
                    if let Some(&next) = chars.peek() {
                        if next == '>' {
                            chars.next();
                            arg.push(c);
                        }
                    }
                    args.push(arg);
                    arg = String::with_capacity(command.len());
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
    if args.is_empty() {
       return (Cmd::Unknown(String::new()), args)
    }

    let command = args.remove(0);
    let command = Cmd::resolve(&command);
    (command, args)
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
