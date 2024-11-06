mod file;
mod command;
mod encrypt;
mod resource;

use std::env;
use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};

extern crate clipboard;
extern crate rpassword;

// use termion::input::TermRead;
use clipboard::ClipboardProvider;
use clipboard::ClipboardContext;

use ctrlc;
use std::io::{self, Write};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use command::Kind;
use chacha20poly1305::{
    aead::{AeadCore, OsRng},
    ChaCha20Poly1305
};

fn is_reserved(input: &str) -> bool {
    input == RESERVED_NONCE || input == RESERVED_RESOURCE
}

fn write(s: &mut StandardStream, msg: &str) -> io::Result<()> {
        writeln!(s, "{}", msg)
}

const RESERVED_NONCE:    &str = "nonce";
const RESERVED_RESOURCE: &str = "resource";

const GET_MSG:     &str = "expecting resource: e.g - onepass get soundcloud";
const COMMAND_MSG: &str = "expecting {{command}} as first argument: init, new, get, suggest. example: onepass init";

static DONE:       AtomicBool = AtomicBool::new(false);
static INPUT_MODE: AtomicBool = AtomicBool::new(false);

fn main() {
    ctrlc::set_handler(move || {
        println!("onepass: cleaning up...");
        if INPUT_MODE.load(Ordering::Relaxed) {
            std::process::exit(1);
        };
        let max_retries = 5;
        for _ in 0..max_retries {
            if DONE.load(Ordering::Relaxed) {
                std::process::exit(1);
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
        // TODO(kg): should not happen, revisit this.
        println!("onepass: WARNING: bad state, run `onepass purge`")
    }).expect("setting ctrl-c handler");

    let mut stdin = io::stdin();
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green))).unwrap_or_default();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        let _ = write(&mut stdout, COMMAND_MSG);
        std::process::exit(0);
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
                std::process::exit(1);
            }
            let master_password = ask_for_master_password().unwrap_or_else(|err| {
                let _ = write(&mut stdout, err.as_str());
                std::process::exit(1);
            });

            let mut root_file = match file::create() {
                Ok(file) => file,
                Err(err) => {
                    let _ = write(&mut stdout, &err.to_string());
                    std::process::exit(1);
                }
            };
            println!("sleeping for 5 secs");
            std::thread::sleep(std::time::Duration::from_secs(5));

            let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);

            if let Err(err) = root_file.write_all(&nonce.to_vec()) {
                let _ = write(&mut stdout, &err.to_string());
                std::process::exit(1);
            };

            let mut content = String::from("\n");
            content.push_str(file::DELIMITER);
            content.push_str("\n");

            let encrypted_content = match encrypt::encrypt(&master_password, &content, nonce) {
                Ok(v) => v,
                Err(err) => {
                    let _ = write(&mut stdout, &err.to_string());
                    std::process::exit(1);
                },
            };

            if let Err(err) = root_file.write_all(&encrypted_content) {
                let _ = write(&mut stdout, &err.to_string());
                std::process::exit(1);
            }
            DONE.store(true, Ordering::Relaxed);
        },
        Kind::New => {
            if !file::exists() {
                let _ = write(&mut stdout, "you are not setup! run onepass init");
                std::process::exit(1);
            }
            // TODO(kg): don't open file multiple times?
            let pw = ask_for_master_password().expect("asking for master password");
            let res = ask_for_resource(&mut stdin, &mut stdout).expect("asking for resource");

            let mut open_file = file::open().expect("open");
            let data = file::extract_data(&mut open_file).expect("extracting data");
            let mut decrypted_content = encrypt::decrypt(&pw, data.buf, data.nonce).expect("decrypting");

            let mut truncated_file = file::open_truncate().expect("open truncate");
            let new_nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
            truncated_file.write_all(&new_nonce.to_vec()).expect("writing all");

            decrypted_content.push_str(&res.to_string());
            let encrypted_content = encrypt::encrypt(&pw, &decrypted_content, new_nonce).expect("encrypting");

            let mut f = file::open_append().expect("open append");
            f.write_all(&encrypted_content).expect("write all");
            DONE.store(true, Ordering::Relaxed);
        },
        Kind::Get => {
            if args.len() < 3 {
                let _ = write(&mut stdout, GET_MSG);
                std::process::exit(1);

            }
            let resource_str = &args[2];
            if is_reserved(resource_str) {
                let _ = write(&mut stdout, "use of a reserved keyword");
                std::process::exit(1);
            }
            if !file::exists() {
                let _ = write(&mut stdout, "please run onepass init to setup!");
                std::process::exit(1);
            }
            let master_password = ask_for_master_password().unwrap_or_else(|err| {
                let _ = write(&mut stdout, err.as_str());
                std::process::exit(1);
            });
            let mut f = match fs::File::open(file::path()) {
                Ok(v) => v,
                Err(err) => {
                    let _ = write(&mut stdout, &err.to_string());
                    std::process::exit(1);
                },
            };

            let data = match file::extract_data(&mut f) {
                Ok(v) => v,
                Err(_) => {
                    let _ = write(&mut stdout, "extracting data");
                    std::process::exit(1);
                }
            };

            let decrypted_content = match encrypt::decrypt(&master_password, data.buf, data.nonce) {
                Ok(v) => v,
                Err(_) => {
                    let _ = write(&mut stdout, "decrypting");
                    std::process::exit(1);
                },
            };

            let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
            let (user, pw): (String, String);
            let lines: Vec<&str> = decrypted_content.lines().collect();
            for (i, line) in lines.iter().enumerate() {
                if *line == RESERVED_RESOURCE && lines[i+1] == resource_str {
                    if let (Some(next), Some(next_next)) = (lines.get(i + 2), lines.get(i + 3)) {
                        user = next.to_string();
                        pw = next_next.to_string();
                        ctx.set_contents(pw.to_owned()).expect("setting clip");
                        println!("{}", user);
                        println!("{}", pw);
                        let _ = write(&mut stdout, &format!("found resource {}", user).to_string());
                        let _ = write(&mut stdout, "password copied to clipboard");
                        std::process::exit(0);
                    } else {
                        let _ = write(&mut stdout, "uncomplete resource is less than 3 lines");
                        std::process::exit(1);
                    }
                }
            }
            let _ = write(&mut stdout, "resource not found");
            std::process::exit(0);
        },
        Kind::Suggest => (),
        Kind::List => {
            if !file::exists() {
                let _ = write(&mut stdout, "please run onepass init to setup!");
                std::process::exit(1);
            }
            let pw = ask_for_master_password().expect("asking for pw");
            let mut f = file::open().expect("opening");
            let data = file::extract_data(&mut f).expect("extracting");
            let decrypted_content = encrypt::decrypt(&pw, data.buf, data.nonce).expect("decrypting");
            let lines: Vec<&str> = decrypted_content.lines().collect();
            let mut result: Vec<String> = vec![];
            for (i, line) in decrypted_content.lines().into_iter().enumerate(){
                if line == RESERVED_RESOURCE {
                    result.push(String::from(lines[i+1]))
                }
            }
            for v in result {
                write(&mut stdout, &v).expect("writing");
            }
        },
        Kind::Purge => {
            if let Err(err) = fs::remove_dir_all(file::dir_path()) {
                let _ = write(&mut stdout, &err.to_string());
                std::process::exit(0);
            };
        },
    }

}

fn ask_for_master_password() -> Result<String, String> {
    INPUT_MODE.store(true, Ordering::Relaxed);
    let input = rpassword::prompt_password("please input your master password: ").expect("reading");
    INPUT_MODE.store(false, Ordering::Relaxed);

    if input.trim().is_empty() {
        return Err("password can not be empty".to_string());
    };
    if input.contains(' ') {
        return Err("password can not contain spaces".to_string());
    };
    Ok(input)
}

fn ask_for_resource(i: &mut io::Stdin, o: &mut StandardStream) -> Result<resource::Instance, String> {
    INPUT_MODE.store(true, Ordering::Relaxed);
    let mut fn_ask_for = |m: &str| -> Result<String, String> {
        let _ = write(o, m);
        let mut input = String::new();
        i.read_line(&mut input).expect("reading");
        if is_reserved(&input) {
            return Err("use of reserved keyword".to_string());
        };
        Ok(input)
    };
    let name     = fn_ask_for("resource: ")?;
    let user     = fn_ask_for("user: ")?;
    let password = fn_ask_for("password: ")?;
    INPUT_MODE.store(false, Ordering::Relaxed);
    Ok(resource::Instance{
        name,
        user,
        password,
    })
}
