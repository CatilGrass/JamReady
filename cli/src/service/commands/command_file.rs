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
use crate::data::client_result::ClientResult;

pub struct FileOperationCommand;

#[async_trait]
impl Command for FileOperationCommand {

    async fn local(&self, stream: &mut TcpStream, args: Vec<&str>) -> Option<ClientResult> {

        // 参数校验
        if args.len() < 3 { return None; }

        let mut command_result = ClientResult::result().await;

        // 此命令的所有操作均在服务端完成，客户端仅处理服务端的响应
        // 处理服务器响应
        let cmd_name = args[1].to_uppercase();
        match read_msg(stream).await {
            Text(msg) => {
                sync_local(stream).await;
                command_result.log(format!("{} {}", format!("[ {} ]", cmd_name).cyan(), msg.as_str()).as_str());
            }
            Deny(msg) => {
                sync_local(stream).await;
                command_result.warn(format!("{} {}", format!("[ {} ]", cmd_name).cyan(), msg.as_str()).as_str());
                return Some(command_result);
            }
            _ => {
                return Some(command_result);
            },
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
                    command_result.warn("Virtual file created but missing locally.");
                    command_result.log("Save completed file to:");
                    command_result.log(format!("{}", local_file_path_buf.display().to_string().green()).as_str());
                }
            }

            LocalFileMap::update(&local).await;
        }

        Some(command_result)
    }

    async fn remote(
        &self,
        stream: &mut TcpStream,
        args: Vec<&str>,
        (uuid, _member): (String, &Member),
        database: Arc<Mutex<Database>>
    ) {
        // 参数校验
        if args.len() < 3 {
            send_msg(stream, &Deny("Insufficient arguments".to_string())).await;
            return;
        }

        let operation = args[1].to_lowercase();
        let inputs = args[2].split("|");

        let mut total = 0;
        let mut success = 0;
        let mut fail = 0;

        // 发送消息 -> 同步 -> 返回
        // 或
        // 增加失败次数 -> 更新错误信息，保证后续无执行

        match operation.trim() {

            // 文件添加
            "add" => {
                entry_mutex_async!(database, |guard| {
                    if guard.search_file(args[2].to_string()).is_some() {
                        send_msg(stream, &Deny(format!("File '{}' already exists", args[2]))).await;
                        sync_remote(stream, guard).await;
                        return;
                    }

                    match guard.insert_virtual_file(VirtualFile::new(args[2].to_string())) {
                        Ok(true) => {
                            send_msg(stream, &Text(format!("Created virtual file '{}'", args[2]))).await;
                            sync_remote(stream, guard).await;
                            Database::update(guard).await;
                            return;
                        }
                        _ => {
                            send_msg(stream, &Deny("Failed to create virtual file".to_string())).await;
                            sync_remote(stream, guard).await;
                            return;
                        }
                    }
                })
            }

            // 文件移除
            "remove" => {
                entry_mutex_async!(database, |guard| {
                    for input in inputs {
                        total += 1;
                        let path = process_path_text(input.to_string());
                        let Some(file) = guard.search_file_mut(input.to_string()) else {
                            fail += 1;
                            continue;
                        };
                        if !is_available(file, stream, uuid.clone()).await {
                            fail += 1;
                            continue;
                        }
                        match guard.remove_file_map(path) {
                            Ok(_) => success += 1,
                            Err(_) => fail += 1
                        }
                    }
                });
            }

            // 文件移动
            "move" => {
                if args.len() < 4 {

                    // 缺失目标点
                    send_msg(stream, &Deny("Missing destination path".to_string())).await;
                    entry_mutex_async!(database, |guard| sync_remote(stream, guard).await);
                    return;
                }

                let from = inputs.map(|s| s.to_string()).collect::<Vec<String>>();
                let to = args[3].split("|").map(|s| s.to_string()).collect::<Vec<String>>();
                let from_count = from.len();
                let to_count = to.len();

                // 目标数量和输入数量不匹配
                if from_count != to_count {
                    send_msg(stream, &Deny("The number of \"from\" and \"to\" parameters does not match.".to_string())).await;
                    entry_mutex_async!(database, |guard| sync_remote(stream, guard).await);
                    return;
                }

                // 匹配，构建表
                let mut i = 0;
                while i < from_count {
                    total += 1;
                    let def = String::default();
                    let (from_path, to_path) =
                        (from.get(i).unwrap_or(&def),
                         to.get(i).unwrap_or(&def));
                    let (from_path, to_path) = (from_path.clone(), to_path.clone());
                    i += 1;

                    entry_mutex_async!(database, |guard| {
                        let Some(file) = guard.search_file_mut(from_path.clone()) else {
                            fail += 1;
                            continue;
                        };

                        if !is_available(file, stream, uuid.clone()).await {
                            continue;
                        }

                        let dest = process_path_text(to_path);

                        // 尝试路径移动
                        if guard.move_file(from_path.clone(), dest.clone()).is_ok() {
                            success += 1;
                            continue;
                        }

                        // 尝试UUID移动
                        if guard.move_file_with_uuid(from_path, dest).is_ok() {
                            success += 1;
                            continue;
                        }

                        // 跳出条件判断则视为失败
                        fail += 1;
                    });
                }
            }

            // 回滚操作
            "rollback" => {
                if args.len() < 4 {
                    send_msg(stream, &Deny("Please specify the version to roll back to.".to_string())).await;
                    entry_mutex_async!(database, |guard| sync_remote(stream, guard).await);
                    return;
                }

                for input in inputs {
                    total += 1;
                    entry_mutex_async!(database, |guard| {

                        // 文件
                        let Some(file) = guard.search_file_mut(input.to_string()) else {
                            fail += 1;
                            continue;
                        };

                        if !is_available(file, stream, uuid.clone()).await {
                            continue;
                        }

                        // 回滚的版本
                        let Ok(rollback_version) = u32::from_str(args[3].to_string().trim()) else {
                            fail += 1;
                            continue;
                        };

                        // 回滚
                        if file.rollback_to_version(rollback_version) {
                            success += 1;
                            continue;
                        } else {
                            fail += 1;
                            continue;
                        }
                    })
                }
            }

            // 文件锁操作
            "get" | "get_longer" => {
                let is_long = operation.trim() == "get_longer";

                for input in inputs {
                    total += 1;
                    entry_mutex_async!(database, |guard| {
                        let Some(file) = guard.search_file_mut(input.to_string()) else {
                            fail += 1;
                            continue;
                        };

                        if file.give_uuid_locker(uuid.clone(), is_long).await {
                            success += 1;
                        } else {
                            fail += 1;
                        }
                    })
                }
            }

            // 释放文件锁操作
            "throw" => {

                for input in inputs {
                    total += 1;
                    entry_mutex_async!(database, |guard| {
                        let Some(file) = guard.search_file_mut(input.to_string()) else {
                            fail += 1;
                            continue;
                        };

                        match file.get_locker_owner().await {
                            Some((owner, _)) if owner == uuid => {
                                file.throw_locker();
                                success += 1;
                            }
                            Some(_) => {
                                fail += 1;
                            }
                            None => {
                                fail += 1;
                            }
                        }
                    })
                }
            }

            // 未知操作
            _ => {
                send_msg(stream, &Deny(format!("Unknown operation '{}'", operation))).await;
                entry_mutex_async!(database, |guard| sync_remote(stream, guard).await);
            }
        }

        // 处理结果信息
        if fail > 0 || success < 1 {
            send_msg(stream, &Deny(format!("Operate {} files (success {}, fail {})", total, success, fail))).await;
            entry_mutex_async!(database, |guard| {
                sync_remote(stream, guard).await;
            })
        } else {
            send_msg(stream, &Text(format!("Operate {} files", success))).await;
            entry_mutex_async!(database, |guard| {
                sync_remote(stream, guard).await;
            })
        }

        // 成功任何一个就需要保存
        if success > 0 {
            entry_mutex_async!(database, |guard| {
                Database::update(guard).await;
            })
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