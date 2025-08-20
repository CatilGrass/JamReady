use std::fs;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;

pub fn move_file(from: &PathBuf, to: &PathBuf) -> Result<(), Error> {
    // Check if source file exists
    if !from.exists() {
        return Err(Error::new(
            ErrorKind::NotFound,
            format!("Source file '{}' does not exist", from.display())
        ));
    }

    // Check if destination file already exists
    if to.exists() {
        return Err(Error::new(
            ErrorKind::AlreadyExists,
            format!("Destination file '{}' already exists", to.display())
        ));
    }

    // Ensure destination directory exists, create if not
    if let Some(parent) = to.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    // Perform file move operation
    fs::rename(&from, &to)?;

    Ok(())
}

pub fn copy_file(from: &PathBuf, to: &PathBuf) -> Result<(), Error> {
    // Check if source exists
    if !from.exists() {
        return Err(Error::new(
            ErrorKind::NotFound,
            format!("Source file '{}' does not exist", from.display()),
        ));
    }

    // Ensure source is a file
    if !from.is_file() {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!("'{}' is not a file", from.display()),
        ));
    }

    // Create destination directory if needed
    if let Some(parent) = to.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    // Perform copy with overwrite
    fs::copy(from, to)?;

    Ok(())
}