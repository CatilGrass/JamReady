use std::env::current_dir;
use std::ops::Add;
use jam_ready::utils::file_digest::md5_digest;
use jam_ready::utils::local_archive::LocalArchive;
use crate::cli_commands::cli_command_client::param_comp::comp::{comp_param_from, comp_param_to};
use crate::cli_commands::cli_command_client::param_comp::data::{CompConfig, CompContext};
use crate::cli_commands::client::ClientQueryCommands;
use crate::data::client_result::{ClientResult, ClientResultQueryProcess};
use crate::data::database::Database;
use crate::data::local_file_map::LocalFileMap;
use crate::data::local_folder_map::{LocalFolderMap, Node};
use crate::data::workspace::Workspace;

pub async fn client_query(command: ClientQueryCommands) {
    match command {

        // List directory structure
        ClientQueryCommands::ListDirectory(args) => {
            let mut result = ClientResult::query(ClientResultQueryProcess::line_by_line).await;
            if args.completion_mode { result.set_debug(false); }
            let folder_map = LocalFolderMap::read().await;
            let database = Database::read().await;
            let current = args.value
                .trim()
                .trim_start_matches("./")
                .trim_start_matches("/");

            // Local files
            if let Ok(current_dir) = current_dir() {
                let current_folder = current_dir.join(current);
                if current_folder.exists() {
                    if let Ok(dir) = current_folder.read_dir() {
                        for dir in dir.into_iter() {
                            if dir.is_err() { continue; }
                            let dir = dir.unwrap().path();
                            if let Some(os_name) = dir.file_name() {
                                if let Some(name) = os_name.to_str() {
                                    let mut path = format!("{}{}", current, name);
                                    if dir.is_dir() {
                                        path = format!("{}/", path);
                                        if path == env!("PATH_WORKSPACE_ROOT") { continue }
                                        if ! folder_map.folder_files.contains_key(&path) {
                                            result.log(format!("{}/", name).as_str());
                                        }
                                    } else {
                                        if ! database.contains_path(&path) {
                                            result.log(format!("{}", name).as_str());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Remote files
            let list = folder_map.folder_files.get(current);
            if let Some(list) = list {
                for item in list {
                    match item {
                        Node::Jump(directory_str) => {
                            let v = process_path(directory_str.trim().trim_end_matches('/'))
                                .to_string().add("/");
                            result.log(v.as_str());
                        }
                        Node::File(virtual_file_path_str) => {
                            let v = process_path(virtual_file_path_str);
                            result.log(v.as_str());
                        }
                        _ => { continue; }
                    }
                }
            }

            // Short names
            if args.completion_mode {
                for (k, _v) in folder_map.short_file_map {
                    result.log(format!(":{}", k).as_str());
                }
            }

            result.end_print();
        }

        // Query virtual file's Uuid
        ClientQueryCommands::FileUuid(args) => {
            let mut result = ClientResult::query(ClientResultQueryProcess::direct).await;
            let database = Database::read().await;
            if let Some(file) = database.search_file(args.value.trim().to_string()) {
                if let Some(uuid) = database.uuid_of_path(file.path()) {
                    result.log(uuid.as_str());
                    result.end_print();
                }
            }
        }

        // Query virtual file's version
        ClientQueryCommands::FileVersion(args) => {
            let mut result = ClientResult::query(ClientResultQueryProcess::direct).await;
            let database = Database::read().await;
            if let Some(file) = database.search_file(args.value.trim().to_string()) {
                result.log(format!("{}", file.version()).as_str());
                result.end_print();
            }
        }

        // Query virtual file's path
        ClientQueryCommands::FilePath(args) => {
            let mut result = ClientResult::query(ClientResultQueryProcess::direct).await;
            let database = Database::read().await;
            if let Some(file) = database.search_file(args.value.trim().to_string()) {
                result.log(format!("{}", file.path()).as_str());
                result.end_print();
            }
        }

        // Query virtual file's name
        ClientQueryCommands::FileName(args) => {
            let mut result = ClientResult::query(ClientResultQueryProcess::direct).await;
            let database = Database::read().await;
            if let Some(file) = database.search_file(args.value.trim().to_string()) {
                result.log(format!("{}", process_path(file.path().as_str())).as_str());
                result.end_print();
            }
        }

        // Query virtual file's lock status
        ClientQueryCommands::FileLockStatus(args) => {
            let mut result = ClientResult::query(ClientResultQueryProcess::direct).await;
            let database = Database::read().await;
            let workspace = Workspace::read().await;
            if let Some(file) = database.search_file(args.value.trim().to_string()) {
                if let Some(locker_owner) = file.get_locker_owner_uuid() {
                    if locker_owner == workspace.client.unwrap().uuid {
                        if file.is_longer_lock_unchecked() {
                            result.log("HELD");
                        } else {
                            result.log("held")
                        }
                    } else {
                        if file.is_longer_lock_unchecked() {
                            result.log("LOCK")
                        } else {
                            result.log("lock")
                        }
                    }
                } else {
                    result.log("Available")
                }
            }
            result.end_print();
        }

        // Query self Uuid
        ClientQueryCommands::SelfUuid => {
            let mut result = ClientResult::query(ClientResultQueryProcess::direct).await;
            result.log(format!("{}", Workspace::read().await.client.unwrap().uuid).as_str());
            result.end_print();
        }

        // Query target workspace address
        ClientQueryCommands::TargetAddress => {
            let mut result = ClientResult::query(ClientResultQueryProcess::direct).await;
            result.log(format!("{}", Workspace::read().await.client.unwrap().target_addr).as_str());
            result.end_print();
        }

        // Query target workspace name
        ClientQueryCommands::Workspace => {
            let mut result = ClientResult::query(ClientResultQueryProcess::direct).await;
            result.log(format!("{}", Workspace::read().await.client.unwrap().workspace_name).as_str());
            result.end_print();
        }

        // Check if virtual file exists locally
        ClientQueryCommands::ContainLocal(args) => {
            let mut result = ClientResult::query(ClientResultQueryProcess::direct).await;
            let database = Database::read().await;
            let local = LocalFileMap::read().await;
            if let Some(file) = database.search_file(args.value.trim().to_string()) {
                if let Some(uuid) = database.uuid_of_path(file.path()) {
                    if let Some(_) = local.file_paths.get(uuid.as_str()) {
                        result.log("true");
                    } else {
                        result.log("false");
                    }
                }
            }
            result.end_print();
        }

        // Query virtual file mapped from local file
        ClientQueryCommands::LocalToRemote(args) => {
            let mut result = ClientResult::query(ClientResultQueryProcess::direct).await;
            let database = Database::read().await;
            let local = LocalFileMap::read().await;
            if let Some(uuid) = local.local_path_to_uuid(args.value.trim().to_string()) {
                if let Some(file) = database.search_file(uuid.trim().to_string()) {
                    if file.path().is_empty() {
                        result.log(format!("{}", uuid).as_str());
                    } else {
                        result.log(format!("{}", file.path()).as_str());
                    }
                }
            }
            result.end_print();
        }

        // Query local file mapped from virtual file
        ClientQueryCommands::RemoteToLocal(args) => {
            let mut result = ClientResult::query(ClientResultQueryProcess::direct).await;
            let database = Database::read().await;
            let local = LocalFileMap::read().await;
            if let Some(file) = database.search_file(args.value.trim().to_string()) {
                if let Some(local_file) = local.search_to_local(&database, file.path()) {
                    result.log(format!("{}", local_file.local_path).as_str());
                }
            }
            result.end_print();
        }

        // Check if local file has been modified
        ClientQueryCommands::Changed(args) => {
            let mut result = ClientResult::query(ClientResultQueryProcess::direct).await;
            let database = Database::read().await;
            let local = LocalFileMap::read().await;
            if let Some(file) = database.search_file(args.value.trim().to_string()) {
                if let Some(local_file) = local.search_to_local(&database, file.path()) {
                    let local_digest = &local_file.local_digest;
                    let current_digest = if let Some(path_buf) = local.search_to_path(&database, args.value.trim().to_string()) {
                        if path_buf.exists() {
                            Some(md5_digest(path_buf).unwrap_or(local_digest.clone()))
                        } else {
                            Some(local_digest.clone())
                        }
                    } else {
                        None
                    };
                    if let Some(digest) = current_digest {
                        if digest.trim() == local_digest {
                            result.log("false");
                        } else {
                            result.log("true");
                        }
                    }
                }
            }
            result.end_print();
        }

        // Query local file version
        ClientQueryCommands::LocalVersion(args) => {
            let mut result = ClientResult::query(ClientResultQueryProcess::direct).await;
            let database = Database::read().await;
            let local = LocalFileMap::read().await;
            if let Some(local_file) = local.search_to_local(&database, args.value.trim().to_string()) {
                result.log(format!("{}", local_file.local_version).as_str());
            }
            result.end_print();
        }

        // Search Test
        ClientQueryCommands::Search(args) => {
            let mut result = ClientResult::result().await;
            let config = CompConfig::read().await;

            // Compile FROM input
            let from = comp_param_from(&config, CompContext::input(&args.from_search));
            let Ok(from) = from else {
                result.err_and_end(format!("{}", from.err().unwrap()).as_str());
                return;
            };

            result.log("FROM RESULTS: ");
            for final_path in from.final_paths.clone() {
                result.log(final_path.as_str());
            }

            // Compile TO input
            if let Some(to_search) = args.to_search {
                let to = comp_param_to(&config, from.clone().next_with_string(to_search));
                let Ok(to) = to else {
                    result.err_and_end(format!("{}", to.err().unwrap()).as_str());
                    return;
                };

                result.log("  TO RESULTS: ");
                for final_path in to.final_paths {
                    result.log(final_path.as_str());
                }
            }

            result.end_print();
        }
    }

    fn process_path(input: &str) -> String {
        let binding = input.to_string();
        binding.split("/").last().unwrap_or("").to_string()
    }
}