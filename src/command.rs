pub enum Command {
    New,
    Get,
    Suggest
}

impl Command {
    pub fn from_string(s: &str) -> Option<Command> {
        match s {
            "new"     => return Some(Command::New),
            "get"     => return Some(Command::Get),
            "suggest" => return Some(Command::Suggest),
            _         => return None,
        }
    }
    pub fn execute(&self, t: &str) {
        match self {
            Command::New     => println!("new command, {t}"),
            Command::Get     => println!("get command, {t}"),
            Command::Suggest => println!("suggest command"),
        }
    }
}
