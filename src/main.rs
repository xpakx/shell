use core::{cell::RefCell, option::Option::{self, Some}, writeln};
use std::{collections::HashMap, fs::File, io::{self, Write, stdout}, process::{Stdio, exit}};
use std::path::Path;
use std::env;
use std::fs::OpenOptions;
use std::process::{Command, ChildStdout};
use rustyline::{self, history::{History, DefaultHistory}};
use std::rc::Rc;

mod readline;
use readline::CommandHelper;

mod parser;
use parser::{parse_command, split_commands, Cmd, Builtin, Executable, CommandLine};

mod jobs;
use jobs::{Job, reap_jobs, jobs_cmd, add_job};

fn main() {
    let rl_config = rustyline::Config::builder()
        .completion_type(rustyline::CompletionType::List)
        .build();
    let mut rl: rustyline::Editor<CommandHelper, DefaultHistory> = rustyline::Editor::with_config(rl_config).unwrap();
    let completions: Rc<RefCell<HashMap<String, String>>> =  Rc::new(RefCell::new(HashMap::new()));
    rl.set_helper(Some(CommandHelper::new(Rc::clone(&completions))));
    let mut jobs: Vec<Job> = Vec::new();

    if let Ok(var) = env::var("HISTFILE") {
        if !var.is_empty() {
            _ = rl.history_mut().load(Path::new(&var));
        }
    };

    loop {
        let command = get_command(&mut rl);
        rl.add_history_entry(command.trim()).unwrap();
        let cmd_line = parse_command(&command);
        let cmds = split_commands(cmd_line, &command);
        if cmds.is_empty() {
            writeln!(stdout(), "").unwrap();
        } else {
            let mut pipe = EvalResult::Empty;
            for mut cmd_line in cmds {
                let buffers = get_buffers(&mut cmd_line.tokens, pipe, cmd_line.has_next_in_pipeline);
                cmd_line.enable_bg();
                pipe = eval(&cmd_line, buffers, Rc::clone(&completions), &mut jobs, &mut rl.history_mut());
            }
        }
        reap_jobs(&mut jobs);
        // println!("{:?}", &args);
    };
}

fn get_command(rl: &mut rustyline::Editor<CommandHelper, DefaultHistory>) -> String {
    let readline = rl.readline("$ ");
    match readline {
        Ok(line) => line,
        Err(_) => String::new(),
    }
}

enum EvalResult {
    Empty,
    Child(ChildStdout),
    Bytes(Vec<u8>),
}

fn eval(
    command: &CommandLine,
    mut buffers: Buffers,
    completions: Rc<RefCell<HashMap<String, String>>>,
    jobs: &mut Vec<Job>,
    history: &mut DefaultHistory,
) -> EvalResult  {
    match &command.cmd {
        Cmd::Builtin(cmd) => {
            run_builtin(cmd, command, &mut buffers, completions, jobs, history);
            return match buffers.out_bytes {
                Option::None => EvalResult::Empty,
                Option::Some(bytes) => EvalResult::Bytes(bytes),
            }
        }
        Cmd::External(cmd) => {
            return match run_external(cmd, command, buffers, jobs) {
                Option::None => EvalResult::Empty,
                Option::Some(child) => EvalResult::Child(child),
            }
        },
        Cmd::Unknown(name) => println!("{}: command not found", name),
    };
    EvalResult::Empty
}

fn run_builtin(
    cmd: &Builtin,
    command: &CommandLine,
    buffers: &mut Buffers,
    completions: Rc<RefCell<HashMap<String, String>>>,
    jobs: &mut Vec<Job>,
    history: &mut DefaultHistory,
) {
    match cmd {
            Builtin::Exit => {
                // TODO: move that
                if let Ok(var) = env::var("HISTFILE") {
                    if !var.is_empty() {
                        _ = history.append(Path::new(&var));
                    }
                };
                exit(0);
            },
            Builtin::Echo => {
                let msg = command.tokens.join(" ");
                writeln!(buffers.out(), "{}", msg).unwrap();
            },
            Builtin::Type => match command.tokens.is_empty() {
                true => writeln!(buffers.out(), "").unwrap(),
                false => match Cmd::resolve(&command.tokens[0]) {
                    Cmd::Builtin(_) => writeln!(buffers.out(), "{} is a shell builtin", &command.tokens[0]).unwrap(),
                    Cmd::External(cmd) => writeln!(buffers.out(), "{} is {}", &cmd.name, &cmd.path.display()).unwrap(),
                    Cmd::Unknown(cmd) => writeln!(buffers.out(), "{} not found", &cmd).unwrap(),
                },
            },
            Builtin::Pwd => match env::current_dir() {
                Ok(cwd) => writeln!(buffers.out(), "{}", cwd.to_str().unwrap()).unwrap(),
                _ => writeln!(buffers.err(), "should not happen").unwrap(),
            },
            Builtin::Cd => {
                if !command.tokens.is_empty() {
                    let path = Path::new(&command.tokens[0]);
                    if !path.is_dir() || !env::set_current_dir(path).is_ok() {
                        writeln!(buffers.err(), "cd: {}: No such file or directory", path.display()).unwrap();
                    }
                }
            },
            Builtin::Complete => {
                let p = command.find_flag("-p");
                if let Some(command) = p {
                    match completions.borrow().get(command) {
                        Option::Some(path) => writeln!(buffers.out(), "complete -C '{}' {}", path, command).unwrap(),
                        Option::None => writeln!(buffers.err(), "complete: {}: no completion specification", command).unwrap(),
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
            Builtin::Jobs => jobs_cmd(jobs, buffers),
            Builtin::History => {
                if let Some(arg) = command.tokens.get(0) {
                    if let Ok(num) = arg.parse::<usize>() {
                        let len = history.len();
                        let skip_count = len.saturating_sub(num);
                        for (idx, entry) in history.iter().skip(skip_count).enumerate() {
                            println!("    {}  {}", idx + skip_count + 1, entry);
                        }
                        return
                    }
                }
                let r = command.find_flag("-r");
                if let Some(file) = r {
                    _ = history.load(Path::new(file));
                    return
                    
                }
                let w = command.find_flag("-w");
                if let Some(file) = w {
                    _ = history.save(Path::new(file));
                    return
                    
                }
                let a = command.find_flag("-a");
                if let Some(file) = a {
                    _ = history.append(Path::new(file));
                    return
                    
                }
                for (idx, entry) in history.iter().enumerate() {
                    println!("    {}  {}", idx + 1, entry);
                }
            },
            Builtin::Declare => {
                let p = command.find_flag("-p");
                if let Some(variable) = p {
                    match env::var(variable) {
                        Err(_) => writeln!(buffers.out(), "declare: {}: not found", variable).unwrap(),
                        Ok(res) => writeln!(buffers.out(), "declare -- {}=\"{}\"", variable, res).unwrap(),
                    }
                    if let Ok(var) = env::var(variable) {
                        if !var.is_empty() {
                        }
                    };
                    return
                }
                for arg in &command.tokens {
                    if let Some((var, val)) = arg.split_once("=") {
                        unsafe {
                            env::set_var(var, val);
                        }
                    }
                }
            },
    }
}


fn run_external(
    cmd_type: &Executable,
    command: &CommandLine,
    buffers: Buffers,
    jobs: &mut Vec<Job>,
) -> Option<ChildStdout> {
    let mut cmd = Command::new(cmd_type.name.to_string());
    if !command.tokens.is_empty() {
        cmd.args(&command.tokens);
    }

    let mut bytes_to_write = None;

    match buffers.in_buffer {
        BufferInput::Inherit => cmd.stdin(Stdio::inherit()),
        BufferInput::File(file) => cmd.stdin(Stdio::from(file)),
        BufferInput::Piped(pipe) => cmd.stdin(Stdio::from(pipe)),
        BufferInput::Bytes(bytes) => {
            bytes_to_write = Some(bytes);
            cmd.stdin(Stdio::piped())
        }
    };

    if command.run_in_bg { // TODO: for now we assume this could only end chain
        cmd.stdout(Stdio::inherit());
        cmd.stderr(Stdio::inherit());
        let child = cmd.spawn().unwrap();
        add_job(jobs, child, cmd_type.name.clone(), command.origin.clone());
        return None
    }

    let should_pipe = command.has_next_in_pipeline && buffers.out_file.is_none();
    if should_pipe {
        cmd.stdout(Stdio::piped());
    } else {
        match buffers.out_file {
            Option::Some(file) => cmd.stdout(Stdio::from(file)),
            Option::None => cmd.stdout(Stdio::inherit()),
        };
    }

    match buffers.err_file {
        Option::Some(file) => cmd.stderr(Stdio::from(file)),
        Option::None => cmd.stderr(Stdio::inherit()),
    };

    let mut child = cmd.spawn().unwrap();
    if let Some(bytes) = bytes_to_write {
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(&bytes).unwrap();
        }
    }

    let stdout_pipe = match should_pipe {
        true => child.stdout.take(),
        false => {
            child.wait().unwrap();
            None
        }
    };

    stdout_pipe
}

enum BufferInput {
    Inherit,
    File(std::fs::File),
    Piped(ChildStdout),
    Bytes(Vec<u8>),
}

struct Buffers {
    out_file: Option<File>,
    err_file: Option<File>,
    in_buffer: BufferInput,
    out_bytes: Option<Vec<u8>>,
}

impl Buffers {
    pub fn out(&mut self) -> Box<dyn Write + '_> {
        match &mut self.out_file {
            Option::Some(file) => Box::new(file),
            Option::None => match &mut self.out_bytes {
                Option::Some(bytes) => Box::new(bytes),
                Option::None => Box::new(io::stdout().lock()),
            }
        }
    }

    pub fn err(&mut self) -> Box<dyn Write + '_> {
        match &mut self.err_file {
            Option::Some(file) => Box::new(file),
            Option::None => Box::new(io::stderr().lock()),
        }
    }
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

fn redirect_in(input: &mut Vec<String>) -> Option<String> {
    let index = input.iter().position(|x| {
        matches!(x.as_str(), "<" | "0<")
    })?;

    if index + 1 < input.len() {
        input.remove(index);
        let path = input.remove(index);
        Some(path)
    } else {
        None
    }
}

fn get_buffers(args: &mut Vec<String>, pipe: EvalResult, has_next: bool) -> Buffers {
    let out: Option<File>;
    if let Some(redirect) = redirect_out(args) {
        let mut opts = OpenOptions::new();
        opts.create(true).write(true);
        match redirect.mode {
            RedirectMode::Overwrite => opts.truncate(true),
            RedirectMode::Append => opts.append(true),
        };
        let file = opts.open(&redirect.path)
            .unwrap_or_else(|err| panic!("cannot open {}: {err}", &redirect.path));
        out = Some(file);
    } else {
        out = None;
    }

    let err: Option<File>;
    if let Some((mode, out_path)) = redirect_err(args) {
        let mut opts = OpenOptions::new();
        opts.create(true).write(true);
        match mode {
            RedirectMode::Overwrite => opts.truncate(true),
            RedirectMode::Append => opts.append(true),
        };
        let file = opts.open(&out_path)
            .unwrap_or_else(|err| panic!("cannot open {out_path}: {err}"));
        err = Some(file);
    } else {
        err = None;
    }


    let in_buffer: BufferInput;
    if let Some(path) = redirect_in(args) {
        let file = File::open(&path)
            .unwrap_or_else(|err| panic!("cannot open {}: {err}", &path));
        in_buffer = BufferInput::File(file);
    } else {
        in_buffer = match pipe {
            EvalResult::Child(child) => BufferInput::Piped(child),
            EvalResult::Bytes(bytes) => BufferInput::Bytes(bytes),
            EvalResult::Empty => BufferInput::Inherit
        }
    }

    let out_bytes = if has_next & out.is_none() {
        Some(Vec::new())
    } else {
        None
    };

    Buffers {
        out_file: out,
        err_file: err,
        in_buffer,
        out_bytes,
    }
}
