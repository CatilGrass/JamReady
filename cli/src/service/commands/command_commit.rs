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
use crate::data::local_file_map::{LocalFile, LocalFileMap};
use crate::service::commands::file_transmitter::{read_file, send_file};
use crate::service::messages::ClientMessage::{Done, Text, Unknown};
use crate::service::messages::{ClientMessage, ServerMessage};
use crate::service::messages::ServerMessage::{Deny, Pass};
use crate::service::service_utils::{read_msg, send_msg};

pub struct CommitCommand;

#[async_trait]
impl Command for CommitCommand {
    async fn local(&self, stream: &mut TcpStream, _args: Vec<&str>) {
        // 同步数据库
        sync_local(stream).await;
        let database = Database::read().await;
        let mut local = LocalFileMap::read().await;

        // 计数器
        let mut all_count = 0;
        let mut success_count = 0;

        // 文件表
        let mut success_files = Vec::new();
        let mut failed_files = Vec::new();

        // 加载工作区
        let workspace = Workspace::read().await;
        if let Some(client) = workspace.client {
            // 寻找数据库内自己锁定的文件
            for file in database.files() {
                // 检查文件是否被当前成员锁定
                let is_locked_by_me = file.get_locker_owner_uuid()
                    .map(|owner_uuid| owner_uuid.trim() == client.uuid.trim())
                    .unwrap_or(false);

                if !is_locked_by_me {
                    continue;
                }

                print!("Found {}. Local: ", format!("\"{}\"", &file.path()).cyan());

                // 获取文件本地路径
                let client_path = match local.file_to_path(&database, file) {
                    Some(path) if path.exists() => path,
                    _ => {
                        print!("{}", "Not Found.\n".red());
                        continue
                    },
                };

                print!("{}, Changed: ", "Exist".green());

                // 计算当前文件 MD5
                let current_md5 = match md5_digest(client_path.clone()) {
                    Ok(md5) => md5,
                    Err(_) => continue,
                };

                // 检查版本是否允许提交
                let local_file = local.file_paths.get(
                    &database.uuid_of_path(file.path()).unwrap_or_default()
                );

                let can_commit = match (local_file, file.version()) {
                    // 远程版本为 0 则允许提交 （新文件）
                    (_, 0) => true,
                    // 本地有记录且版本匹配但 MD5 不同（已修改）
                    (Some(local_file), _) if local_file.local_version == file.version()
                        && local_file.local_digest != current_md5 => true,
                    // 其他情况不允许提交
                    _ => false,
                };

                if !can_commit {
                    print!("{}.\n", "No".red());
                    continue;
                } else {
                    print!("{}, Uploading ...\n", "Changed".green());
                }

                all_count += 1;
                let record_file_path = process_path_text(client_path.display().to_string());

                // 请求服务器上传权限
                send_msg(stream, &Text(file.path())).await;
                match read_msg::<ServerMessage>(stream).await {
                    Pass => {
                        // 上传文件
                        if send_file(stream, client_path.clone()).await.is_ok() {
                            success_count += 1;
                            success_files.push(record_file_path.clone());

                            // 更新本地映射
                            if let Some(uuid) = database.uuid_of_path(file.path()) {
                                let new_version = file.version() + 1;
                                if let Some(local_file) = local.file_paths.get_mut(&uuid) {
                                    local_file.local_version = new_version;
                                    local_file.local_digest = current_md5.clone();
                                } else {
                                    local.file_paths.insert(uuid.clone(), LocalFile {
                                        local_path: file.path().to_string(),
                                        local_version: new_version,
                                        local_digest: current_md5,
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

            // 打印提交结果
            if all_count == 0 {
                eprintln!("Err: No files committed.");
            } else if success_count == all_count {
                println!("Ok: Commited {} files", all_count);
            } else {
                eprintln!("Err: Commited {} files, Success {} files.", all_count, success_count);
            }

            // 打印成功和失败的文件列表
            if !success_files.is_empty() {
                println!("Success:");
                for file in success_files {
                    println!("{}", file);
                }
            }

            if !failed_files.is_empty() {
                eprintln!("Failed:");
                for file in failed_files {
                    eprintln!("{}", file);
                }
            }

            // 发送完成消息
            send_msg(stream, &Done).await
        }
    }

    async fn remote(
        &self,
        stream: &mut TcpStream,
        args: Vec<&str>,
        (uuid, _member): (String, &Member),
        database: Arc<Mutex<Database>>
    ) {
        let mut changed = false;
        let commit_description = args.get(1).unwrap_or(&"Update");

        // 同步数据库
        entry_mutex_async!(database, |guard| {
            sync_remote(stream, guard).await;
        });

        loop {
            select! {
                // 60 秒超时
                _ = sleep(Duration::from_secs(60)) => break,

                // 处理消息
                msg = read_msg::<ClientMessage>(stream) => {
                    if msg == Unknown || msg == Done {
                        break;
                    }

                    if let Text(path) = msg {

                        let pack;

                        // 查找文件
                        entry_mutex_async!(database, |guard| {
                            let Some(file) = guard.file_mut(path.clone()) else {
                                send_msg(stream, &Deny("Virtual file not found.".to_string())).await;
                                continue;
                            };

                            // 检查锁定状态
                            let is_locked_by_client = file.get_locker_owner_uuid()
                                .map(|owner| owner.trim() == uuid.trim())
                                .unwrap_or(false);

                            if !is_locked_by_client {
                                send_msg(stream, &Deny("Lock mismatch".to_string())).await;
                                continue;
                            }

                            // 生成新文件UUID
                            let real_file_uuid = Uuid::new_v4().to_string();

                            // 获取服务器路径
                            if let Some(path) = file.server_path_temp(real_file_uuid.clone()) {
                                send_msg(stream, &Pass).await;
                                pack = Some((path.clone(), real_file_uuid.clone()));
                            } else {
                                send_msg(stream, &Deny("Cannot get server file path.".to_string())).await;
                                continue;
                            }
                        });

                        if let Some((real_path, real_file_uuid)) = pack {

                            // 接收文件
                            if read_file(stream, real_path.clone()).await.is_ok() {

                                entry_mutex_async!(database, |guard| {
                                    let Some(file) = guard.file_mut(path) else {
                                        continue;
                                    };

                                    // 更新文件
                                    file.update(real_file_uuid, commit_description.to_string());
                                    info!("Update file {}: \"{}\"", file.path(), commit_description);

                                    // 如果不是长期锁则释放
                                    if !file.is_longer_lock_unchecked() {
                                        file.throw_locker();
                                    }
                                });

                                changed = true;
                                continue;
                            }
                        }

                        // 拒绝请求
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