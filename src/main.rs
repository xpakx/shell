use core::{cell::RefCell, writeln};
use std::{collections::HashMap, io::{self, BufWriter, Write}, process::exit};
use std::path::Path;
use std::env;
use std::fs::OpenOptions;
use std::process::Command;
use rustyline::{self, history::DefaultHistory};
use std::rc::Rc;

mod readline;
use readline::CommandHelper;

mod parser;
use parser::{parse_command, Cmd, Builtin, Executable, CommandLine};

fn main() {
    let rl_config = rustyline::Config::builder()
        .completion_type(rustyline::CompletionType::List)
        .build();
    let mut rl: rustyline::Editor<CommandHelper, DefaultHistory> = rustyline::Editor::with_config(rl_config).unwrap();
    let completions: Rc<RefCell<HashMap<String, String>>> =  Rc::new(RefCell::new(HashMap::new()));
    rl.set_helper(Some(CommandHelper::new(Rc::clone(&completions))));

    loop {
        let command = get_command(&mut rl);
        let mut cmd_line = parse_command(&command);
        let mut buffers = get_buffers(&mut cmd_line.tokens);
        eval(&cmd_line, &mut buffers, Rc::clone(&completions));
        // println!("{:?}", &args);
    }
}

fn get_command(rl: &mut rustyline::Editor<CommandHelper, DefaultHistory>) -> String {
    let readline = rl.readline("$ ");
    match readline {
        Ok(line) => line,
        Err(_) => String::new(),
    }
}

fn eval(
    command: &CommandLine,
    buffers: &mut Buffers,
    completions: Rc<RefCell<HashMap<String, String>>>
) {
    match &command.cmd {
        Cmd::Builtin(cmd) => run_builtin(cmd, command, buffers, completions),
        Cmd::External(cmd) => run_external(cmd, command, buffers),
        Cmd::Unknown(name) => println!("{}: command not found", name),
    }
}

fn run_builtin(
    cmd: &Builtin,
    command: &CommandLine,
    buffers: &mut Buffers,
    completions: Rc<RefCell<HashMap<String, String>>>
) {
    match cmd {
            Builtin::Exit => exit(0),
            Builtin::Echo => {
                let msg = command.tokens.join(" ");
                writeln!(buffers.out, "{}", msg).unwrap();
            },
            Builtin::Type => match command.tokens.is_empty() {
                true => writeln!(buffers.out, "").unwrap(),
                false => match Cmd::resolve(&command.tokens[0]) {
                    Cmd::Builtin(_) => writeln!(buffers.out, "{} is a shell builtin", &command.tokens[0]).unwrap(),
                    Cmd::External(cmd) => writeln!(buffers.out, "{} is {}", &cmd.name, &cmd.path.display()).unwrap(),
                    Cmd::Unknown(cmd) => writeln!(buffers.out, "{} not found", &cmd).unwrap(),
                },
            },
            Builtin::Pwd => match env::current_dir() {
                Ok(cwd) => writeln!(buffers.out, "{}", cwd.to_str().unwrap()).unwrap(),
                _ => writeln!(buffers.err, "should not happen").unwrap(),
            },
            Builtin::Cd => {
                if !command.tokens.is_empty() {
                    let path = Path::new(&command.tokens[0]);
                    if !path.is_dir() || !env::set_current_dir(path).is_ok() {
                        writeln!(buffers.err, "cd: {}: No such file or directory", path.display()).unwrap();
                    }
                }
            },
            Builtin::Complete => {
                let p = command.find_flag("-p");
                if let Some(command) = p {
                    match completions.borrow().get(command) {
                        Option::Some(path) => writeln!(buffers.err, "complete -C '{}' {}", path, command).unwrap(),
                        Option::None => writeln!(buffers.err, "complete: {}: no completion specification", command).unwrap(),
                    };
                }
                let c = command.find_flag_double("-C");
                if let Some((path, command)) = c {
                    let mut map = completions.borrow_mut();
                    map.insert(command.clone(), path.clone());
                };
                let r = command.find_flag("-r");
                if let Some(command) = r {
                    completions.borrow_mut().remove(command);
                }
            },
            Builtin::Jobs => (),
    }
}


fn run_external(cmd: &Executable, command: &CommandLine, buffers: &mut Buffers) {
    let mut cmd = Command::new(cmd.name.to_string());
    if !command.tokens.is_empty() {
        cmd.args(&command.tokens);
    }
    match cmd.output() {
        Ok(output) => {
            // TODO: interleaving
            let _ = buffers.out.write_all(&output.stdout);
            let _ = buffers.out.flush();
            let _ = buffers.err.write_all(&output.stderr);
            let _ = buffers.err.flush();
        },
        Err(_) => (),
    }
}


struct Buffers {
    out: BufWriter<Box<dyn Write>>,
    err: BufWriter<Box<dyn Write>>,
}


enum RedirectMode {
    Append,
    Overwrite,
}

struct Redirect {
    mode: RedirectMode,
    path: String,
}



fn redirect_out(input: &mut Vec<String>) -> Option<Redirect> {
    let index = input.iter().position(|x| {
        matches!(x.as_str(), ">" | ">>" | "1>" | "1>>")
    })?;

    if index + 1 < input.len() {
        let op = input[index].clone();
        input.remove(index);
        let path = input.remove(index);
        let mode = if op.contains(">>") {
            RedirectMode::Append
        } else {
            RedirectMode::Overwrite
        };
        Some(Redirect{mode, path})
    } else {
        None
    }
}

fn redirect_err(input: &mut Vec<String>) -> Option<(RedirectMode, String)> {
    let index = input.iter().position(|x| {
        matches!(x.as_str(), "2>" | "2>>")
    })?;

    if index + 1 < input.len() {
        let op = input[index].clone();
        input.remove(index);
        let path = input.remove(index);
        let mode = if op.contains(">>") {
            RedirectMode::Append
        } else {
            RedirectMode::Overwrite
        };
        Some((mode, path))
    } else {
        None
    }
}

fn get_buffers(args: &mut Vec<String>) -> Buffers {
    let out: BufWriter<Box<dyn Write>>;
    if let Some(redirect) = redirect_out(args) {
        let mut opts = OpenOptions::new();
        opts.create(true).write(true);
        match redirect.mode {
            RedirectMode::Overwrite => opts.truncate(true),
            RedirectMode::Append => opts.append(true),
        };
        let file = opts.open(&redirect.path)
            .unwrap_or_else(|err| panic!("cannot open {}: {err}", &redirect.path));
        out = BufWriter::new(Box::new(file));
    } else {
        out = BufWriter::new(Box::new(io::stdout()));
    }

    let err: BufWriter<Box<dyn Write>>;
    if let Some((mode, out_path)) = redirect_err(args) {
        let mut opts = OpenOptions::new();
        opts.create(true).write(true);
        match mode {
            RedirectMode::Overwrite => opts.truncate(true),
            RedirectMode::Append => opts.append(true),
        };
        let file = opts.open(&out_path)
            .unwrap_or_else(|err| panic!("cannot open {out_path}: {err}"));
        err = BufWriter::new(Box::new(file));
    } else {
        err = BufWriter::new(Box::new(io::stderr()));
    }

    Buffers {out, err}
}
