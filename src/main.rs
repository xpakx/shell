use std::io::{self, Write};

fn main() {
    loop {
        prompt();
        let command = get_command();
        print!("{}", &command);
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
