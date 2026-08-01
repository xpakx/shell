use std::{io::{self, Write}, process::exit};
use std::path::PathBuf;

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
        Cmd::External(_) => (),
        Cmd::Unknown(name) => println!("{}: command not found", name),
    }
}

fn run_builtin(cmd: &Builtin, args: &Vec<String>) {
    match cmd {
            Builtin::Exit => exit(0),
            Builtin::Echo => {
                let msg = args.join(" ");
                println!("{}", msg);
            }
    }
}

enum Cmd {
    Builtin(Builtin),
    External(Executable),
    Unknown(String),
}

enum Builtin {
    Exit,
    Echo,
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
            _ => Cmd::Unknown(command.to_string()),
        }
    }
}
