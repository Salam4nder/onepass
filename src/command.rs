use std::io::Stdin;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::text;
use crate::file;
use crate::input;
use crate::resource;
use crate::password;

use clipboard::ClipboardProvider;
use clipboard::ClipboardContext;

pub static DONE: AtomicBool = AtomicBool::new(false);

pub enum Kind {
    New,
    Get,
    Del,
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
/// Returns the inputted master password.
pub fn init() -> Result<String, String> {
    if file::exists(None) {
        return Err("file already initialised".to_string())
    }

    let pw = input::master_password()?;
    file::bootstrap(None, &pw)?;
    println!("{}", text::MSG_SETUP);

    DONE.store(true, Ordering::Relaxed);
    Ok(pw)
}

pub fn new(stdin: &mut Stdin) -> Result<(), String> {
    let pw: String;
    if !file::exists(None) {
        pw = init()?
    } else {
        pw = input::master_password()?;
    }

    let res = input::resource(stdin)?;
    file::write(None, &pw, res)?;

    DONE.store(true, Ordering::Relaxed);
    Ok(())
}

pub fn get(args: Vec<String>) -> Result<(), String> {
    if args.len() < 3 {
        return Err(text::MSG_COMMAND_GET.to_string());
    }

    let pw: String;
    if !file::exists(None) {
        pw = init()?
    } else {
        pw = input::master_password()?;
    }

    let target = &args[2];
    if input::is_reserved(target) {
        return Err("use of reserved keyword".to_string());
    }

    let mut f = match file::open(None) {
        Ok(v) => v,
        Err(err) => return Err(err.to_string())
    };
    let data = match file::extract_data(&mut f) {
        Ok(v) => v,
        Err(err) => return Err(err.to_string())
    };
    let decrypted_content = file::decrypt(
        &pw,
        data.buf,
        data.nonce,
    )?;

    let got = resource::get(target, decrypted_content)?;
    println!("Username: {}", got.user);
    let mut ctx: ClipboardContext = match ClipboardProvider::new() {
        Ok(v) => v,
        Err(err) => return Err(err.to_string()),
    };
    if let Err(_) = ctx.set_contents(got.password.to_owned()) {
        println!("Password: {}", got.password);
        println!("Don't forget to clear your terminal.");
    } else {
        println!("Password copied to clipboard");
    };

    DONE.store(true, Ordering::Relaxed);
    Ok(())
}

pub fn list() -> Result<(), String> {
    let pw: String;
    if !file::exists(None) {
        pw = init()?
    } else {
        pw = input::master_password()?;
    }

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
    if args.len() < 3 {
        return Err(text::MSG_COMMAND_UPDATE.to_string());
    }

    let pw: String;
    if !file::exists(None) {
        pw = init()?
    } else {
        pw = input::master_password()?;
    }

    let res = &args[2];
    if input::is_reserved(res) {
        return Err("use of reserved keyword".to_string());
    }
    let (key, val) = input::update_resource(stdin)?;
    file::update(None, &pw, res, &key, &val)?;

    DONE.store(true, Ordering::Relaxed);
    Ok(())
}

pub fn del(args: Vec<String>) -> Result<(), String> {
    if args.len() < 3 {
        return Err(text::MSG_COMMAND_DEL.to_string());
    }

    let pw: String;
    if !file::exists(None) {
        pw = init()?
    } else {
        pw = input::master_password()?;
    }

    let res = &args[2];
    if input::is_reserved(res) {
        return Err("use of reserved keyword".to_string());
    }
    file::delete(None, &pw, &res)?;

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
