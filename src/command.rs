use crate::file;
use crate::input;
use std::io::Write;
use rand::rngs::OsRng;
use std::io::Stdin;
use std::sync::atomic::{AtomicBool, Ordering};

use clipboard::ClipboardProvider;
use clipboard::ClipboardContext;
use chacha20poly1305::{
    aead::AeadCore,
    ChaCha20Poly1305
};

pub static DONE: AtomicBool = AtomicBool::new(false);

const MSG_SETUP:     &str = "you are all setup! run `onepass help`";
const MSG_NOT_SETUP: &str = "you are not setup! run `onepass init`";
const MSG_GET:       &str = "expecting resource: e.g - onepass get soundcloud";

pub enum Kind {
    New,
    Get,
    Init,
    List,
    Purge,
    Suggest,
}

impl Kind {
    pub fn from_string(s: &str) -> Option<Kind> {
        match s {
            "new"     => return Some(Kind::New),
            "get"     => return Some(Kind::Get),
            "init"    => return Some(Kind::Init),
            "list"    => return Some(Kind::List),
            "purge"   => return Some(Kind::Purge),
            "suggest" => return Some(Kind::Suggest),
            _         => return None,
        }
    }
}

pub fn init() -> Result<(), String> {
    if file::exists() {
        return Err(MSG_SETUP.to_string())
    }

    let master_password = input::master_password()?;

    let mut root_file = match file::create() {
        Ok(v) => v,
        Err(err) => return Err(err.to_string()) 
    };

    let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
    if let Err(err) = root_file.write_all(&nonce.to_vec()) {
        return Err(err.to_string())
    };
    let mut content = String::from("\n");
    content.push_str(file::DELIMITER);
    content.push_str("\n");

    let encrypted_content = file::encrypt(&master_password, &content, nonce)?;
    if let Err(err) = root_file.write_all(&encrypted_content) {
        return Err(err.to_string())
    };
    DONE.store(true, Ordering::Relaxed);
    Ok(())
}

pub fn new(stdin: &mut Stdin) -> Result<(), String> {
    if !file::exists() {
        return Err(MSG_NOT_SETUP.to_string())
    }
    // TODO(kg): don't open file multiple times?
    let pw = input::master_password()?;
    let res = input::resource(stdin)?;
    let mut open_file = match file::open() {
        Ok(v) => v,
        Err(err) => return Err(err.to_string()) 
    };
    let data = match file::extract_data(&mut open_file) { 
        Ok(v) => v,
        Err(err) => return Err(err.to_string()) 
    };
    let mut decrypted_content = file::decrypt(&pw, data.buf, data.nonce)?;

    let mut truncated_file = match file::open_truncate() {
        Ok(v) => v,
        Err(err) => return Err(err.to_string()) 
    };
    let new_nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
    if let Err(err) = truncated_file.write_all(&new_nonce.to_vec()) {
        return Err(err.to_string())
    };
    decrypted_content.push_str(&res.to_string());
    let encrypted_content = file::encrypt(&pw, &decrypted_content, new_nonce)?;
    let mut f = match file::open_append() {
        Ok(v) => v,
        Err(err) => return Err(err.to_string()) 
    };
    if let Err(err) = f.write_all(&encrypted_content) {
        return Err(err.to_string())
    };
    DONE.store(true, Ordering::Relaxed);
    Ok(())
}

pub fn get(args: Vec<String>) -> Result<(), String> {
    if !file::exists() {
        return Err(MSG_NOT_SETUP.to_string());
    }
    if args.len() < 3 {
        return Err(MSG_GET.to_string());
    }
    let resource_str = &args[2];
    if input::is_reserved(resource_str) {
        return Err("use of reserved keyword".to_string());
    }
    let master_password = input::master_password()?;
    let mut f = match std::fs::File::open(file::path()) {
        Ok(v) => v,
        Err(err) => return Err(err.to_string())
    };

    let data = match file::extract_data(&mut f) {
        Ok(v) => v,
        Err(err) => return Err(err.to_string())
    };

    let decrypted_content = file::decrypt(
        &master_password,
        data.buf,
        data.nonce,
    )?;

    let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
    let (user, pw): (String, String);
    let lines: Vec<&str> = decrypted_content.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        if *line == input::RESERVED_RESOURCE && lines[i+1] == resource_str {
            if let (Some(next), Some(next_next)) = (lines.get(i + 2), lines.get(i + 3)) {
                user = next.to_string();
                pw = next_next.to_string();
                println!("{}", user);
                if let Err(err) = ctx.set_contents(pw.to_owned()) {
                    println!("could not copy password to clipboard: {}", err);
                    println!("printing password, make sure to clear your terminal...");
                    println!("{}", pw);
                };
                println!("{}", format!("found resource {}", user));
                println!("password copied to clipboard");
                return Ok(());
            } else {
                return Err("uncomplete resource is less than 3 lines".to_string());
            }
        }
    }
    println!("resource not found");
    Ok(())
}

pub fn list() -> Result<(), String> {
    if !file::exists() {
        return Err(MSG_NOT_SETUP.to_string());
    }
    let pw = input::master_password()?;
    let mut f = match file::open() {
        Ok(v) => v,
        Err(err) => return Err(err.to_string())
    };
    let data = match file::extract_data(&mut f) {
        Ok(v) => v,
        Err(err) => return Err(err.to_string())
    };
    let decrypted_content = file::decrypt(&pw, data.buf, data.nonce)?;
    let lines: Vec<&str> = decrypted_content.lines().collect();
    let mut result: Vec<String> = vec![];
    for (i, line) in decrypted_content.lines().into_iter().enumerate(){
        if line == input::RESERVED_RESOURCE {
            result.push(String::from(lines[i+1]))
        }
    }
    for v in result {
        println!("{}", v);
    }
    Ok(())
}

pub fn purge() -> Result<(), String> {
    if let Err(err) = std::fs::remove_dir_all(file::dir_path()) {
        return Err(err.to_string());
    };
    Ok(())
}
