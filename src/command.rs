pub enum Kind {
    New,
    Get,
    Init,
    List,
    Purge,
    Suggest,
}

impl Kind {
    pub fn from_string(s: &str) -> Option<Kind> {
        match s {
            "new"     => return Some(Kind::New),
            "get"     => return Some(Kind::Get),
            "init"    => return Some(Kind::Init),
            "list"    => return Some(Kind::List),
            "purge"   => return Some(Kind::Purge),
            "suggest" => return Some(Kind::Suggest),
            _         => return None,
        }
    }
}
