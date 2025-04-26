use crate::text;

pub enum Key {
    Name,
    User,
    Password,
}

#[derive(Debug)]
pub struct Instance {
    pub name:     String,
    pub user:     String,
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
        if lines[i] == text::RESERVED_RESOURCE && lines[i+1] == name {
            return Ok(
                Instance {
                    name:     lines[i+1].to_string(),
                    user:     lines[i+2].to_string(),
                    password: lines[i+3].to_string(),
                }
            )
        }
    }
    Err("Resource not found".to_string())
}

pub struct UpdateInput {
    pub key: Key,
    pub val: String,

    pub name: String,
    pub content: String
}

pub fn update(input: UpdateInput) -> Result<String, String> {
    let mut target_idx = 1;

    match input.key {
        Key::Name     => (),
        Key::User     => target_idx += 1,
        Key::Password => target_idx += 2,
    };

    let mut lines: Vec<String> = input.content.lines().map(|s| s.to_string()).collect();
    for (i, _) in lines.iter().enumerate() {
        if lines[i] == text::RESERVED_RESOURCE && lines[i+1] == input.name {
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
        if lines[i] == "\n" { continue }
        if lines[i] == text::RESERVED_RESOURCE && lines[i+1] == name {
            name_idx = i + 1;
            user_idx = i + 2;
            pw_idx = i + 3;
            continue
        }
        if i == name_idx && name_idx != 0 {
            continue
        }
        if i == user_idx && user_idx != 0 {
            continue
        }
        if i == pw_idx && pw_idx != 0 {
            continue
        }
        result.push(v);
    }

    if name_idx == 0 {
        return Err("Resource not found".to_string())
    }

    Ok(result.join("\n").to_string())
}
