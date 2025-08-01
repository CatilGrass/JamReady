use std::env::{current_dir, set_current_dir};
use std::path::PathBuf;
use jam_ready_cli::cli_commands::setup::cli_entry;

#[tokio::main]
async fn main() {
    try_correct_current_dir();
    cli_entry().await;
}

/// 尝试纠正当前目录
fn try_correct_current_dir() {
    let current = current_dir().unwrap();
    if current.join(env!("PATH_WORKSPACE_ROOT")).exists() {
        return;
    } else {
        if let Some(found) = check_parent(current) {
            set_current_dir(found).unwrap();
        }
    }

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