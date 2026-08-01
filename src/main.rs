use std::{io::{self, Write}, process::exit};
use std::path::PathBuf;
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

fn main() {
    loop {
        prompt();
        let command = get_command();
        let (command, args) = parse_command(&command);
        eval(&command, &args);
        // println!("{}, {:?}", &command, &args);
    }
}

fn prompt() {
    print!("$ ");
    io::stdout().flush().unwrap();
}

fn get_command() -> String {
    let mut command = String::new();
    match io::stdin().read_line(&mut command) {
        Ok(_) => command,
        Err(_) => String::new(),
    }
}


enum ParseMode {
    Normal,
    SingleQuote,
    DoubleQuote,
}


fn parse_command(command: &str) -> (Cmd, Vec<String>) {
    let mut args: Vec<String> = Vec::new();
    let mut last = ' ';
    let mut arg = String::with_capacity(command.len());
    let mut chars = command.trim().chars();
    let mut mode = ParseMode::Normal;

    while let Some(c) = chars.next() {
        match mode {
            ParseMode::Normal => match c {
                ' ' => match last {
                    ' ' => (),
                    _ => {
                        args.push(arg);
                        arg = String::with_capacity(command.len());
                    }
                },
                '\'' => mode = ParseMode::SingleQuote,
                '"' => mode = ParseMode::DoubleQuote,
                '\\' => if let Some(next) = chars.next() {
                    arg.push(next);
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

fn eval(command: &Cmd, args: &Vec<String>) {
    match command {
        Cmd::Builtin(cmd) => run_builtin(cmd, args),
        Cmd::External(cmd) => run_external(cmd, args),
        Cmd::Unknown(name) => println!("{}: command not found", name),
    }
}

fn run_builtin(cmd: &Builtin, args: &Vec<String>) {
    match cmd {
            Builtin::Exit => exit(0),
            Builtin::Echo => {
                let msg = args.join(" ");
                println!("{}", msg);
            },
            Builtin::Type => match args.is_empty() {
                true => println!(""),
                false => match Cmd::resolve(&args[0]) {
                    Cmd::Builtin(_) => println!("{} is a shell builtin", &args[0]),
                    Cmd::External(cmd) => println!("{} is {}", &cmd.name, &cmd.path.display()),
                    Cmd::Unknown(cmd) => println!("{} not found", &cmd),
                },
            },
    }
}


fn run_external(cmd: &Executable, args: &Vec<String>) {
    let mut cmd = Command::new(cmd.name.to_string());
    if !args.is_empty() {
        cmd.args(args);
    }
    let _ = cmd.status();

}

enum Cmd {
    Builtin(Builtin),
    External(Executable),
    Unknown(String),
}

enum Builtin {
    Exit,
    Echo,
    Type,
}

struct Executable {
    name: String,
    path: PathBuf,
}

impl Cmd {
    fn resolve(command: &str) -> Self {
        match command {
            "exit" => Cmd::Builtin(Builtin::Exit),
            "echo" => Cmd::Builtin(Builtin::Echo),
            "type" => Cmd::Builtin(Builtin::Type),
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

