use std::io::{self, Write};

fn main() {
    loop {
        prompt();
        let command = get_command();
        let (command, args) = parse_command(&command);
        println!("{}, {:?}", &command, &args);
    }
}

fn prompt() {
    print!("$ ");
    io::stdout().flush().unwrap();
}

fn get_command() -> String {
    let mut command = String::new();
    io::stdin().read_line(&mut command).unwrap();
    command
}

fn parse_command(command: &str) -> (String, Vec<String>) {
    let mut args: Vec<String> = Vec::new();
    let mut last = ' ';
    let mut arg = String::with_capacity(command.len());
    let mut chars = command.trim().chars();

    while let Some(c) = chars.next() {
        match c {
            ' ' => match last {
                ' ' => (),
                _ => {
                    args.push(arg);
                    arg = String::with_capacity(command.len());
                }
            },
            _ => arg.push(c),
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
