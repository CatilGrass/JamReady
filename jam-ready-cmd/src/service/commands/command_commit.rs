use std::time::Duration;
use crate::data::database::Database;
use crate::data::member::Member;
use crate::data::workspace::Workspace;
use crate::service::commands::database_sync::{sync_local, sync_remote};
use crate::service::jam_command::Command;
use async_trait::async_trait;
use log::{error, info};
use jam_ready::utils::local_archive::LocalArchive;
use tokio::net::TcpStream;
use tokio::select;
use tokio::time::sleep;
use uuid::Uuid;
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
        let database = Database::read();

        // 加载工作区
        let workspace = Workspace::read();
        if let Some(client) = workspace.client {

            // 寻找数据库内自己锁定的文件
            for file in database.files() {
                if let Some(owner_uuid) = file.get_locker_owner_uuid() {

                    // 若所有者的 Uid 和自己相同
                    if owner_uuid.trim() != client.uuid.trim() { continue; }

                    // 若此文件对应的本地位置存在
                    if let Some(client_path) = file.client_path() {
                        if !client_path.exists() { continue; }

                        // 告知服务器要开始上传，并征求同意
                        send_msg(stream, &Text(file.path())).await;
                        match read_msg::<ServerMessage>(stream).await {
                            Pass => {
                                // 通过，则开始上传
                                let result = send_file(stream, client_path.clone()).await;
                                match result {
                                    Ok(_) => {
                                        print!("Commit \"{}\" SUCCESS", client_path.display());
                                    }
                                    Err(_) => {
                                        eprint!("Commit \"{}\" FAILED", client_path.display());
                                    }
                                }
                            }
                            Deny(_) => {
                                eprintln!("Commit \"{}\" FAILED", file.path());
                                continue;
                            }
                            _ => {}
                        }
                    }
                }
            }
            // 所有操作完成后，发送一个结束消息
            send_msg(stream, &Done).await
        }
    }

    async fn remote(
        &self, stream: &mut TcpStream, args: Vec<&str>,
        (uuid, _member): (String, &Member), database: &mut Database)
        -> bool {

        let mut changed = false;

        // 判断是否存在描述
        let commit_description = if args.len() >= 2 { args[1] } else { "Update" };

        // 首先同步数据库
        sync_remote(stream, database).await;

        // 客户端会持续发来待上传的地址
        loop {
            select! {
                // 单次传输中 3 秒没有新文件加入，就会结束传输
                _ = sleep(Duration::from_secs(3)) => {
                    break;
                }

                // 接收到新消息
                msg = read_msg::<ClientMessage>(stream) => {
                    if msg == Unknown || msg == Done {
                        break;
                    } else {
                        if process_remote_receive(stream, msg, database, &uuid, commit_description.to_string()).await {
                            changed = true;
                        }
                    }
                }
            }
        }

        changed
    }
}

async fn process_remote_receive(
    stream: &mut TcpStream, message: ClientMessage, database: &mut Database, uuid: &String, description: String)
    -> bool {

    // 读取客户端的消息
    if let Text(path) = message {

        // 检查此文件路径是否存在
        if let Some(file) = database.file_mut(path) {

            // 为文件分配一串新的 Uuid
            let real_file_uuid = Uuid::new_v4().to_string();

            // 获得文件真实地址
            if let Some(real_path) = file.server_path_temp(real_file_uuid.clone()) {

                // 检查此文件是否被锁定
                if let Some(owner_uuid) = file.get_locker_owner_uuid() {

                    // 是否为该成员锁定
                    if owner_uuid.trim() == uuid.trim() {

                        // 发送同意
                        send_msg(stream, &Pass).await;

                        // 开始接收此文件
                        let result = read_file(stream, real_path.clone()).await;
                        match result {
                            Ok(_) => {

                                // 成功接收，标记工作区更新，并将 Uuid 应用到文件
                                file.update(real_file_uuid, description.clone());
                                info!("Update file {}: \"{}\"", file.path(), description);

                                return true;
                            }
                            Err(err) => {
                                error!("Failed to update file \"{}\": {}", file.path(), err);
                            }
                        }
                    }
                }
            }
        }
    }

    // 失败
    send_msg(stream, &Deny("".to_string())).await;
    false
}