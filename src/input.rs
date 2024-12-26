use crate::resource;
use crate::password;

use std::sync::atomic::{Ordering, AtomicBool};
use std::io::Stdin;

pub const RESERVED_NONCE:    &str = "nonce";
pub const RESERVED_RESOURCE: &str = "resource";

pub static MODE: AtomicBool = AtomicBool::new(false);

pub fn master_password() -> Result<String, String> {
    MODE.store(true, Ordering::Relaxed);
    let input = match rpassword::prompt_password("please input your master password: ") {
        Ok(v) => v,
        Err(err) => return Err(err.to_string())
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
            return Err(err.to_string())
        }
        if is_reserved(&input) {
            return Err("use of reserved keyword".to_string());
        };
        Ok(input.trim().to_string())
    };
    let password: String;
    let name = fn_ask_for("resource")?;
    let user = fn_ask_for("user")?;
    let yes_no = fn_ask_for("generated a strong password, do you want to use it? (y/n)")?;
    if yes_no == "y" {
        password = password::suggest(14);
    } else {
        password = match rpassword::prompt_password("choose a password: ") {
            Ok(v) => v,
            Err(err) => return Err(err.to_string())
        };
    }
    MODE.store(false, Ordering::Relaxed);
    Ok(resource::Instance{
        name,
        user,
        password,
    })
}

// Returns a tuple of (Key, Value) of a resource to update.
// E.g (resource::NAME, new_name).
pub fn update_resource(i: &mut Stdin) -> Result<(String, String), String> {
    MODE.store(true, Ordering::Relaxed);
    println!("update name (n), user (u) or password (p)?");
    let mut input = String::new();
    if let Err(err) = i.read_line(&mut input) { 
        return Err(err.to_string())
    }
    let key = match input.as_str() {
        "n\n" => resource::NAME,
        "u\n" => resource::USER,
        "p\n" => resource::PASSWORD,
        _     => { 
                    println!("unsupported command, aborting...");
                    return Err("cancelled by user".to_string())
                 },
    };

    let val: String;
    match key {
        resource::NAME     => {
            println!("new resource name: ");
            let mut tmp = String::new();
            if let Err(err) = i.read_line(&mut tmp) { 
                return Err(err.to_string())
            }
            // TODO(kg): Check for existing resource.
            val = tmp;
        },
        resource::USER     => {
            println!("new resource user: ");
            let mut tmp = String::new();
            if let Err(err) = i.read_line(&mut tmp) { 
                return Err(err.to_string())
            }
            val = tmp;
        },
        resource::PASSWORD => {
            val = match rpassword::prompt_password("new password: ") {
                Ok(v) => v,
                Err(err) => return Err(err.to_string())
            };
        },
        _                  => {
            return Err("key malformed".to_string())
        },
    }

    MODE.store(false, Ordering::Relaxed);
    Ok((String::from(key), val))
}

pub fn is_reserved(input: &str) -> bool {
    input == RESERVED_NONCE || input == RESERVED_RESOURCE
}
