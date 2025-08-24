pub mod data;
pub mod service;
pub mod cli_commands;
pub mod help;
pub mod linker;

use std::env::current_dir;
use std::env::set_current_dir;
use std::io::ErrorKind::NotFound;
use std::path::PathBuf;

/// Attempt to correct the current directory
pub fn try_correct_current_dir() -> Result<(), std::io::Error> {
    let current = current_dir()?;
    return if current.join(env!("PATH_WORKSPACE_ROOT")).exists() {
        Ok(())
    } else {
        if let Some(found) = check_parent(current) {
            set_current_dir(found)?;
        }
        Err(std::io::Error::new(NotFound, "Workspace directory does not exist."))
    };
    fn check_parent(current: PathBuf) -> Option<PathBuf> {
        if current.join(env!("PATH_WORKSPACE_ROOT")).exists() {
            Some(current)
        } else {
            if let Some(parent) = current.parent() {
                check_parent(PathBuf::from(parent))
            } else {
                None
            }
        }
    }
}