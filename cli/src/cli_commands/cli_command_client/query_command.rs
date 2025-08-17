use std::env::current_dir;
use std::ops::Add;
use jam_ready::utils::file_digest::md5_digest;
use jam_ready::utils::local_archive::LocalArchive;
use crate::cli_commands::client::ClientQueryCommands;
use crate::data::client_result::{ClientResult, ClientResultQueryProcess};
use crate::data::database::Database;
use crate::data::local_file_map::LocalFileMap;
use crate::data::local_folder_map::{LocalFolderMap, Node};
use crate::data::workspace::Workspace;

pub async fn client_query(command: ClientQueryCommands) {
    match command {

        // 列出某个目录下的结构
        ClientQueryCommands::ListDirectory(args) => {
            let mut result = ClientResult::query(ClientResultQueryProcess::line_by_line).await;
            if args.completion_mode { result.set_debug(false); }
            let folder_map = LocalFolderMap::read().await;
            let database = Database::read().await;
            let current = args.value
                .trim()
                .trim_start_matches("./")
                .trim_start_matches("/");

            // 本地文件
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

            // 远程文件
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

            // 短名称
            if args.completion_mode {
                for (k, _v) in folder_map.short_file_map {
                    result.log(format!(":{}", k).as_str());
                }
            }

            result.end_print();
        }

        // 查询虚拟文件的Uuid
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

        // 查询虚拟文件的版本
        ClientQueryCommands::FileVersion(args) => {
            let mut result = ClientResult::query(ClientResultQueryProcess::direct).await;
            let database = Database::read().await;
            if let Some(file) = database.search_file(args.value.trim().to_string()) {
                result.log(format!("{}", file.version()).as_str());
                result.end_print();
            }
        }

        // 查询虚拟文件的路径
        ClientQueryCommands::FilePath(args) => {
            let mut result = ClientResult::query(ClientResultQueryProcess::direct).await;
            let database = Database::read().await;
            if let Some(file) = database.search_file(args.value.trim().to_string()) {
                result.log(format!("{}", file.path()).as_str());
                result.end_print();
            }
        }

        // 查询虚拟文件的名称
        ClientQueryCommands::FileName(args) => {
            let mut result = ClientResult::query(ClientResultQueryProcess::direct).await;
            let database = Database::read().await;
            if let Some(file) = database.search_file(args.value.trim().to_string()) {
                result.log(format!("{}", process_path(file.path().as_str())).as_str());
                result.end_print();
            }
        }

        // 查询虚拟文件的锁定状态
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

        // 查询自己的Uuid
        ClientQueryCommands::SelfUuid => {
            let mut result = ClientResult::query(ClientResultQueryProcess::direct).await;
            result.log(format!("{}", Workspace::read().await.client.unwrap().uuid).as_str());
            result.end_print();
        }

        // 查询目标工作区地址
        ClientQueryCommands::TargetAddress => {
            let mut result = ClientResult::query(ClientResultQueryProcess::direct).await;
            result.log(format!("{}", Workspace::read().await.client.unwrap().target_addr).as_str());
            result.end_print();
        }

        // 查询目标工作区名称
        ClientQueryCommands::Workspace => {
            let mut result = ClientResult::query(ClientResultQueryProcess::direct).await;
            result.log(format!("{}", Workspace::read().await.client.unwrap().workspace_name).as_str());
            result.end_print();
        }

        // 查询虚拟文件是否在本地
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

        // 查询本地文件映射的虚拟文件
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

        // 查询虚拟文件映射的本地文件
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

        // 查询本地文件是否被更改
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

        // 查询本地文件的版本号
        ClientQueryCommands::LocalVersion(args) => {
            let mut result = ClientResult::query(ClientResultQueryProcess::direct).await;
            let database = Database::read().await;
            let local = LocalFileMap::read().await;
            if let Some(local_file) = local.search_to_local(&database, args.value.trim().to_string()) {
                result.log(format!("{}", local_file.local_version).as_str());
            }
            result.end_print();
        }
    }

    fn process_path(input: &str) -> String {
        let binding = input.to_string();
        binding.split("/").last().unwrap_or("").to_string()
    }
}