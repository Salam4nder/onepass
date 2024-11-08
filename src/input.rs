use std::sync::atomic::{Ordering, AtomicBool};
use std::io::Stdin;
use crate::resource;

pub const RESERVED_NONCE:    &str = "nonce";
pub const RESERVED_RESOURCE: &str = "resource";

pub static MODE: AtomicBool = AtomicBool::new(false);

pub fn master_password() -> Result<String, String> {
    MODE.store(true, Ordering::Relaxed);
    let input = rpassword::prompt_password("please input your master password: ").expect("reading");
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
        i.read_line(&mut input).expect("reading");
        if is_reserved(&input) {
            return Err("use of reserved keyword".to_string());
        };
        Ok(input)
    };
    let name     = fn_ask_for("resource: ")?;
    let user     = fn_ask_for("user: ")?;
    let password = fn_ask_for("password: ")?;
    MODE.store(false, Ordering::Relaxed);
    Ok(resource::Instance{
        name,
        user,
        password,
    })
}

pub fn is_reserved(input: &str) -> bool {
    input == RESERVED_NONCE || input == RESERVED_RESOURCE
}
