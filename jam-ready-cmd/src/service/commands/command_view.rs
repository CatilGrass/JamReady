use crate::data::database::Database;
use crate::data::local_file_map::{LocalFile, LocalFileMap};
use crate::data::member::Member;
use crate::service::commands::database_sync::{sync_local, sync_remote};
use crate::service::commands::file_transmitter::{read_file, send_file};
use crate::service::jam_command::Command;
use async_trait::async_trait;
use jam_ready::utils::local_archive::LocalArchive;
use tokio::net::TcpStream;
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

        // 文件目录
        let file_path_str = args[1].to_string();
        let file = database.search_file(file_path_str.clone());
        if let Some(file) = file {
            if let Some(client_path) = file.client_path() {

                let mut ready = true;

                // 如果文件存在且版本大于或等于服务端的版本，则不下载
                if let Some(local_uuid) = local.file_uuids.get(&file.path()) {
                    if let Some(local_file) = local.file_paths.get(local_uuid) {
                        if local_file.local_version >= file.version() && client_path.exists() {
                            // 发送 "未就绪"
                            send_msg(stream, &ClientMessage::NotReady).await;
                            ready = false;
                        }
                    }
                }

                if ready {
                    // 发送 "就绪"
                    send_msg(stream, &ClientMessage::Ready).await;

                    // 读取文件
                    match read_file(stream, client_path).await {
                        Ok(_) => {
                            // 写入本地文件映射
                            if let Some(uuid) = database.uuid_of_path(file.path()) {
                                local.file_paths.insert(uuid.clone(), LocalFile {
                                    local_path: file.path(),
                                    local_version: file.version(),
                                });
                                local.file_uuids.insert(file.path(), uuid.clone());
                            }
                        }
                        Err(_) => { }
                    }
                }
            }
        }

        // 读取结束消息
        let _ = read_msg::<ServerMessage>(stream).await;
        LocalFileMap::update(&local);
    }

    async fn remote(&self, stream: &mut TcpStream, args: Vec<&str>, (_uuid, _member): (String, &Member), database: &mut Database) -> bool {

        // 首先同步数据库
        sync_remote(stream, database).await;

        // 检查参数数量
        if args.len() < 2 { return false; } // <搜索>

        // 确认客户端的准备状态
        let read_message: ClientMessage = read_msg(stream).await;
        match read_message {
            ClientMessage::Ready => {
                // 文件路径
                let file_path_str = args[1].to_string();
                let file = database.search_file(file_path_str.clone());
                if let Some(file) = file {
                    if let Some(server_path) = file.server_path() {
                        match send_file(stream,server_path).await {
                            Ok(_) => { }
                            Err(_) => { }
                        }
                    }
                }
            }
            _ => { }
        }

        // 完成信号
        send_msg(stream, &ServerMessage::Done).await;
        false
    }
}