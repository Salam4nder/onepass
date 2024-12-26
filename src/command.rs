use crate::text;
use crate::file;
use crate::input;
use crate::password;
use std::io::Write;
use std::io::Stdin;
use std::sync::atomic::{AtomicBool, Ordering};

use rand::rngs::OsRng;
use chacha20poly1305::{
    aead::AeadCore,
    ChaCha20Poly1305
};

pub static DONE: AtomicBool = AtomicBool::new(false);

pub enum Kind {
    New,
    Get,
    Del,
    Init,
    List,
    Purge,
    Update,
    Suggest,
}

impl Kind {
    pub fn from_string(s: &str) -> Option<Kind> {
        match s {
            "new"     => return Some(Kind::New),
            "get"     => return Some(Kind::Get),
            "del"     => return Some(Kind::Del),
            "init"    => return Some(Kind::Init),
            "list"    => return Some(Kind::List),
            "purge"   => return Some(Kind::Purge),
            "update"  => return Some(Kind::Update),
            "suggest" => return Some(Kind::Suggest),
            _         => return None,
        }
    }
}

/// Initialize the onepass engine by creating the needed directory and file.
pub fn init() -> Result<(), String> {
    if file::exists(None) {
        return Err(text::MSG_SETUP.to_string())
    }

    let pw = input::master_password()?;
    file::bootstrap(None, &pw)?;

    DONE.store(true, Ordering::Relaxed);
    Ok(())
}

pub fn new(stdin: &mut Stdin) -> Result<(), String> {
    if !file::exists(None) {
        return Err(text::MSG_NOT_SETUP.to_string())
    }
    let pw = input::master_password()?;
    let res = input::resource(stdin)?;
    file::write(None, &pw, res)?;

    DONE.store(true, Ordering::Relaxed);
    Ok(())
}

pub fn get(args: Vec<String>) -> Result<(), String> {
    if !file::exists(None) {
        return Err(text::MSG_NOT_SETUP.to_string());
    }
    if args.len() < 3 {
        return Err(text::MSG_GET.to_string());
    }
    let res = &args[2];
    if input::is_reserved(res) {
        return Err("use of reserved keyword".to_string());
    }
    let pw = input::master_password()?;

    let resp = file::get(None, res, &pw)?;
    println!("username: {}", resp.resource.user);
    if !resp.copied {
        println!("printing password, make sure to copy it and clear your terminal...");
        println!("{}", resp.resource.password);
    } else {
        println!("password copied to clipboard");
    };

    DONE.store(true, Ordering::Relaxed);
    Ok(())
}

pub fn list() -> Result<(), String> {
    if !file::exists(None) {
        return Err(text::MSG_NOT_SETUP.to_string());
    }
    let pw = input::master_password()?;
    let result = file::list(None, &pw)?;
    if result.len() < 1 {
        println!("no saved resources");
    }
    for v in result {
        println!("{}", v);
    }
    DONE.store(true, Ordering::Relaxed);
    Ok(())
}

pub fn purge() -> Result<(), String> {
    if let Err(err) = file::purge(None) {
        return Err(err.to_string());
    };
    DONE.store(true, Ordering::Relaxed);
    Ok(())
}

pub fn suggest() -> String {
    DONE.store(true, Ordering::Relaxed);
    password::suggest(14)
}

pub fn update(stdin: &mut Stdin, args: Vec<String>) -> Result<(), String> {
    if !file::exists(None) {
        return Err(text::MSG_NOT_SETUP.to_string());
    }
    if args.len() < 3 {
        return Err(text::MSG_GET.to_string());
    }
    let res = &args[2];
    if input::is_reserved(res) {
        return Err("use of reserved keyword".to_string());
    }
    let password = input::master_password()?;
    let (key, val) = input::update_resource(stdin)?;
    file::update(None, &password, res, &key, &val)?;

    DONE.store(true, Ordering::Relaxed);
    Ok(())
}

pub fn del(args: Vec<String>) -> Result<(), String> {
    if !file::exists(None) {
        return Err(text::MSG_NOT_SETUP.to_string());
    }
    if args.len() < 3 {
        return Err(text::MSG_GET.to_string());
    }
    let resource_str = &args[2];
    if input::is_reserved(resource_str) {
        return Err("use of reserved keyword".to_string());
    }
    let master_password = input::master_password()?;

    let mut f = match file::open(None) {
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

    let mut data = vec![];
    let mut found: bool = false;
    let mut res_idx: usize = 0;
    let mut user_idx: usize = 0;
    let mut pass_idx: usize = 0;
    let lines: Vec<&str> = decrypted_content.lines().collect();
    for (i, _) in lines.iter().enumerate() {
        if lines[i] == "\n" {
            continue
        }
        if lines[i] == input::RESERVED_RESOURCE && lines[i+1] == resource_str {
            found = true;
            res_idx  = i+1;
            user_idx = i+2;
            pass_idx = i+3;
            continue
        }
        if found && i == res_idx || i == user_idx || i == pass_idx {
            continue
        } else {
            data.push(String::from(lines[i]));
        }
    }
    if !found {
        return Err("resource not found".to_string())
    }
    let new_nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
    let encrypted_content = file::encrypt(&master_password, &data.join("\n"), new_nonce)?;
    let mut truncated_file = match file::open_truncate(None) {
        Ok(v) => v,
        Err(err) => return Err(err.to_string()) 
    };
    if let Err(err) = truncated_file.write_all(&new_nonce.to_vec()) {
        return Err(err.to_string())
    };
    if let Err(err) = truncated_file.write_all(&encrypted_content) {
        return Err(err.to_string())
    };
    DONE.store(true, Ordering::Relaxed);
    Ok(())
}
