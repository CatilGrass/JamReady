use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;

pub fn md5_digest(file_path: PathBuf) -> std::io::Result<String> {
    let file = File::open(file_path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = md5::Context::new();

    let mut buffer = [0; 1024];
    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.consume(&buffer[..count]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}