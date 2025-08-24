use crate::data::client_result::ClientResult;
use crate::data::database::{Database, VirtualFile};
use crate::data::local_file_map::{LocalFile, LocalFileMap};
use crate::data::member::Member;
use crate::service::commands::utils_database_sync::{sync_local, sync_remote};
use crate::service::commands::utils_file_transmitter::{read_file, send_file};
use crate::service::jam_command::Command;
use crate::service::messages::{ClientMessage, ServerMessage};
use crate::service::service_utils::{read_msg, send_msg};
use async_trait::async_trait;
use jam_ready::entry_mutex_async;
use jam_ready::utils::file_digest::md5_digest;
use jam_ready::utils::file_operation::copy_file;
use jam_ready::utils::local_archive::LocalArchive;
use jam_ready::utils::text_process::process_path_text;
use std::env::current_dir;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::select;
use tokio::sync::Mutex;
use tokio::time::sleep;

pub struct ViewCommand;

#[async_trait]
impl Command for ViewCommand {

    async fn local(&self, stream: &mut TcpStream, args: Vec<&str>) -> Option<ClientResult> {
        let mut command_result = ClientResult::result().await;

        // Sync database
        sync_local(stream).await;
        let database = Database::read().await;

        // Load local file mapping
        let mut local = LocalFileMap::read().await;

        // Validate arguments
        if args.len() < 2 { return None; } // <search>
        let inputs = args[1].split("|");

        // Check for version parameter <search> <version>
        let view_version = if args.len() < 3 { "0" } else { args[2] };

        for input in inputs {
            let mut success = false;
            let mut print_msg = String::new();

            let file_path_str = input.to_string();
            if let Some(file) = database.search_file(file_path_str.clone()) {
                if let Some(client_path) = local.file_to_path(&database, file) {
                    let mut ready = true;

                    // Check if file exists locally and matches server version
                    if let Some(local_uuid) = database.uuid_of_path(file.path()) {
                        if let Some(local_file) = local.file_paths.get(&local_uuid) {
                            if local_file.local_version == file.version() &&
                                client_path.exists() &&
                                view_version == "0" {
                                send_msg(stream, &ClientMessage::NotReady).await;
                                ready = false;
                                print_msg = "The file is already the latest version, no need to download".to_string();
                            }
                        }
                    }

                    // Check local cache files
                    if ready {
                        if let Some(cache_file) = local_cache_file(file, view_version) {
                            if cache_file.exists() {
                                match copy_file(&cache_file, &client_path.clone()) {
                                    Ok(_) => {
                                        print_msg = "File download completed! (from local cache)".to_string();
                                        success = true;
                                        ready = false;

                                        let uuid = database.uuid_of_path(file.path()).unwrap_or("".to_string());
                                        let local_path_str = if let Some(local_file) = local.search_to_local(&database, file.path()) {
                                            local_file.local_path.clone()
                                        } else {
                                            file.path()
                                        };

                                        generate_local_file_map_info(&mut local, file, client_path.clone(), local_path_str, uuid, view_version);

                                        send_msg(stream, &ClientMessage::NotReady).await;
                                    }
                                    Err(_) => {
                                        success = false;
                                        ready = true;
                                        // Not ready, trying to download
                                    }
                                }
                            }
                        }
                    }

                    if ready {
                        send_msg(stream, &ClientMessage::Ready).await;

                        match read_file(stream, client_path.clone()).await {
                            Ok(_) => {
                                let local_path_buf = match local.search_to_path_relative(&database, file.path()) {
                                    Some(p) => p,
                                    None => PathBuf::from_str(file.path().as_str()).unwrap(),
                                };

                                let local_path_str = process_path_text(local_path_buf.display().to_string());
                                if let Some(uuid) = database.uuid_of_path(file.path()) {
                                    generate_local_file_map_info(&mut local, file, client_path.clone(), local_path_str, uuid, view_version);
                                }
                                print_msg = "File download completed!".to_string();
                                success = true;

                                // Attempting to establish cache
                                if let Some(path) = local_cache_file(file, view_version) {
                                    if ! path.exists() {
                                        let _ = copy_file(&client_path.clone(), &path);
                                    }
                                }
                            }
                            Err(_) => {
                                print_msg = "File download failed".to_string();
                            }
                        }
                    }
                }
            }

            // Handle timeout or server response
            select! {
                _ = sleep(Duration::from_secs(15)) => {
                    print_msg = "Timeout".to_string();
                }
                result = read_msg::<ServerMessage>(stream) => {
                    if let ServerMessage::Deny(err) = result {
                        print_msg = format!("{} {}", print_msg, err);
                    }
                }
            }

            if success {
                command_result.log(&print_msg);
            } else {
                command_result.err(&print_msg);
            }
        }

        LocalFileMap::update(&local).await;
        Some(command_result)
    }

    async fn remote(
        &self,
        stream: &mut TcpStream,
        args: Vec<&str>,
        (_uuid, _member): (String, &Member),
        database: Arc<Mutex<Database>>
    ) {
        // First sync database
        entry_mutex_async!(database, |guard| {
            sync_remote(stream, guard).await;
        });

        let database = Database::read().await;

        // Validate arguments
        if args.len() < 2 { return; } // <search>
        let inputs = args[1].split("|");

        // Check for version parameter <search> <version>
        let view_version = if args.len() < 3 { "0" } else { args[2] };

        for input in inputs {
            let mut success = false;
            let mut return_message = String::new();

            match read_msg::<ClientMessage>(stream).await {
                ClientMessage::Ready => {
                    let file_path_str = input.to_string();
                    if let Some(file) = database.search_file(file_path_str.clone()) {
                        let real = if view_version == "0" {
                            file.server_path()
                        } else {
                            u32::from_str(view_version)
                                .ok()
                                .and_then(|v| file.server_path_version(v))
                        };

                        if let Some(server_path) = real {
                            match send_file(stream, server_path).await {
                                Ok(_) => success = true,
                                Err(err) => return_message = err.to_string(),
                            }
                        }
                    }
                }
                _ => {}
            }

            if success {
                send_msg(stream, &ServerMessage::Done).await;
            } else {
                send_msg(stream, &ServerMessage::Deny(return_message)).await;
            }
        }
    }
}

fn local_cache_file(virtual_path: &VirtualFile, version_str: &str) -> Option<PathBuf> {
    let Ok(current_dir) = current_dir() else { return None };
    let Ok(version) = u32::from_str(version_str) else { return None };
    if let Some(path) = virtual_path.real_path_version(version) {
        return Some(current_dir.join(env!("PATH_CACHE")).join(path))
    }
    None
}

fn generate_local_file_map_info(
    local: &mut LocalFileMap, file: &VirtualFile,
    client_path: PathBuf, local_path_str: String, uuid: String, view_version: &str) {
    local.file_paths.insert(uuid.clone(), LocalFile {
        local_path: local_path_str.clone(),
        local_version: if let Ok(version) = u32::from_str(view_version) {
            if version == 0 { file.version() } else { version }
        } else {
            file.version()
        },
        local_digest: md5_digest(client_path).unwrap_or_default(),
        completed: false,
        completed_digest: String::new(),
        completed_commit: String::new(),
    });
    local.file_uuids.insert(local_path_str, uuid);
}