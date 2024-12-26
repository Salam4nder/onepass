use crate::text;
use crate::file;
use crate::input;
use crate::password;
use std::io::Stdin;
use std::sync::atomic::{AtomicBool, Ordering};

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
    let res = &args[2];
    if input::is_reserved(res) {
        return Err("use of reserved keyword".to_string());
    }
    let password = input::master_password()?;
    file::delete(None, &password, &res)?;

    DONE.store(true, Ordering::Relaxed);
    Ok(())
}
