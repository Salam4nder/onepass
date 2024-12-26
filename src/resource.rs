pub const NAME:     &str = "RESOURCE_NAME";
pub const USER:     &str = "RESOURCE_USER";
pub const PASSWORD: &str = "RESOURCE_PASSWORD";

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
