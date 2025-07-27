mod database_sync;
mod duty_verifier;
mod file_transmitter;

mod command_archive;
mod command_commit;
mod command_file;
mod command_show_struct;
mod command_update;
mod command_view;

use crate::service::commands::command_archive::ArchiveCommand;
use crate::service::commands::command_commit::CommitCommand;
use crate::service::commands::command_file::FileOperationCommand;
use crate::service::commands::command_update::UpdateCommand;
use crate::service::commands::command_show_struct::ShowFileStructCommand;
use crate::service::commands::command_view::ViewCommand;
use crate::service::jam_command::CommandRegistry;
use std::collections::HashMap;
use std::sync::Arc;

/// 获得命令注册表
pub fn registry() -> CommandRegistry {
    let mut registry : CommandRegistry = HashMap::new();

    // 注册
    registry.insert("update", Arc::new(UpdateCommand));
    registry.insert("view", Arc::new(ViewCommand));
    registry.insert("commit", Arc::new(CommitCommand));
    registry.insert("file", Arc::new(FileOperationCommand));
    registry.insert("struct", Arc::new(ShowFileStructCommand));

    // 调试指令
    registry.insert("archive", Arc::new(ArchiveCommand));

    registry
}