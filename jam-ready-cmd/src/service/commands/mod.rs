mod database_sync;
mod duty_verifier;
mod file_transmitter;

mod command_clean;
mod command_commit;
mod command_file;
mod command_show_struct;
mod command_sync;
mod command_view;

use crate::service::commands::command_clean::CleanCommand;
use crate::service::commands::command_commit::CommitCommand;
use crate::service::commands::command_file::FileOperationCommand;
use crate::service::commands::command_sync::SyncCommand;
use crate::service::commands::command_show_struct::ShowFileStructCommand;
use crate::service::commands::command_view::ViewCommand;
use crate::service::jam_command::CommandRegistry;
use std::collections::HashMap;
use std::sync::Arc;

/// 获得命令注册表
pub fn registry() -> CommandRegistry {
    let mut registry : CommandRegistry = HashMap::new();

    // 注册
    registry.insert("sync", Arc::new(SyncCommand));
    registry.insert("view", Arc::new(ViewCommand));
    registry.insert("commit", Arc::new(CommitCommand));
    registry.insert("file", Arc::new(FileOperationCommand));
    registry.insert("struct", Arc::new(ShowFileStructCommand));

    // 调试指令
    registry.insert("clean", Arc::new(CleanCommand));

    registry
}