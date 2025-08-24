mod utils_database_sync;
mod utils_duty_verifier;
mod utils_file_transmitter;

mod archive;
mod commiter;
mod file_manager;
mod file_struct_renderer;
mod workspace_update;
mod file_viewer;

use crate::service::commands::archive::ArchiveCommand;
use crate::service::commands::commiter::CommitCommand;
use crate::service::commands::file_manager::FileOperationCommand;
use crate::service::commands::workspace_update::UpdateCommand;
use crate::service::commands::file_struct_renderer::ShowFileStructCommand;
use crate::service::commands::file_viewer::ViewCommand;
use crate::service::jam_command::CommandRegistry;
use std::collections::HashMap;
use std::sync::Arc;

pub fn registry() -> CommandRegistry {
    let mut registry : CommandRegistry = HashMap::new();

    // Core commands
    registry.insert("update", Arc::new(UpdateCommand));
    registry.insert("view", Arc::new(ViewCommand));
    registry.insert("commit", Arc::new(CommitCommand));
    registry.insert("file", Arc::new(FileOperationCommand));
    registry.insert("struct", Arc::new(ShowFileStructCommand));

    // Debug commands
    registry.insert("archive", Arc::new(ArchiveCommand));

    registry
}