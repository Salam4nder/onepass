use std::io::Stdin;
use std::os::unix::fs::MetadataExt;
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

/// Create a new resource and append it to the file.
pub fn new(custom_path: Option<&str>, stdin: &mut Stdin) -> Result<(), String> {
    if !file::exists(custom_path) {
        if let Err(err) =  file::create(custom_path) {
            return Err(err.to_string())
        }
    } 

    let resource = input::resource(stdin)?;
    let password = input::master_password()?;
    new_resource(custom_path, &password, resource)?;

    DONE.store(true, Ordering::Relaxed);
    Ok(())
}

fn new_resource(
    custom_path: Option<&str>,
    password: &str,
    resource: resource::Instance,
) -> Result<(), String> {
    let path = file::path(custom_path);
    let metadata = match std::fs::metadata(path) {
        Ok(v) => v, 
        Err(err) => return Err(err.to_string())
    };

    let mut content = String::new();
    if metadata.size() > 0 {
        content = file::decrypt(custom_path, password)?;
        let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        for line in &lines {
            if line.trim() == resource.name.trim() {
                return Err("Resource already exists".to_string())
            }
        }
        lines.push(resource.to_string());
        content = lines.join("\n");
    } else {
        content.push_str(resource.to_string().as_str());
    }

    file::encrypt(custom_path, &password, content)?;
    Ok(())
}

pub fn get(custom_path: Option<&str>, args: Vec<String>) -> Result<(), String> {
    if args.len() < 3 {
        return Err(text::MSG_COMMAND_GET.to_string());
    }

    if !file::exists(custom_path) {
        return Err(text::MSG_NO_RESOURCES.to_string());
    } 

    let password = input::master_password()?;
    let resource_name = &args[2];
    if input::is_reserved(resource_name) {
        return Err("use of reserved keyword".to_string());
    }

    let got = get_resource(custom_path, &password, resource_name)?;
    println!("Username: {}", got.user);
    let mut ctx: ClipboardContext = match ClipboardProvider::new() {
        Ok(v) => v,
        Err(err) => return Err(err.to_string()),
    };
    if let Err(_) = ctx.set_contents(got.password.to_owned()) {
        println!("Password: {}", got.password);
        println!("Don't forget to clear your terminal");
    } else {
        println!("Password copied to clipboard");
    };

    DONE.store(true, Ordering::Relaxed);
    Ok(())
}

fn get_resource(
    custom_path: Option<&str>,
    password: &str,
    resource_name: &str,
) -> Result<resource::Instance, String> {
    let content = file::decrypt(custom_path, &password)?;
    let got = resource::get(resource_name, &content)?;
    Ok(got)
}

pub fn list(custom_path: Option<&str>) -> Result<(), String> {
    if !file::exists(custom_path) {
        return Err(text::MSG_NO_RESOURCES.to_string());
    } 

    let password = input::master_password()?;

    let result = list_resources(custom_path, &password)?;
    if result.len() < 1 {
        return Err(text::MSG_NO_RESOURCES.to_string());
    }
    for v in result {
        println!("{}", v);
    }
    DONE.store(true, Ordering::Relaxed);

    Ok(())
}

fn list_resources(custom_path: Option<&str>, password: &str) -> Result<Vec<String>, String> {
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
