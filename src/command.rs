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
    Help,
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
            "help"    => return Some(Kind::Help),
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
        return Err(text::MSG_COMMAND_GET.to_string());
    }
    let res = &args[2];
    if input::is_reserved(res) {
        return Err("use of reserved keyword".to_string());
    }
    let pw = input::master_password()?;

    let resp = file::get(None, &pw, res)?;
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
        return Err(text::MSG_COMMAND_UPDATE.to_string());
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
        return Err(text::MSG_COMMAND_DEL.to_string());
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

pub fn help(args: Vec<String>) -> String {
    if args.len() != 3 {
        return text::MSG_HELP.to_string()
    };

    if let Some(command) = Kind::from_string(&args[2]) {
        match command {
            Kind::Get    => return text::MSG_COMMAND_GET.to_string(),
            Kind::Del    => return text::MSG_COMMAND_DEL.to_string(),
            Kind::Update => return text::MSG_COMMAND_UPDATE.to_string(),
                    _    => return text::MSG_HELP.to_string()
        }
    } else {
        return text::MSG_HELP.to_string()
    }
}
