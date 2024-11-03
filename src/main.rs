mod file;
mod command;
mod encrypt;
mod resource;

use std::env;
use std::fs;
use std::path::PathBuf;
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

fn main() {
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
            let master_password = ask_for_master_password(&mut stdin, &mut stdout).unwrap_or_else(|err| {
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
        },
        Kind::New => {
            let pw = ask_for_master_password(&mut stdin, &mut stdout).expect("asking for master password");
            // TODO(kg): don't open file multiple times?
            let mut open_file = file::open().expect("open");
            let data = file::extract_data(&mut open_file).expect("extracting data");
            let mut decrypted_content = encrypt::decrypt(&pw, data.buf, data.nonce).expect("decrypting");

            let mut truncated_file = file::open_truncate().expect("open truncate");
            let new_nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
            truncated_file.write_all(&new_nonce.to_vec()).expect("writing all");

            let res = ask_for_resource(&mut stdin, &mut stdout).expect("asking for resource");

            decrypted_content.push_str(&res.to_string());
            let encrypted_content = encrypt::encrypt(&pw, &decrypted_content, new_nonce).expect("encrypting");

            let mut f = file::open_append().expect("open append");
            f.write_all(&encrypted_content).expect("write all");

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
            let master_password = ask_for_master_password(&mut stdin, &mut stdout).unwrap_or_else(|err| {
                let _ = write(&mut stdout, err.as_str());
                std::process::exit(1);
            });
            let mut f = match fs::File::open(file::file_path()) {
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
            let (user, pw): (String, String);
            let lines: Vec<&str> = decrypted_content.lines().collect();
            for (i, line) in lines.iter().enumerate() {
                if *line == RESERVED_RESOURCE && lines[i+1] == resource_str {
                    if let (Some(next), Some(next_next)) = (lines.get(i + 2), lines.get(i + 3)) {
                        user = next.to_string();
                        pw = next_next.to_string();
                        println!("{}", user);
                        println!("{}", pw);
                        return;
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
        Kind::Purge => {
            if let Err(err) = fs::remove_dir_all(file::dir_path()) {
                let _ = write(&mut stdout, &err.to_string());
                std::process::exit(0);
            };
        },
    }

}

fn ask_for_master_password(i: &mut io::Stdin, o: &mut StandardStream) -> Result<String, String> {
    let _ = write(o, "please input your master password: \n");

    let mut input = String::new();
    match i.read_line(&mut input) {
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

fn ask_for_resource(i: &mut io::Stdin, o: &mut StandardStream) -> Result<resource::Instance, String> {
    let mut fn_ask_for = |m: &str| -> Result<String, String> {
        let _ = write(o, m);
        let mut s = String::new();
        match i.read_line(&mut s) {
            Ok(_) => (),
            Err(err) => {
                return Err(format!("reading {}: {}", m, err));
            },
        };
        if is_reserved(&s) {
            return Err("use of reserved keyword".to_string());
        };
        Ok(s)
    };
    let name = fn_ask_for("resource: ")?;
    let user     = fn_ask_for("user: ")?;
    let password = fn_ask_for("password: ")?;
    Ok(resource::Instance{
        name,
        user,
        password,
    })
}
