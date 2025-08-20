use crate::data::client_result::ClientResult;
use crate::data::database::Database;
use crate::data::local_file_map::LocalFileMap;
use crate::data::member::Member;
use crate::service::commands::database_sync::{sync_local, sync_local_with_progress, sync_remote_with_progress};
use crate::service::jam_command::Command;
use async_trait::async_trait;
use jam_ready::entry_mutex_async;
use jam_ready::utils::file_operation::move_file;
use jam_ready::utils::local_archive::LocalArchive;
use jam_ready::utils::text_process::process_path_text;
use std::env::current_dir;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::{fs, io};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

pub struct UpdateCommand;

#[async_trait]
impl Command for UpdateCommand {

    async fn local(&self, stream: &mut TcpStream, _args: Vec<&str>) -> Option<ClientResult> {

        let debug = ClientResult::debug_mode().await;
        let mut command_result = ClientResult::result().await;

        command_result.log("Step1: Sync Database.");
        // Sync database
        if debug {
            sync_local(stream).await;
        } else {
            sync_local_with_progress(stream).await;
        }
        command_result.log("Ok");

        // Sync local file structure with remote
        command_result.log("Step2: Sync File Struct.");
        Self::sync_file_struct().await;
        command_result.log("Ok");

        // Remove all empty directories locally
        command_result.log("Step3: Remove Empty Directories.");
        if let Ok(current) = current_dir() {
            let _ = Self::remove_unused_directory(current);
        }
        command_result.log("Ok");

        Some(command_result)
    }

    async fn remote(
        &self,
        stream: &mut TcpStream, _args: Vec<&str>,
        (_uuid, _member): (String, &Member), database: Arc<Mutex<Database>>) {

        // Sync database
        entry_mutex_async!(database, |guard| {
            sync_remote_with_progress(stream, guard).await;
        });
    }
}

impl UpdateCommand {

    /// Sync local file structure with remote
    async fn sync_file_struct() {

        // Local files and database
        let database = Database::read().await;
        let mut local = LocalFileMap::read().await;

        // Track successful UUIDs
        let mut success_uuid = Vec::new();

        // Compare all local files
        for (uuid, local_file) in &local.file_paths {

            // Find VirtualFile and compare remote vs local paths
            if let Some(file) = database.file_with_uuid(uuid.clone()) {

                // Paths match, skip
                if file.path() == local_file.local_path { continue; }

                // Check if local path exists
                if let Some(from) = local.search_to_path(&database, uuid.clone()) {

                    // Check if we can get corresponding server-side local path
                    if let Some(to) = file.client_path() {

                        let from_str = process_path_text(from.display().to_string());
                        let to_str = process_path_text(to.display().to_string());

                        // Process file move
                        match move_file(&from, &to) {
                            Ok(_) => {
                                println!("Ok: Move {} to {}", from_str, to_str);
                                success_uuid.push(uuid.clone());
                            }
                            Err(err) => {
                                eprintln!("Err: Move {} to {} failed: {}", from_str, to_str, err);
                            }
                        }
                    }
                }
            }
        }

        // Process successful UUIDs and update their local mappings
        for uuid in success_uuid {

            // Find path-to-UUID mapping
            let mut path = None;
            for (local_path, local_uuid) in &local.file_uuids {
                if uuid.trim() == local_uuid.trim() {
                    path = Some(local_path.clone());
                    break;
                }
            }

            // Get file
            let file = database.file_with_uuid(uuid.clone());

            // Rebuild path-to-UUID mapping
            if let Some(path) = path {
                if let Some(file) = file.clone() {
                    local.file_uuids.remove(&path);
                    local.file_uuids.insert(file.path(), uuid.clone());
                }
            }

            // Update UUID-to-file mapping
            let local_file = local.file_paths.get_mut(&uuid);
            if let Some(local_file) = local_file {
                if let Some(file) = file {
                    local_file.local_path = file.path();
                }
            }
        }

        LocalFileMap::update(&local).await;
    }

    /// Remove all empty directories
    pub fn remove_unused_directory(dir_path: PathBuf) -> io::Result<()> {
        if !dir_path.exists() {
            return Err(Error::new(
                ErrorKind::NotFound,
                "Directory does not exist",
            ));
        }

        if !dir_path.is_dir() {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Path is not a directory",
            ));
        }

        fn remove_empty_dirs(path: &Path) -> io::Result<bool> {
            if !path.is_dir() {
                return Ok(false);
            }

            let mut has_entries = false;

            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let entry_path = entry.path();

                if entry_path == current_dir()?.join(env!("PATH_WORKSPACE_ROOT")) {
                    continue;
                }

                if entry_path.is_dir() {
                    let has_sub_entries = remove_empty_dirs(&entry_path)?;
                    has_entries = has_entries || has_sub_entries;
                } else {
                    has_entries = true;
                }
            }

            if !has_entries {
                fs::remove_dir(path)?;
                println!("Ok: Removed empty directory: {}", path.display());
                Ok(false)
            } else {
                Ok(true)
            }
        }

        remove_empty_dirs(&dir_path)?;
        Ok(())
    }
}