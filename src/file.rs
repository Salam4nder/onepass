use std::fs::OpenOptions;
use std::io;
use std::path::PathBuf;
use std::env;

pub const DELIMITER:         &str = "|-----|";
const DEFAULT_DIR_NAME:      &str = ".onepass";
const DEFAULT_FILE_NAME:     &str = "main.txt";

pub fn file_path() -> PathBuf {
    let home_dir = match env::var("HOME") {
        Ok(dir) => dir,
        Err(err) => panic!("{}", err),
    };
    let mut path = PathBuf::from(home_dir);
    path.push(DEFAULT_DIR_NAME);
    path.push(DEFAULT_FILE_NAME);
    path
}

pub fn create() -> io::Result<std::fs::File> {
    let path = file_path();

    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(&path)?;

    Ok(file)
}

pub fn exists() -> bool {
    let path = file_path();

    if let Err(err) = OpenOptions::new().read(true).open(&path) {
        if err.kind() == io::ErrorKind::NotFound {
            return false;
        } else {
            return true;
        }
    };
    true
}
