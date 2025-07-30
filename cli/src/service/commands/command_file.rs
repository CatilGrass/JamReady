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
use std::env::current_dir;
use tokio::net::TcpStream;

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
            let mut local = LocalFileMap::read();
            let database = Database::read();
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

            LocalFileMap::update(&local);
        }
    }

    async fn remote(
        &self,
        stream: &mut TcpStream,
        args: Vec<&str>,
        (uuid, _member): (String, &Member),
        database: &mut Database
    ) -> bool {
        // 参数校验
        if args.len() < 3 {
            send_msg(stream, &Deny("Insufficient arguments".to_string())).await;
            return false;
        }

        let operation = args[1].to_lowercase();
        let input = args[2];

        match operation.trim() {
            // ===== 文件添加操作 =====
            "add" => {
                if database.search_file(input.to_string()).is_some() {
                    send_msg(stream, &Deny(format!("File '{}' already exists", input))).await;
                    return false;
                }

                match database.insert_virtual_file(VirtualFile::new(input.to_string())) {
                    Ok(true) => {
                        send_msg(stream, &Text(format!("Created virtual file '{}'", input))).await;
                        sync_remote(stream, database).await;
                        true
                    }
                    _ => {
                        send_msg(stream, &Deny("Failed to create virtual file".to_string())).await;
                        false
                    }
                }
            }

            // ===== 文件移除操作 =====
            "remove" => {
                let path = process_path_text(input.to_string());
                let Some(file) = database.search_file_mut(input.to_string()) else {
                    send_msg(stream, &Deny(format!("File '{}' not found", input))).await;
                    return false;
                };

                if !is_available(file, stream, uuid.clone()).await {
                    return false;
                }

                match database.remove_file_map(path) {
                    Ok(_) => {
                        send_msg(stream, &Text(format!("Removed virtual file '{}'", input))).await;
                        sync_remote(stream, database).await;
                        true
                    }
                    Err(_) => {
                        send_msg(stream, &Deny(format!("Failed to remove '{}'", input))).await;
                        false
                    }
                }
            }

            // ===== 文件移动操作 =====
            "move" => {
                if args.len() < 4 {
                    send_msg(stream, &Deny("Missing destination path".to_string())).await;
                    return false;
                }

                let Some(file) = database.search_file_mut(input.to_string()) else {
                    send_msg(stream, &Deny(format!("File '{}' not found", input))).await;
                    return false;
                };

                if !is_available(file, stream, uuid.clone()).await {
                    return false;
                }

                let dest = process_path_text(args[3].to_string());

                // 尝试路径移动
                if database.move_file(input.to_string(), dest.clone()).is_ok() {
                    send_msg(stream, &Text(format!("Moved '{}' to '{}'", input, args[3]))).await;
                    sync_remote(stream, database).await;
                    return true;
                }

                // 尝试UUID移动
                if database.move_file_with_uuid(input.to_string(), dest).is_ok() {
                    send_msg(stream, &Text(format!("Moved UUID '{}' to '{}'", input, args[3]))).await;
                    sync_remote(stream, database).await;
                    return true;
                }

                send_msg(stream, &Deny(format!("Failed to move '{}'", input))).await;
                false
            }

            // ===== 文件锁操作 =====
            "get" | "get_longer" => {
                let is_long = operation.trim() == "get_longer";

                // 仅当需要时才获取文件的可变引用
                let Some(file) = database.search_file_mut(input.to_string()) else {
                    send_msg(stream, &Deny(format!("File '{}' not found", input))).await;
                    return false;
                };

                if file.give_uuid_locker(uuid.clone(), is_long) {
                    let action = if is_long { "long-term lock" } else { "lock" };
                    send_msg(stream, &Text(format!("Acquired {} on '{}'", action, input))).await;
                    // 立即释放文件的可变引用
                    let _ = file;

                    // 现在可以安全地借用整个数据库进行同步
                    sync_remote(stream, database).await;
                    true
                } else {
                    send_msg(stream, &Deny("Failed to acquire lock".to_string())).await;
                    false
                }
            }

            // ===== 释放文件锁操作 =====
            "throw" => {
                // 仅当需要时才获取文件的可变引用
                let Some(file) = database.search_file_mut(input.to_string()) else {
                    send_msg(stream, &Deny(format!("File '{}' not found", input))).await;
                    return false;
                };

                match file.get_locker_owner() {
                    Some((owner, _)) if owner == uuid => {
                        file.throw_locker();
                        send_msg(stream, &Text(format!("Released lock on '{}'", input))).await;
                        // 立即释放文件的可变引用
                        let _ = file;

                        // 现在可以安全地借用整个数据库进行同步
                        sync_remote(stream, database).await;
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
            }

            // ===== 未知操作处理 =====
            _ => {
                send_msg(stream, &Deny(format!("Unknown operation '{}'", operation))).await;
                false
            }
        }
    }
}

/// 检查文件可用性（锁定状态）
async fn is_available(file: &VirtualFile, stream: &mut TcpStream, self_uuid: String) -> bool {
    match file.get_locker_owner() {
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