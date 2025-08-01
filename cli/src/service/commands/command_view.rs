use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;
use crate::data::database::Database;
use crate::data::local_file_map::{LocalFile, LocalFileMap};
use crate::data::member::Member;
use crate::service::commands::database_sync::{sync_local, sync_remote};
use crate::service::commands::file_transmitter::{read_file, send_file};
use crate::service::jam_command::Command;
use async_trait::async_trait;
use jam_ready::utils::local_archive::LocalArchive;
use tokio::net::TcpStream;
use tokio::select;
use tokio::time::sleep;
use jam_ready::utils::file_digest::md5_digest;
use jam_ready::utils::text_process::process_path_text;
use crate::service::messages::{ClientMessage, ServerMessage};
use crate::service::service_utils::{read_msg, send_msg};

pub struct ViewCommand;

#[async_trait]
impl Command for ViewCommand {

    async fn local(&self, stream: &mut TcpStream, args: Vec<&str>) {

        // 同步数据库
        sync_local(stream).await;
        let database = Database::read();

        // 加载本地映射表
        let mut local = LocalFileMap::read();

        // 检查参数数量
        if args.len() < 2 { return; } // <搜索>

        // 成功状态
        let mut success = false;
        let mut print_msg = "".to_string();

        // 文件目录
        let file_path_str = args[1].to_string();
        let file = database.search_file(file_path_str.clone());
        if let Some(file) = file {

            // 尝试寻找本地映射，找不到用默认映射
            if let Some(client_path) = local.file_to_path(&database, file) {
                // 是否准备就绪
                let mut ready = true;

                // 如果文件存在且版本大于或等于服务端的版本，则不下载
                if let Some(local_uuid) = database.uuid_of_path(file.path()) {
                    if let Some(local_file) = local.file_paths.get(&local_uuid) {
                        if local_file.local_version >= file.version() && client_path.exists() {
                            // 发送 "未就绪"
                            send_msg(stream, &ClientMessage::NotReady).await;
                            ready = false;

                            print_msg = "The file is already the latest version, no need to download".to_string();
                        }
                    }
                }

                if ready {
                    // 发送 "就绪"
                    send_msg(stream, &ClientMessage::Ready).await;

                    // 下载文件
                    match read_file(stream, client_path.clone()).await {
                        Ok(_) => {
                            // 获得本地文件映射的键
                            let local_path_buf;
                            if let Some(p) = local.search_to_path_relative(&database, file.path()) {
                                local_path_buf = p;
                            } else {
                                local_path_buf = PathBuf::from_str(file.path().as_str()).unwrap();
                            }

                            // 写入本地文件映射
                            let local_path_str = process_path_text(local_path_buf.display().to_string());
                            if let Some(uuid) = database.uuid_of_path(file.path()) {
                                local.file_paths.insert(uuid.clone(), LocalFile {
                                    local_path: local_path_str.clone(),
                                    local_version: file.version(),
                                    local_digest: md5_digest(client_path).unwrap_or("".to_string()),
                                });
                                local.file_uuids.insert(local_path_str, uuid.clone());
                            }
                            print_msg = "File download completed".to_string();
                            success = true;
                        }
                        Err(_) => {
                            print_msg = "File download failed".to_string();
                        }
                    }
                }
            }
        }

        // 读取结束消息
        select! {
            _ = sleep(Duration::from_secs(2)) => {
                print_msg = "Timeout".to_string();
            }
            result = read_msg::<ServerMessage>(stream) => {
                match result {
                    ServerMessage::Deny(err) => {
                        print_msg = format!("{}. {}", print_msg, err);
                    }
                    ServerMessage::Done => {}
                    _ => {}
                }
            }
        }

        if success {
            println!("Ok: {}", print_msg);
        } else {
            eprintln!("Err: {}", print_msg);
        }

        LocalFileMap::update(&local);
    }

    async fn remote(&self, stream: &mut TcpStream, args: Vec<&str>, (_uuid, _member): (String, &Member), database: &mut Database) -> bool {

        // 首先同步数据库
        sync_remote(stream, database).await;

        // 检查参数数量
        if args.len() < 2 { return false; } // <搜索>

        // 确认客户端的准备状态
        let read_message: ClientMessage = read_msg(stream).await;

        // 成功状态
        let mut success = false;
        let mut return_message = "".to_string();

        match read_message {
            ClientMessage::Ready => {
                // 文件路径
                let file_path_str = args[1].to_string();
                let file = database.search_file(file_path_str.clone());
                if let Some(file) = file {
                    if let Some(server_path) = file.server_path() {
                        match send_file(stream, server_path).await {
                            Ok(_) => {
                                success = true;
                            }
                            Err(err) => {
                                let err_string = format!("{}", err);
                                return_message = err_string;
                            }
                        }
                    }
                }
            }
            _ => { }
        }

        if success {
            send_msg(stream, &ServerMessage::Done).await;
        } else {
            send_msg(stream, &ServerMessage::Deny(return_message.to_string())).await;
        }
        false
    }
}