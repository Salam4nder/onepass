mod file;
mod command;

use std::env;
use std::io::{self, Write, Read};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use crate::command::Kind;

fn write(s: &mut StandardStream, msg: &str) -> io::Result<()> {
        writeln!(s, "{}", msg)
}

const PARSE_MSG:   &str = "expecting argument: {{command}}";
const COMMAND_MSG: &str = "expecting {{command}} as first argument: init, new, get, suggest. example: onepass init";

fn main() {
    let mut stdin = io::stdin();
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green))).unwrap_or_default();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        let _ = write(&mut stdout, PARSE_MSG);
    }
    let command_string = &args[1];

    let command_kind: Kind;
    if let Some(c) = command::Kind::from_string(command_string.as_str()) {
        command_kind = c;
    } else {
        let _ = write(&mut stdout, COMMAND_MSG);
        std::process::exit(2);
    }

    match command_kind {
        Kind::Init =>  {
            if file::exists() {
                let _ = write(&mut stdout, "you are all setup! run `onepass help`");
                std::process::exit(0);
            }
            let master_password = ask_for_master_password(&mut stdin, &mut stdout).unwrap_or_else(|err| {
                let _ = write(&mut stdout, err.as_str());
                std::process::exit(2);
            });
        },
        Kind::New => (),
        Kind::Get => (),
        Kind::Suggest => (),
    }

}

fn ask_for_master_password(i: &mut io::Stdin, o: &mut StandardStream) -> Result<String, String> {
    let _ = write(o, "please input your master password: \n");

    let mut input = String::new();
    match i.read_to_string(&mut input) {
        Ok(_) => (),
        Err(err) => {
            return Err(format!("err: {}", err));
        },
    };

    if input.trim().is_empty() {
        return Err("password can not be empty".to_string());
    };
    if input.contains(' ') {
        return Err("password can not contain spaces".to_string());
    };
    Ok(input)
}
