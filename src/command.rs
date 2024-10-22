pub enum Kind {
    New,
    Get,
    Init,
    Suggest
}

impl Kind {
    pub fn from_string(s: &str) -> Option<Kind> {
        match s {
            "new"     => return Some(Kind::New),
            "get"     => return Some(Kind::Get),
            "init"    => return Some(Kind::Init),
            "suggest" => return Some(Kind::Suggest),
            _         => return None,
        }
    }
    pub fn execute(&self, t: &str) {
        match self {
            Kind::New     => println!("new command, {t}"),
            Kind::Get     => println!("get command, {t}"),
            Kind::Init    => println!("init command, {t}"),
            Kind::Suggest => println!("suggest command"),
        }
    }
}
