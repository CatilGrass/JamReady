use std::fs;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;

pub fn move_file(from: &PathBuf, to: &PathBuf) -> Result<(), Error> {
    // 检查源文件是否存在
    if !from.exists() {
        return Err(Error::new(
            ErrorKind::NotFound,
            format!("Source file '{}' does not exist", from.display())
        ));
    }

    // 检查目标文件是否已存在
    if to.exists() {
        return Err(Error::new(
            ErrorKind::AlreadyExists,
            format!("Destination file '{}' already exists", to.display())
        ));
    }

    // 确保目标目录存在，如果不存在则创建
    if let Some(parent) = to.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    // 执行文件移动
    fs::rename(&from, &to)?;

    Ok(())
}