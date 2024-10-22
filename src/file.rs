use std::fs::OpenOptions;
use std::io;
use std::path::PathBuf;
use std::env;

const DEFAULT_DIR_NAME:  &str = ".onepass";
const DEFAULT_FILE_NAME: &str = "main.txt";

fn open() -> io::Result<std::fs::File> {
    let home_dir = match env::var("HOME") {
        Ok(dir) => dir,
        Err(err) => return Err(io::Error::new(io::ErrorKind::NotFound, err)),
    };

    let mut path = PathBuf::from(home_dir);
    path.push(DEFAULT_DIR_NAME);
    path.push(DEFAULT_FILE_NAME);

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .write(true)
        .open(&path)?;

    Ok(file)
}

pub fn root_exists() -> bool {
    false
}

pub fn create_root() {
}

