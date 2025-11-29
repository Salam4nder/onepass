use crate::text;

pub enum Key {
    Name,
    User,
    Password,
}

#[derive(Debug)]
pub struct Instance {
    pub name: String,
    pub user: String,
    pub password: String,
}

impl ToString for Instance {
    fn to_string(&self) -> String {
        let mut s = String::from("resource\n");
        s.push_str(&self.name);
        s.push_str("\n");
        s.push_str(&self.user);
        s.push_str("\n");
        s.push_str(&self.password);
        s.push_str("\n");
        s
    }
}

pub fn get(name: &str, content: &str) -> Result<Instance, String> {
    let lines: Vec<&str> = content.lines().collect();
    for (i, _) in lines.iter().enumerate() {
        if lines[i] == text::RESERVED_RESOURCE && lines[i + 1] == name {
            return Ok(Instance {
                name: lines[i + 1].to_string(),
                user: lines[i + 2].to_string(),
                password: lines[i + 3].to_string(),
            });
        }
    }
    Err("Resource not found".to_string())
}

pub struct UpdateInput {
    pub key: Key,
    pub val: String,

    pub name: String,
    pub content: String,
}

pub fn update(input: UpdateInput) -> Result<String, String> {
    let mut target_idx = 1;

    match input.key {
        Key::Name => (),
        Key::User => target_idx += 1,
        Key::Password => target_idx += 2,
    };

    let mut lines: Vec<String> = input.content.lines().map(|s| s.to_string()).collect();
    for (i, _) in lines.iter().enumerate() {
        if lines[i] == text::RESERVED_RESOURCE && lines[i + 1] == input.name {
            lines[target_idx + i] = input.val;
            break;
        }
    }

    Ok(lines.join("\n").to_string())
}

pub fn delete(name: &str, content: String) -> Result<String, String> {
    let mut name_idx = 0;
    let mut user_idx = 0;
    let mut pw_idx = 0;
    let mut result: Vec<&str> = vec![];

    let lines: Vec<&str> = content.lines().collect();
    for (i, v) in lines.iter().enumerate() {
        if lines[i] == "\n" {
            continue;
        }
        if lines[i] == text::RESERVED_RESOURCE && lines[i + 1] == name {
            name_idx = i + 1;
            user_idx = i + 2;
            pw_idx = i + 3;
            continue;
        }
        if i == name_idx && name_idx != 0 {
            continue;
        }
        if i == user_idx && user_idx != 0 {
            continue;
        }
        if i == pw_idx && pw_idx != 0 {
            continue;
        }
        result.push(v);
    }

    if name_idx == 0 {
        return Err("Resource not found".to_string());
    }

    Ok(result.join("\n").to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seed(amount: u8) -> String {
        let mut content = String::new();
        for i in 0..amount {
            content.push_str("resource\n");
            content.push_str(format!("name{}\n", i).as_str());
            content.push_str(format!("user{}\n", i).as_str());
            content.push_str(format!("password{}\n", i).as_str());
        }
        content
    }

    #[test]
    fn test_get() {
        let name = "twitter";
        let user = "user@email.com";
        let password = "password";
        let content = format!("resource\n{}\n{}\n{}\n", name, user, password);

        let resource = get("twitter", &content).expect("getting");
        assert_eq!(resource.name, name);
        assert_eq!(resource.user, user);
        assert_eq!(resource.password, password);

        let err_result = get("does-not-exist", &content);
        assert_eq!(err_result.unwrap_err(), "Resource not found");
    }

    #[test]
    fn test_get_many() {
        let mut long_content = String::new();
        let target_name = "website";
        let target_user = "website@user.com";
        let target_password = "websitepass";
        for i in 1..100 {
            if i == 50 {
                long_content.push_str("resource\n");
                long_content.push_str(format!("{}\n", target_name).as_str());
                long_content.push_str(format!("{}\n", target_user).as_str());
                long_content.push_str(format!("{}\n", target_password).as_str());
                continue;
            }
            long_content.push_str("resource\n");
            long_content.push_str(format!("name{}\n", i).as_str());
            long_content.push_str(format!("user{}\n", i).as_str());
            long_content.push_str(format!("password{}\n", i).as_str());
        }
        let long_result = get("website", &long_content).expect("getting long result");
        assert_eq!(long_result.name, target_name);
        assert_eq!(long_result.user, target_user);
        assert_eq!(long_result.password, target_password);
    }

    #[test]
    #[should_panic]
    fn test_get_panics_on_unfinished_resource() {
        let content = format!("resource\n{}\n{}\n", "name", "password");
        get("twitter", &content).expect("getting");
    }

    #[test]
    fn test_update_name() {
        let new_value = "website";
        let content = seed(3);
        let updated = update(UpdateInput {
            key: Key::Name,
            val: String::from(new_value),
            name: String::from("name2"),
            content,
        })
        .expect("updating");
        let lines: Vec<&str> = updated.lines().collect();
        assert_eq!(lines[9], new_value);
        assert_eq!(lines[10], "user2");
        assert_eq!(lines[11], "password2");
    }

    #[test]
    fn test_update_user() {
        let new_value = "new_user";
        let content = seed(3);
        let updated = update(UpdateInput {
            key: Key::User,
            val: String::from(new_value),
            name: String::from("name0"),
            content,
        })
        .expect("updating");
        let lines: Vec<&str> = updated.lines().collect();
        assert_eq!(lines[1], "name0");
        assert_eq!(lines[2], new_value);
        assert_eq!(lines[3], "password0");
    }

    #[test]
    fn test_update_password() {
        let new_value = "new_password";
        let content = seed(3);
        let updated = update(UpdateInput {
            key: Key::Password,
            val: String::from(new_value),
            name: String::from("name1"),
            content,
        })
        .expect("updating");
        let lines: Vec<&str> = updated.lines().collect();
        assert_eq!(lines[5], "name1");
        assert_eq!(lines[6], "user1");
        assert_eq!(lines[7], new_value);
    }

    #[test]
    fn test_delete() {
        let mut content = seed(3);
        let deleted = delete("name0", content).expect("deleting");
        let lines: Vec<&str> = deleted.lines().collect();
        assert_eq!(lines.len(), 8);
        assert!(!lines.contains(&"name0"));
        assert!(!lines.contains(&"user0"));
        assert!(!lines.contains(&"password0"));

        content = seed(3);
        let not_found = delete("non", content);
        assert_eq!(not_found.unwrap_err(), "Resource not found");
    }
}
