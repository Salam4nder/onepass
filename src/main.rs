mod file;
mod command;

use std::env;
use std::io::{self, Write};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use crate::command::Kind;

fn write(s: &mut StandardStream, msg: &str) -> io::Result<()> {
        writeln!(s, "{}", msg)
}

const PARSE_MSG:   &str = "expecting argument: {{command}}";
const COMMAND_MSG: &str = "expecting {{command}} as first argument: init, new, get, suggest. example: onepass init";

fn main() {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green))).unwrap_or_default();

    let _std_in = io::stdin();

    let args: Vec<String> = env::args().collect();
    if args.len() < 1 {
        let _ = write(&mut stdout, PARSE_MSG);
    }
    let command_string = &args[0];

    let command_kind: Kind;
    if let Some(c) = command::Kind::from_string(command_string.as_str()) {
        command_kind = c;
    } else {
        let _ = write(&mut stdout, COMMAND_MSG);
        std::process::exit(2);
    }

    match command_kind {
        Kind::Init => (),
        Kind::New => (),
        Kind::Get => (),
        Kind::Suggest => (),
    }

}
