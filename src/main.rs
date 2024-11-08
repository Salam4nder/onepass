mod file;
mod input;
mod command;
mod resource;

use std::env;
use std::sync::atomic::Ordering;

extern crate clipboard;
extern crate rpassword;

use ctrlc;
use command::Kind;

const COMMAND_MSG: &str = "expecting {{command}} as first argument: init, new, get, suggest. example: onepass init";

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
        println!("{}", COMMAND_MSG);
        std::process::exit(0);
    }
    let command_string = &args[1];

    let cmd = match command::Kind::from_string(command_string.as_str()) {
        Some(v) => v,
        None => {
            println!("{}", COMMAND_MSG);
            std::process::exit(1);
        },
    };
    match cmd {
        Kind::Init =>  {
            if let Err(err) = command::init() {
               println!("{}", &err);
            };
        },
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
        Kind::Suggest => (),
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
    }
}
