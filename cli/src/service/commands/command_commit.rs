use std::sync::Arc;
use std::time::Duration;
use crate::data::database::Database;
use crate::data::member::Member;
use crate::data::workspace::Workspace;
use crate::service::commands::database_sync::{sync_local, sync_remote};
use crate::service::jam_command::Command;
use async_trait::async_trait;
use colored::Colorize;
use log::{info};
use jam_ready::utils::local_archive::LocalArchive;
use tokio::net::TcpStream;
use tokio::select;
use tokio::sync::Mutex;
use tokio::time::sleep;
use uuid::Uuid;
use jam_ready::entry_mutex_async;
use jam_ready::utils::file_digest::md5_digest;
use jam_ready::utils::text_process::process_path_text;
use crate::data::client_result::ClientResult;
use crate::data::local_file_map::{LocalFile, LocalFileMap};
use crate::service::commands::file_transmitter::{read_file, send_file};
use crate::service::messages::ClientMessage::{Done, Text, Unknown};
use crate::service::messages::{ClientMessage, ServerMessage};
use crate::service::messages::ServerMessage::{Deny, Pass};
use crate::service::service_utils::{read_msg, send_msg};

pub struct CommitCommand;

#[async_trait]
impl Command for CommitCommand {
    async fn local(&self, stream: &mut TcpStream, _args: Vec<&str>) -> Option<ClientResult> {

        let mut command_result = ClientResult::result().await;

        // Sync database
        sync_local(stream).await;
        let database = Database::read().await;
        let mut local = LocalFileMap::read().await;

        // Counters
        let mut all_count = 0;
        let mut success_count = 0;

        // File lists
        let mut success_files = Vec::new();
        let mut failed_files = Vec::new();

        // Load workspace
        let workspace = Workspace::read().await;
        if let Some(client) = workspace.client {

            // Find files locked by current member in database
            for file in database.files() {

                // Check if file is locked by current member
                let is_locked_by_me = file.get_locker_owner_uuid()
                    .map(|owner_uuid| owner_uuid.trim() == client.uuid.trim())
                    .unwrap_or(false);

                if !is_locked_by_me {
                    continue;
                }

                command_result.log(format!("Checking {}", format!("\"{}\"", &file.path()).cyan()).as_str());

                // Get local file path
                let client_path = match local.file_to_path(&database, file) {
                    Some(path) if path.exists() => path,
                    _ => {
                        command_result.warn("File Not Found.");
                        continue
                    },
                };

                command_result.log("File Found, Checking Completed");

                // Calculate current file MD5
                let current_digest = match md5_digest(client_path.clone()) {
                    Ok(md5) => md5,
                    Err(_) => continue,
                };

                // Check if the local file is marked as completed
                let Some(completed_commit) = ({
                    if let Some(local_file) = local.search_to_local(&database, file.path()) {
                        if local_file.completed && local_file.completed_digest == current_digest {
                            Some(local_file.completed_commit.clone())
                        } else { None }
                    } else {
                        Some("First commit".to_string())
                    }
                }) else {
                    command_result.warn("File Not Completed!");
                    continue;
                };

                command_result.log("File Completed, Checking Modified");

                // Check if version allows commit
                let local_file = local.file_paths.get(
                    &database.uuid_of_path(file.path()).unwrap_or_default()
                );

                let modified = match (local_file, file.version()) {

                    // Allow commit if remote version is 0 (new file)
                    (_, 0) => true,

                    // File modified if local record exists, versions match but MD5 differs
                    (Some(local_file), _) if local_file.local_version == file.version()
                        && local_file.local_digest != current_digest => true,

                    // Other cases don't allow commit
                    _ => false,
                };

                if !modified {
                    command_result.warn("File Not Modified.");
                    continue;
                } else {
                    command_result.log("File Modified, Start Uploading ...");
                }

                all_count += 1;
                let record_file_path = process_path_text(client_path.display().to_string());

                // Request upload permission from server
                send_msg(stream, &Text(format!("{}|{}", file.path(), completed_commit))).await;
                match read_msg::<ServerMessage>(stream).await {
                    Pass => {

                        // Upload file
                        if send_file(stream, client_path.clone()).await.is_ok() {
                            success_count += 1;
                            success_files.push(record_file_path.clone());

                            // Update local mapping
                            if let Some(uuid) = database.uuid_of_path(file.path()) {
                                let new_version = file.version() + 1;
                                if let Some(local_file) = local.file_paths.get_mut(&uuid) {
                                    local_file.local_version = new_version;
                                    local_file.local_digest = current_digest.clone();
                                    local_file.completed = false;
                                    local_file.completed_digest = String::new();
                                    local_file.completed_commit = String::new();
                                } else {
                                    local.file_paths.insert(uuid.clone(), LocalFile {
                                        local_path: file.path().to_string(),
                                        local_version: new_version,
                                        local_digest: current_digest,
                                        completed: false,
                                        completed_digest: String::new(),
                                        completed_commit: String::new(),
                                    });
                                    local.file_uuids.insert(file.path().to_string(), uuid);
                                }
                                LocalFileMap::update(&local).await;
                            }
                        } else {
                            failed_files.push(record_file_path);
                        }
                    }
                    _ => {
                        failed_files.push(record_file_path);
                    }
                }
            }

            // Print commit results
            if all_count == 0 {
                command_result.err("No File Committed.");
            } else if success_count == all_count {
                command_result.log(format!("Committed {} file(s).", all_count).as_str());
            } else {
                command_result.warn(format!("Committed {} file(s).", all_count).as_str());
            }

            // Print success and failed file lists
            if !success_files.is_empty() {
                command_result.log("Success file(s):");
                for file in success_files {
                    command_result.log(format!("{}", file).as_str());
                }
            }

            if !failed_files.is_empty() {
                command_result.log("Failed file(s):");
                for file in failed_files {
                    command_result.log(format!("{}", file).as_str());
                }
            }

            // Send completion message
            send_msg(stream, &Done).await;
            return Some(command_result)
        }
        None
    }

    async fn remote(
        &self,
        stream: &mut TcpStream,
        _args: Vec<&str>,
        (uuid, _member): (String, &Member),
        database: Arc<Mutex<Database>>
    ) {
        let mut changed = false;

        // Sync database
        entry_mutex_async!(database, |guard| {
            sync_remote(stream, guard).await;
        });

        loop {
            select! {
                // 60 seconds timeout
                _ = sleep(Duration::from_secs(60)) => break,

                // Process messages
                msg = read_msg::<ClientMessage>(stream) => {
                    if msg == Unknown || msg == Done {
                        break;
                    }

                    if let Text(msg) = msg {

                        let split = msg.split("|").into_iter().collect::<Vec<&str>>();
                        let path = split[0];
                        let commit_description = split[1];

                        let pack;

                        // Find file
                        entry_mutex_async!(database, |guard| {
                            let Some(file) = guard.file_mut(path.to_string()) else {
                                send_msg(stream, &Deny("Virtual file not found.".to_string())).await;
                                continue;
                            };

                            // Check lock status
                            let is_locked_by_client = file.get_locker_owner_uuid()
                                .map(|owner| owner.trim() == uuid.trim())
                                .unwrap_or(false);

                            if !is_locked_by_client {
                                send_msg(stream, &Deny("Lock mismatch".to_string())).await;
                                continue;
                            }

                            // Generate new file UUID
                            let real_file_uuid = Uuid::new_v4().to_string();

                            // Get server path
                            if let Some(path) = file.server_path_temp(real_file_uuid.clone()) {
                                send_msg(stream, &Pass).await;
                                pack = Some((path.clone(), real_file_uuid.clone()));
                            } else {
                                send_msg(stream, &Deny("Cannot get server file path.".to_string())).await;
                                continue;
                            }
                        });

                        if let Some((real_path, real_file_uuid)) = pack {

                            // Receive file
                            if read_file(stream, real_path.clone()).await.is_ok() {

                                entry_mutex_async!(database, |guard| {
                                    let Some(file) = guard.file_mut(path.to_string()) else {
                                        continue;
                                    };

                                    // Update file
                                    file.update(real_file_uuid, commit_description.to_string());
                                    info!("Update file {}: \"{}\"", file.path(), commit_description);

                                    // Release lock if not long-term
                                    if !file.is_longer_lock_unchecked() {
                                        file.throw_locker();
                                    }
                                });

                                changed = true;
                                continue;
                            }
                        }

                        // Deny request
                        send_msg(stream, &Deny("Invalid request".to_string())).await;
                    }
                }
            }
        }

        if changed {
            entry_mutex_async!(database, |guard| {
                Database::update(guard).await;
            });
        }
    }
}