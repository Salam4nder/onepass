use crate::password;
use crate::resource;
use crate::text;

use std::io::Stdin;
use std::sync::atomic::{AtomicBool, Ordering};

pub static MODE: AtomicBool = AtomicBool::new(false);

pub fn master_password() -> Result<String, String> {
    MODE.store(true, Ordering::Relaxed);
    let input = match rpassword::prompt_password("master password: ") {
        Ok(v) => v,
        Err(err) => return Err(err.to_string()),
    };
    MODE.store(false, Ordering::Relaxed);

    if input.trim().is_empty() {
        return Err("password can not be empty".to_string());
    };
    if input.contains(' ') {
        return Err("password can not contain spaces".to_string());
    };
    Ok(input)
}

pub fn resource(i: &mut Stdin) -> Result<resource::Instance, String> {
    MODE.store(true, Ordering::Relaxed);
    let fn_ask_for = |m: &str| -> Result<String, String> {
        println!("{}: ", m);
        let mut input = String::new();
        if let Err(err) = i.read_line(&mut input) {
            return Err(err.to_string());
        }
        if is_reserved(&input) {
            return Err("use of reserved keyword".to_string());
        };
        Ok(input.trim().to_string())
    };
    let name = fn_ask_for("resource")?;
    let user = fn_ask_for("user")?;
    let yes_no = fn_ask_for("generated a strong password, do you want to use it? (y/n)")?;
    let password: String = if yes_no == "y" {
        password::suggest(14)
    } else {
        match rpassword::prompt_password("choose a password: ") {
            Ok(v) => v,
            Err(err) => return Err(err.to_string()),
        }
    };
    MODE.store(false, Ordering::Relaxed);
    Ok(resource::Instance {
        name,
        user,
        password,
    })
}

// Returns a tuple of (Key, Value) of a resource to update.
// E.g (resource::Key::NAME, new_name).
pub fn update_resource(i: &mut Stdin) -> Result<(resource::Key, String), String> {
    MODE.store(true, Ordering::Relaxed);
    println!("update name (n), user (u) or password (p)?");
    let mut target = String::new();
    if let Err(err) = i.read_line(&mut target) {
        return Err(err.to_string());
    }
    let key = match target.as_str() {
        "n\n" => resource::Key::Name,
        "u\n" => resource::Key::User,
        "p\n" => resource::Key::Password,
        _ => return Err("Unsupported command".to_string()),
    };

    let mut val = String::new();
    match key {
        resource::Key::Name => {
            println!("new resource name: ");
            if let Err(err) = i.read_line(&mut val) {
                return Err(err.to_string());
            }
        }
        resource::Key::User => {
            println!("new resource user: ");
            if let Err(err) = i.read_line(&mut val) {
                return Err(err.to_string());
            }
        }
        resource::Key::Password => {
            val = match rpassword::prompt_password("new password: ") {
                Ok(v) => v,
                Err(err) => return Err(err.to_string()),
            };
        }
    }
    val = val.trim().to_string();

    MODE.store(false, Ordering::Relaxed);
    Ok((key, val))
}

pub fn drop_clipboard_ctx(i: &mut Stdin) {
    MODE.store(true, Ordering::Relaxed);
    println!("Press any button to drop clipboard context");
    let mut target = String::new();
    if let Err(err) = i.read_line(&mut target) {
        println!("{err}");
    }
}

pub fn is_reserved(input: &str) -> bool {
    input == text::RESERVED_NONCE || input == text::RESERVED_RESOURCE
}
