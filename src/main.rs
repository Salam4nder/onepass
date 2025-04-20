mod file;
mod text;
mod input;
mod command;
mod resource;
mod password;

use std::env;
use std::sync::atomic::Ordering;
use ctrlc;
use command::Kind;

extern crate clipboard;
extern crate rpassword;

fn main() {
    ctrlc::set_handler(move || {
        println!("onepass: cleaning up...");
        if input::MODE.load(Ordering::Relaxed) {
            std::process::exit(1);
        };
        let max_retries = 5;
        for _ in 0..max_retries {
            if command::DONE.load(Ordering::Relaxed) {
                std::process::exit(1);
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
        // TODO(kg): should not happen, revisit this.
        println!("onepass: WARNING: bad state, run `onepass purge`")
    }).expect("setting ctrl-c handler");

    let mut stdin = std::io::stdin();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("{}", text::MSG_HELP);
        std::process::exit(0);
    }
    let command_string = &args[1];

    let cmd = match command::Kind::from_string(command_string.as_str()) {
        Some(v) => v,
        None => {
            println!("{}", text::MSG_HELP);
            std::process::exit(1);
        },
    };
    match cmd {
        Kind::New => {
            if let Err(err) = command::new(&mut stdin) {
               println!("{}", &err);
            };
        },
        Kind::Get => {
            if let Err(err) = command::get(args) {
               println!("{}", &err);
            };
        },
        Kind::Del => {
            if let Err(err) = command::del(args) {
               println!("{}", &err);
            };
        },
        Kind::Suggest => {
            println!("{}", command::suggest());
        },
        Kind::List => {
            if let Err(err) = command::list() {
               println!("{}", &err);
            };
        },
        Kind::Purge => {
            if let Err(err) = command::purge() {
               println!("{}", &err);
            };
       },
        Kind::Update => {
            if let Err(err) = command::update(&mut stdin, args) {
               println!("{}", &err);
            };
       },
        Kind::Help => {
            println!("{}", command::help(args));
        },
    }
}
