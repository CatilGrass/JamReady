use std::path::PathBuf;
use std::io::Error;
use std::process::Command;

/// 打开本地资源管理器
pub fn open_in_explorer(path: PathBuf) -> Result<(), Error> {
    let absolute_path = if path.is_relative() {
        std::env::current_dir()?.join(path)
    } else {
        path
    };

    let absolute_path = absolute_path.canonicalize().unwrap_or(absolute_path);

    if !absolute_path.exists() {
        return Err(Error::new(
            std::io::ErrorKind::NotFound,
            format!("Path does not exist: {}", absolute_path.display())
        ));
    }

    if cfg!(target_os = "windows") {
        if absolute_path.is_file() {
            absolute_path.parent()
                .ok_or_else(|| Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "File has no parent directory"
                ))?;

            Command::new("explorer")
                .args(&["/select,", absolute_path.to_str().unwrap()])
                .spawn()?;
        } else {
            Command::new("explorer")
                .arg(absolute_path.to_str().unwrap())
                .spawn()?;
        }
    } else if cfg!(target_os = "macos") {
        if absolute_path.is_file() {
            Command::new("open")
                .args(&["-R", absolute_path.to_str().unwrap()])
                .spawn()?;
        } else {
            Command::new("open")
                .arg(absolute_path.to_str().unwrap())
                .spawn()?;
        }
    } else {
        Command::new("xdg-open")
            .arg(absolute_path.to_str().unwrap())
            .spawn()?;
    }

    Ok(())
}