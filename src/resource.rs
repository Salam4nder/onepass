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
