mod command;
mod file;
mod input;
mod password;
mod resource;
mod text;

use std::{
    env::{self},
    process,
    sync::atomic::Ordering,
};

use command::Kind;

fn main() {
    ctrlc::set_handler(move || {
        println!("onepass: cleaning up...");
        if input::MODE.load(Ordering::Relaxed) {
            process::exit(1);
        }
        let max_retries = 5;
        for _ in 0..max_retries {
            if command::DONE.load(Ordering::Relaxed) {
                process::exit(1);
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
        println!("onepass: WARNING: bad state, run `onepass purge`");
    })
    .expect("setting ctrl-c handler");

    let mut stdin = std::io::stdin();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("{}", text::MSG_HELP);
        std::process::exit(0);
    }
    let command_string = &args[1];
    let custom_path = if let Some(v) = args.get(3)
        && (v == "-l" || v == "--location")
    {
        if let Some(vv) = args.get(4) {
            Some(vv.as_str())
        } else {
            println!("{}", text::MSG_HELP);
            std::process::exit(0);
        }
    } else {
        None
    };

    let Some(cmd) = command::Kind::from_string(command_string.as_str()) else {
        println!("{}", text::MSG_HELP);
        std::process::exit(1);
    };
    match cmd {
        Kind::New => {
            if let Err(err) = command::new(custom_path, &mut stdin) {
                println!("{err}");
            }
        }
        Kind::Get => {
            let Some(argument_string) = args.get(2) else {
                println!("{}", text::MSG_HELP);
                std::process::exit(0);
            };
            match command::get(custom_path, argument_string) {
                Ok(_) => input::drop_clipboard_ctx(&mut stdin),
                Err(e) => println!("{e}"),
            }
        }
        Kind::Del => {
            let Some(argument_string) = args.get(2) else {
                println!("{}", text::MSG_HELP);
                std::process::exit(0);
            };
            if let Err(err) = command::del(custom_path, argument_string) {
                println!("{err}");
            }
        }
        Kind::Suggest => {
            println!("{}", command::suggest());
        }
        Kind::List => {
            if let Err(err) = command::list(custom_path) {
                println!("{err}");
            }
        }
        Kind::Purge => {
            if let Err(err) = command::purge() {
                println!("{err}");
            }
        }
        Kind::Update => {
            let Some(argument_string) = args.get(2) else {
                println!("{}", text::MSG_HELP);
                std::process::exit(0);
            };
            if let Err(err) = command::update(custom_path, argument_string, &mut stdin) {
                println!("{err}");
            }
        }
        Kind::Help => {
            let Some(argument_string) = args.get(2) else {
                println!("{}", text::MSG_HELP);
                std::process::exit(0);
            };
            println!("{}", command::help(argument_string));
        }
    }
}
