
use std::io;

enum Command {
    None,
    New,
    Get,
    Suggest
}

impl Command {
    fn new(s: &str) -> Command {
        match s {
            "new"     => return Command::New,
            "get"     => return Command::Get,
            "suggest" => return Command::Suggest,
            _         => return Command::None,
        }
    }
    fn execute(&self, t: &str) {
        match self {
            Command::Get     => println!("get command, {t}"),
            Command::None    => println!("no command"),
            Command::Suggest => println!("suggest command"),
            _                => println!("default"),
        }
    }
}

fn main() -> (){
    let mut input= String::new();
    let (mut command, mut target) = (String::new(), String::new());
    let std_in = io::stdin();

    match std_in.read_line(&mut input) {
        Ok(_) => (),
        Err(err) => panic!("{err}")
    }
        
    let mut split = input.split_whitespace();
    if split.clone().count() != 2 {
        panic!("expecting {{command}} {{target}}");
    }
    if let Some(cmd) = split.next() {
        command = cmd.to_string();
    }
    if let Some(trgt) = split.next() {
        target = trgt.to_string();
    }

    let cmd = Command::new(command.as_str());
    cmd.execute(target.as_str());
    println!("got {command}, {target}")
}
