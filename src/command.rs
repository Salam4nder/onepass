use std::io::Stdin;
use std::os::unix::fs::MetadataExt;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::file;
use crate::input;
use crate::password;
use crate::resource;
use crate::text;

use arboard::Clipboard;

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
            "new" => return Some(Kind::New),
            "get" => return Some(Kind::Get),
            "del" => return Some(Kind::Del),
            "help" => return Some(Kind::Help),
            "list" => return Some(Kind::List),
            "purge" => return Some(Kind::Purge),
            "update" => return Some(Kind::Update),
            "suggest" => return Some(Kind::Suggest),
            _ => return None,
        }
    }
}

/// Create a new resource and append it to the file.
pub fn new(custom_path: Option<&str>, stdin: &mut Stdin) -> Result<(), String> {
    if !file::exists(custom_path) {
        if let Err(err) = file::create(custom_path) {
            return Err(err.to_string());
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
        Err(err) => return Err(err.to_string()),
    };

    let mut content = String::new();
    if metadata.size() > 0 {
        content = file::decrypt(custom_path, password)?;
        let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        for line in &lines {
            if line.trim() == resource.name.trim() {
                return Err("Resource already exists".to_string());
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

pub fn get(custom_path: Option<&str>, args: Vec<String>) -> Result<Clipboard, String> {
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
    let mut ctx = match Clipboard::new() {
        Ok(v) => v,
        Err(err) => return Err(err.to_string()),
    };
    if let Err(_) = ctx.set_text(got.password.to_owned()) {
        println!("Password: {}", got.password);
        println!("Don't forget to clear your terminal");
    } else {
        println!("Password copied to clipboard");
    };

    DONE.store(true, Ordering::Relaxed);
    Ok(ctx)
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
    let content = file::decrypt(custom_path, &password)?;

    let mut result: Vec<String> = vec![];
    let lines: Vec<&str> = content.lines().collect();
    for (i, v) in lines.iter().enumerate() {
        if *v == text::RESERVED_RESOURCE {
            result.push(lines[i + 1].to_string());
        }
    }

    Ok(result)
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
    password::suggest(16)
}

pub fn update(
    custom_path: Option<&str>,
    args: Vec<String>,
    stdin: &mut Stdin,
) -> Result<(), String> {
    if !file::exists(custom_path) {
        return Err(text::MSG_NO_RESOURCES.to_string());
    }

    if args.len() < 3 {
        return Err(text::MSG_COMMAND_UPDATE.to_string());
    }

    let name = args[2].clone();
    if input::is_reserved(&name) {
        return Err("use of reserved keyword".to_string());
    }
    let password = input::master_password()?;
    let (key, val) = input::update_resource(stdin)?;

    update_resource(custom_path, &password, name, key, val)?;

    DONE.store(true, Ordering::Relaxed);
    Ok(())
}

fn update_resource(
    custom_path: Option<&str>,
    password: &str,
    name: String,
    key: resource::Key,
    val: String,
) -> Result<(), String> {
    let content = file::decrypt(custom_path, &password)?;

    resource::get(&name, &content)?;

    let updated = resource::update(resource::UpdateInput {
        key,
        val,
        name,
        content,
    })?;

    file::encrypt(custom_path, &password, updated)?;

    Ok(())
}

pub fn del(custom_path: Option<&str>, args: Vec<String>) -> Result<(), String> {
    if args.len() < 3 {
        return Err(text::MSG_COMMAND_DEL.to_string());
    }

    if !file::exists(custom_path) {
        return Err(text::MSG_NO_RESOURCES.to_string());
    }

    let password = input::master_password()?;
    let name = &args[2];
    if input::is_reserved(name) {
        return Err("Keyword is reserved".to_string());
    }

    delete_resource(custom_path, &password, name)?;

    DONE.store(true, Ordering::Relaxed);
    Ok(())
}

fn delete_resource(custom_path: Option<&str>, password: &str, name: &str) -> Result<(), String> {
    let content = file::decrypt(custom_path, &password)?;
    let deleted = resource::delete(name, content)?;
    file::encrypt(custom_path, &password, deleted)?;
    Ok(())
}

pub fn help(args: Vec<String>) -> String {
    if args.len() != 3 {
        return text::MSG_HELP.to_string();
    };

    if let Some(command) = Kind::from_string(&args[2]) {
        match command {
            Kind::Get => return text::MSG_COMMAND_GET.to_string(),
            Kind::Del => return text::MSG_COMMAND_DEL.to_string(),
            Kind::Update => return text::MSG_COMMAND_UPDATE.to_string(),
            _ => return text::MSG_HELP.to_string(),
        }
    } else {
        return text::MSG_HELP.to_string();
    }
}

#[cfg(test)]
mod tests {
    use core::panic;

    use super::*;
    use uuid::Uuid;

    struct Cleanup {
        file_name: String,
    }

    impl Cleanup {
        fn path(&self) -> String {
            format!("{}/{}.txt", file::DEFAULT_DIR_NAME, self.file_name)
                .as_str()
                .to_string()
        }
    }

    impl Drop for Cleanup {
        fn drop(&mut self) {
            let file_path = format!("{}/{}.txt", file::DEFAULT_DIR_NAME, self.file_name);
            file::purge(Some(&file_path)).expect("cleaning up");
        }
    }

    fn seed(path: &str, amount: u8) -> String {
        let password = password::suggest(16);

        for i in 0..amount {
            if let Err(err) = new_resource(
                Some(path),
                &password,
                resource::Instance {
                    name: String::from(format!("name{}", i)),
                    user: String::from(format!("user{}", i)),
                    password: String::from(format!("password{}", i)),
                },
            ) {
                panic!("seeding: {}", err)
            }
        }

        password.to_string()
    }

    fn count_lines(path: &str, password: &str) -> Result<usize, String> {
        let mut count: usize = 0;
        let content = file::decrypt(Some(path), &password)?;
        for _ in content.lines().into_iter() {
            count += 1;
        }
        Ok(count)
    }

    #[test]
    fn test_get_resource() {
        let id = Uuid::new_v4();
        let cleanup = Cleanup {
            file_name: id.to_string(),
        };
        let t_path = &cleanup.path();
        file::create(Some(t_path)).expect("creating");

        let resource_name = "name3";
        let resource_user = "user3";
        let resource_password = "password3";
        let master_password = seed(t_path, 5);
        let got =
            get_resource(Some(&t_path), &master_password, resource_name).expect("getting resource");

        assert_eq!(resource_name, got.name);
        assert_eq!(resource_user, got.user);
        assert_eq!(resource_password, got.password);
    }

    #[test]
    fn test_list_resources() {
        let id = Uuid::new_v4();
        let cleanup = Cleanup {
            file_name: id.to_string(),
        };
        let t_path = &cleanup.path();
        file::create(Some(t_path)).expect("creating");

        let master_password = seed(t_path, 5);
        let list = list_resources(Some(t_path), &master_password).expect("listing");
        assert_eq!(5, list.len());
    }

    #[test]
    fn test_update_resource() {
        let id = Uuid::new_v4();
        let cleanup = Cleanup {
            file_name: id.to_string(),
        };
        let t_path = &cleanup.path();
        file::create(Some(t_path)).expect("creating");
        let master_password = seed(t_path, 5);

        let new_name = "new_name";
        let new_user = "new_user";
        let new_password = "new_password";

        update_resource(
            Some(t_path),
            &master_password,
            "name0".to_string(),
            resource::Key::Name,
            new_name.to_string(),
        )
        .expect("updating name");

        let list = list_resources(Some(t_path), &master_password).expect("listing");
        assert_eq!(5, list.len());
        assert_eq!(
            5 * 4,
            count_lines(t_path, &master_password).expect("counting lintes")
        );

        let got = get_resource(Some(t_path), &master_password, new_name).expect("getting name");
        assert_eq!(new_name, got.name);
        assert_eq!("user0", got.user);
        assert_eq!("password0", got.password);

        update_resource(
            Some(t_path),
            &master_password,
            "name1".to_string(),
            resource::Key::User,
            new_user.to_string(),
        )
        .expect("updating user");

        let list = list_resources(Some(t_path), &master_password).expect("listing");
        assert_eq!(5, list.len());
        assert_eq!(
            5 * 4,
            count_lines(t_path, &master_password).expect("counting lintes")
        );

        let got = get_resource(Some(t_path), &master_password, "name1").expect("getting name");
        assert_eq!("name1", got.name);
        assert_eq!(new_user, got.user);
        assert_eq!("password1", got.password);

        update_resource(
            Some(t_path),
            &master_password,
            "name2".to_string(),
            resource::Key::Password,
            new_password.to_string(),
        )
        .expect("updating password");

        let list = list_resources(Some(t_path), &master_password).expect("listing");
        assert_eq!(5, list.len());
        assert_eq!(
            5 * 4,
            count_lines(t_path, &master_password).expect("counting lintes")
        );

        let got = get_resource(Some(t_path), &master_password, "name2").expect("getting password");
        assert_eq!("name2", got.name);
        assert_eq!("user2", got.user);
        assert_eq!(new_password, got.password);
    }

    #[test]
    fn test_delete_resource() {
        let id = Uuid::new_v4();
        let cleanup = Cleanup {
            file_name: id.to_string(),
        };
        let t_path = &cleanup.path();
        file::create(Some(t_path)).expect("creating");

        let master_password = seed(t_path, 5);

        delete_resource(Some(t_path), &master_password, "name1").expect("deleting");

        let list = list_resources(Some(t_path), &master_password).expect("listing");
        assert_eq!(4, list.len());
    }
}
