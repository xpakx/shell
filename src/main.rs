use std::{io::{self, Write}, process::exit};

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


fn parse_command(command: &str) -> (String, Vec<String>) {
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
                _ => arg.push(c),
            },
            ParseMode::SingleQuote => match c {
                '\'' => mode = ParseMode::Normal,
                _ => arg.push(c),
            }
            ParseMode::DoubleQuote => match c {
                '"' => mode = ParseMode::Normal,
                _ => arg.push(c),
            }
        }
        last = c;
    }
    if arg != "" {
        args.push(arg);
    }
    if args.is_empty() {
       return (String::new(), args)
    }

    let command = args.remove(0);
    (command, args)
}

fn eval(command: &str, args: &Vec<String>) {
    match command {
        "exit" => exit(0),
        "echo" => {
            let msg = args.join(" ");
            println!("{}", msg);
        }
        _ => println!("{}: command not found", command),
    }
}
