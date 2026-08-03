use core::writeln;
use std::{io::{self, Write, BufWriter}, process::exit};
use std::path::{PathBuf, Path};
use std::env;
use std::fs::{self, File};
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

fn main() {
    loop {
        prompt();
        let command = get_command();
        let (command, mut args) = parse_command(&command);
        let mut buffers = get_buffers(&mut args);
        eval(&command, &args, &mut buffers);
        // println!("{:?}", &args);
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

fn eval(command: &Cmd, args: &Vec<String>, buffers: &mut Buffers) {
    match command {
        Cmd::Builtin(cmd) => run_builtin(cmd, args, buffers),
        Cmd::External(cmd) => run_external(cmd, args, buffers),
        Cmd::Unknown(name) => println!("{}: command not found", name),
    }
}

fn run_builtin(cmd: &Builtin, args: &Vec<String>, buffers: &mut Buffers) {
    match cmd {
            Builtin::Exit => exit(0),
            Builtin::Echo => {
                let msg = args.join(" ");
                writeln!(buffers.out, "{}", msg).unwrap();
            },
            Builtin::Type => match args.is_empty() {
                true => writeln!(buffers.out, "").unwrap(),
                false => match Cmd::resolve(&args[0]) {
                    Cmd::Builtin(_) => writeln!(buffers.out, "{} is a shell builtin", &args[0]).unwrap(),
                    Cmd::External(cmd) => writeln!(buffers.out, "{} is {}", &cmd.name, &cmd.path.display()).unwrap(),
                    Cmd::Unknown(cmd) => writeln!(buffers.out, "{} not found", &cmd).unwrap(),
                },
            },
            Builtin::Pwd => match env::current_dir() {
                Ok(cwd) => writeln!(buffers.out, "{}", cwd.to_str().unwrap()).unwrap(),
                _ => writeln!(buffers.err, "should not happen").unwrap(),
            },
            Builtin::Cd => {
                if !args.is_empty() {
                    let path = Path::new(&args[0]);
                    if !path.is_dir() || !env::set_current_dir(path).is_ok() {
                        writeln!(buffers.err, "cd: {}: No such file of directory", path.display()).unwrap();
                    }
                }
            },
    }
}


fn run_external(cmd: &Executable, args: &Vec<String>, buffers: &mut Buffers) {
    let mut cmd = Command::new(cmd.name.to_string());
    if !args.is_empty() {
        cmd.args(args);
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

enum Cmd {
    Builtin(Builtin),
    External(Executable),
    Unknown(String),
}

enum Builtin {
    Exit,
    Echo,
    Type,
    Pwd,
    Cd,
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
            "pwd" => Cmd::Builtin(Builtin::Pwd),
            "cd" => Cmd::Builtin(Builtin::Cd),
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


struct Buffers {
    out: BufWriter<Box<dyn Write>>,
    err: BufWriter<Box<dyn Write>>,
}

fn redirect_out(input: &mut Vec<String>) -> Option<String> {
    let index = input.iter().position(|x| x == ">" || x == "1>")?;
    if index + 1 < input.len() {
        input.remove(index);
        Some(input.remove(index))
    } else {
        None
    }
}

fn redirect_err(input: &mut Vec<String>) -> Option<String> {
    let index = input.iter().position(|x| x == "2>")?;
    if index + 1 < input.len() {
        input.remove(index);
        Some(input.remove(index))
    } else {
        None
    }
}

fn get_buffers(args: &mut Vec<String>) -> Buffers {
    let out: BufWriter<Box<dyn Write>>;
    if let Some(out_path) = redirect_out(args) {
        let file = File::create(&out_path)
            .unwrap_or_else(|err| panic!("cannot open {out_path}: {err}"));
        out = BufWriter::new(Box::new(file));
    } else {
        out = BufWriter::new(Box::new(io::stdout()));
    }

    let err: BufWriter<Box<dyn Write>>;
    if let Some(out_path) = redirect_err(args) {
        let file = File::create(&out_path)
            .unwrap_or_else(|err| panic!("cannot open {out_path}: {err}"));
        err = BufWriter::new(Box::new(file));
    } else {
        err = BufWriter::new(Box::new(io::stdout()));
    }

    Buffers {out, err}
}
