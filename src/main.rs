use std::io::{self, Write, Result};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

enum Command {
    New,
    Get,
    Suggest
}

fn write(s: &mut StandardStream, msg: &str) -> io::Result<()> {
        writeln!(s, "{}", msg)
}

const PARSE_MSG:   &str = "expecting 2 arguments: {{command}} {{target}}";
const COMMAND_MSG: &str = "expecting {{command}} as first argument: new, get, suggest";

impl Command {
    fn from_string(s: &str) -> Option<Command> {
        match s {
            "new"     => return Some(Command::New),
            "get"     => return Some(Command::Get),
            "suggest" => return Some(Command::Suggest),
            _         => return None,
        }
    }
    fn execute(&self, t: &str) {
        match self {
            Command::New     => println!("new command, {t}"),
            Command::Get     => println!("get command, {t}"),
            Command::Suggest => println!("suggest command"),
        }
    }
}

fn main() {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green))).unwrap_or_default();

    let mut input= String::new();
    let (mut cmd_str, mut target_str) = (String::new(), String::new());
    let std_in = io::stdin();

    match std_in.read_line(&mut input) {
        Ok(_) => (),
        Err(err) => panic!("{err}")
    }
        
    let mut split = input.split_whitespace();
    if split.clone().count() != 2 {
        let _ = write(&mut stdout, PARSE_MSG);
        std::process::exit(2);
    }
    if let Some(cmd) = split.next() {
        cmd_str = cmd.to_string();
    }
    if let Some(trgt) = split.next() {
        target_str = trgt.to_string();
    }

    let command :Command;
    match Command::from_string(cmd_str.as_str()) {
        Some(c) => command = c,
        None => {
            let _ = write(&mut stdout, COMMAND_MSG);
            std::process::exit(2);
        }

    }
    command.execute(target_str.as_str());
}
