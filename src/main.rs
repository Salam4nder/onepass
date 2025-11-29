mod command;
mod file;
mod input;
mod password;
mod resource;
mod text;

use command::Kind;
use ctrlc;
use std::env;
use std::sync::atomic::Ordering;

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
        println!("onepass: WARNING: bad state, run `onepass purge`")
    })
    .expect("setting ctrl-c handler");

    let mut stdin = std::io::stdin();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("{}", text::MSG_HELP);
        std::process::exit(0);
    }
    let command_string = &args[1];

    let mut path: Option<String> = None;
    for i in 2..args.len() {
        if args[i] == "-l" || args[i] == "--location" {
            match args.get(i + 1) {
                None => {
                    println!("{}", text::MSG_HELP);
                    std::process::exit(0);
                }
                Some(v) => path = Some(v.to_string()),
            }
        }
    }

    let cmd = match command::Kind::from_string(command_string.as_str()) {
        Some(v) => v,
        None => {
            println!("{}", text::MSG_HELP);
            std::process::exit(1);
        }
    };
    match cmd {
        Kind::New => {
            if let Err(err) = command::new(path.as_deref(), &mut stdin) {
                println!("{}", &err);
            };
        }
        Kind::Get => {
            if let Err(err) = command::get(path.as_deref(), args) {
                println!("{}", &err);
            };
        }
        Kind::Del => {
            if let Err(err) = command::del(path.as_deref(), args) {
                println!("{}", &err);
            };
        }
        Kind::Suggest => {
            println!("{}", command::suggest());
        }
        Kind::List => {
            if let Err(err) = command::list(path.as_deref()) {
                println!("{}", &err);
            };
        }
        Kind::Purge => {
            if let Err(err) = command::purge() {
                println!("{}", &err);
            };
        }
        Kind::Update => {
            if let Err(err) = command::update(path.as_deref(), args, &mut stdin) {
                println!("{}", &err);
            };
        }
        Kind::Help => {
            println!("{}", command::help(args));
        }
    }
}
