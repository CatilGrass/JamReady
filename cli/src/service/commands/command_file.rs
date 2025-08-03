use crate::data::database::{Database, VirtualFile};
use crate::data::local_file_map::{LocalFile, LocalFileMap};
use crate::data::member::Member;
use crate::service::commands::database_sync::{sync_local, sync_remote};
use crate::service::jam_command::Command;
use crate::service::messages::ServerMessage::{Deny, Text};
use crate::service::service_utils::{read_msg, send_msg};
use async_trait::async_trait;
use colored::Colorize;
use jam_ready::utils::file_digest::md5_digest;
use jam_ready::utils::local_archive::LocalArchive;
use jam_ready::utils::text_process::process_path_text;
use jam_ready::entry_mutex_async;
use std::env::current_dir;
use std::str::FromStr;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

pub struct FileOperationCommand;

#[async_trait]
impl Command for FileOperationCommand {

    async fn local(&self, stream: &mut TcpStream, args: Vec<&str>) {
        // 参数校验
        if args.len() < 3 { return; }

        // 此命令的所有操作均在服务端完成，客户端仅处理服务端的响应
        // 处理服务器响应
        match read_msg(stream).await {
            Text(msg) => {
                sync_local(stream).await;
                println!("Ok: {}", msg)
            }
            Deny(msg) => {
                eprintln!("Err: {}", msg);
                return;
            }
            _ => return,
        }

        // 若操作成功，则会开始处理客户端的后续逻辑

        // 文件添加成功后，检查本地是否存在对应文件，若存在，则更新本地映射
        if args[1].to_lowercase().trim() == "add" {
            let mut local = LocalFileMap::read().await;
            let database = Database::read().await;
            let search = args[2];

            if let Ok(current) = current_dir() {
                let local_file_path_buf = current.join(search);

                // 处理本地文件存在的情况
                if local_file_path_buf.exists() {
                    if let Some(file) = database.search_file(search.to_string()) {
                        let file_path = file.path();
                        if let Some(file_uuid) = database.uuid_of_path(file_path.clone()) {
                            local.file_uuids.insert(file_path, file_uuid.clone());
                            local.file_paths.insert(file_uuid, LocalFile {
                                local_path: search.to_string(),
                                local_version: file.version(),
                                local_digest: md5_digest(local_file_path_buf).unwrap_or_default(),
                            });
                        }
                    }
                }
                else {
                    // 否则，提示成员该文件应当被存储的地址
                    println!("Virtual file created but missing locally.");
                    println!("Save completed file to:");
                    println!("{}", local_file_path_buf.display().to_string().green());
                }
            }

            LocalFileMap::update(&local).await;
        }
    }

    async fn remote(
        &self,
        stream: &mut TcpStream,
        args: Vec<&str>,
        (uuid, _member): (String, &Member),
        database: Arc<Mutex<Database>>
    ) -> bool {
        // 参数校验
        if args.len() < 3 {
            send_msg(stream, &Deny("Insufficient arguments".to_string())).await;
            return false;
        }

        let operation = args[1].to_lowercase();
        let input = args[2];

        match operation.trim() {
            // 文件添加
            "add" => {
                entry_mutex_async!(database, |guard| {
                    if guard.search_file(input.to_string()).is_some() {
                        send_msg(stream, &Deny(format!("File '{}' already exists", input))).await;
                        return false;
                    }

                    match guard.insert_virtual_file(VirtualFile::new(input.to_string())) {
                        Ok(true) => {
                            send_msg(stream, &Text(format!("Created virtual file '{}'", input))).await;
                            sync_remote(stream).await;
                            true
                        }
                        _ => {
                            send_msg(stream, &Deny("Failed to create virtual file".to_string())).await;
                            false
                        }
                    }
                })
            }

            // 文件移除
            "remove" => {
                entry_mutex_async!(database, |guard| {
                    let path = process_path_text(input.to_string());
                    let Some(file) = guard.search_file_mut(input.to_string()) else {
                        send_msg(stream, &Deny(format!("File '{}' not found", input))).await;
                        return false;
                    };

                    if !is_available(file, stream, uuid.clone()).await {
                        return false;
                    }

                    match guard.remove_file_map(path) {
                        Ok(_) => {
                            send_msg(stream, &Text(format!("Removed virtual file '{}'", input))).await;
                            sync_remote(stream).await;
                            true
                        }
                        Err(_) => {
                            send_msg(stream, &Deny(format!("Failed to remove '{}'", input))).await;
                            false
                        }
                    }
                })
            }

            // 文件移动
            "move" => {
                if args.len() < 4 {
                    send_msg(stream, &Deny("Missing destination path".to_string())).await;
                    return false;
                }

                entry_mutex_async!(database, |guard| {
                    let Some(file) = guard.search_file_mut(input.to_string()) else {
                        send_msg(stream, &Deny(format!("File '{}' not found", input))).await;
                        return false;
                    };

                    if !is_available(file, stream, uuid.clone()).await {
                        return false;
                    }

                    let dest = process_path_text(args[3].to_string());

                    // 尝试路径移动
                    if guard.move_file(input.to_string(), dest.clone()).is_ok() {
                        send_msg(stream, &Text(format!("Moved '{}' to '{}'", input, args[3]))).await;
                        sync_remote(stream).await;
                        return true;
                    }

                    // 尝试UUID移动
                    if guard.move_file_with_uuid(input.to_string(), dest).is_ok() {
                        send_msg(stream, &Text(format!("Moved UUID '{}' to '{}'", input, args[3]))).await;
                        sync_remote(stream).await;
                        return true;
                    }

                    send_msg(stream, &Deny(format!("Failed to move '{}'", input))).await;
                    false
                })
            }

            // 回滚操作
            "rollback" => {
                if args.len() < 4 {
                    send_msg(stream, &Deny("Missing destination path".to_string())).await;
                    return false;
                }

                entry_mutex_async!(database, |guard| {
                    // 文件
                    let Some(file) = guard.search_file_mut(input.to_string()) else {
                        send_msg(stream, &Deny(format!("File '{}' not found", input))).await;
                        return false;
                    };

                    if !is_available(file, stream, uuid.clone()).await {
                        return false;
                    }

                    // 回滚的版本
                    let Ok(rollback_version) = u32::from_str(args[3].to_string().trim()) else {
                        send_msg(stream, &Deny(format!("Invalid version number '{}' ", args[3]))).await;
                        return false;
                    };

                    // 回滚
                    if file.rollback_to_version(rollback_version) {
                        send_msg(stream, &Text(format!("Rollback to '{}'", rollback_version))).await;
                        sync_remote(stream).await;
                        true
                    } else {
                        send_msg(stream, &Deny(format!("Rollback to '{}' failed", rollback_version))).await;
                        false
                    }
                })
            }

            // 文件锁操作
            "get" | "get_longer" => {
                let is_long = operation.trim() == "get_longer";

                entry_mutex_async!(database, |guard| {
                    let Some(file) = guard.search_file_mut(input.to_string()) else {
                        send_msg(stream, &Deny(format!("File '{}' not found", input))).await;
                        return false;
                    };

                    if file.give_uuid_locker(uuid.clone(), is_long).await {
                        let action = if is_long { "long-term lock" } else { "lock" };
                        send_msg(stream, &Text(format!("Acquired {} on '{}'", action, input))).await;
                        let _ = file;

                        sync_remote(stream).await;
                        true
                    } else {
                        send_msg(stream, &Deny("Failed to acquire lock".to_string())).await;
                        false
                    }
                })
            }

            // 释放文件锁操作
            "throw" => {

                entry_mutex_async!(database, |guard| {
                    let Some(file) = guard.search_file_mut(input.to_string()) else {
                        send_msg(stream, &Deny(format!("File '{}' not found", input))).await;
                        return false;
                    };

                    match file.get_locker_owner().await {
                        Some((owner, _)) if owner == uuid => {
                            file.throw_locker();
                            send_msg(stream, &Text(format!("Released lock on '{}'", input))).await;
                            let _ = file;

                            sync_remote(stream).await;
                            true
                        }
                        Some(_) => {
                            send_msg(stream, &Deny("File locked by another member".to_string())).await;
                            false
                        }
                        None => {
                            send_msg(stream, &Deny("File is not locked".to_string())).await;
                            false
                        }
                    }
                })
            }

            // 未知操作
            _ => {
                send_msg(stream, &Deny(format!("Unknown operation '{}'", operation))).await;
                false
            }
        }
    }
}

/// 检查文件可用性（锁定状态）
async fn is_available(file: &VirtualFile, stream: &mut TcpStream, self_uuid: String) -> bool {
    match file.get_locker_owner().await {
        Some((owner, _)) if owner != self_uuid => {
            send_msg(stream, &Deny("File locked by another team member".to_string())).await;
            false
        }
        None => {
            send_msg(stream, &Deny("Acquire lock before file operations".to_string())).await;
            false
        }
        _ => true
    }
}