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
        let database = Database::read();
        let mut local = LocalFileMap::read();

        // 计数器
        let mut all_count = 0;
        let mut success_count = 0;

        // 文件表
        let mut success_files = Vec::new();
        let mut failed_files = Vec::new();

        // 加载工作区
        let workspace = Workspace::read();
        if let Some(client) = workspace.client {

            // 寻找数据库内自己锁定的文件
            for file in database.files() {
                if let Some(owner_uuid) = file.get_locker_owner_uuid() {

                    // 若所有者的 Uid 和自己相同
                    if owner_uuid.trim() != client.uuid.trim() { continue; }

                    // 尝试寻找本地映射，找不到用默认映射

                    // 若此文件对应的本地位置存在
                    if let Some(client_path) = local.file_to_path(&database, file) {
                        if !client_path.exists() { continue; }

                        // 当前的 Md5
                        let current_md5= md5_digest(client_path.clone()).unwrap_or("".to_string());

                        // 本地文件
                        let local_file = local.file_paths.get(&database.uuid_of_path(file.path()).unwrap_or("".to_string()));

                        // 确认版本和MD5，以确保文件是最新版且已更改的
                        if let Some(local_file) = local_file {

                            // 如果远程的版本号为0则可以放通，因为是第一次提交
                            if file.version() > 0 {
                                // 若 MD5 一致，说明未产生修改
                                let record_md5 = &local_file.local_digest;
                                if record_md5 == &current_md5 {
                                    continue;
                                }
                                // 若版本号不一致，说明修改时版本未同步，不允许
                                if local_file.local_version != file.version() {
                                    continue;
                                }
                            }
                        } else {
                            // 若本地映射不存在，判断远端是否为版本0
                            // 若远端存在版本，则说明本地的文件和远端不同步，需要先下载
                            if file.version() > 0 {
                                continue;
                            }
                        }

                        // 确认要上传，加入计数器
                        all_count += 1;

                        // 记录该文件目录
                        let record_file_path = process_path_text(client_path.clone().display().to_string());

                        // 告知服务器要开始上传，并征求同意
                        send_msg(stream, &Text(file.path())).await;
                        match read_msg::<ServerMessage>(stream).await {
                            Pass => {
                                // 通过，则开始上传
                                let result = send_file(stream, client_path.clone()).await;
                                match result {
                                    Ok(_) => {

                                        // 成功，增加成功计数
                                        success_count += 1;

                                        // 记录文件到成功表
                                        success_files.push(record_file_path);

                                        // 更新本地映射
                                        if let Some(local_file_uuid) = local.file_uuids.get(&file.path()) {
                                            if let Some(local_file) = local.file_paths.get_mut(local_file_uuid) {

                                                // 提交成功后版本 +1
                                                local_file.local_version += 1;
                                                local_file.local_digest = current_md5;
                                            }
                                        } else {
                                            // 没有映射就创建
                                            if let Some(uuid) = database.uuid_of_path(file.path()) {
                                                local.file_paths.insert(uuid.clone(), LocalFile {
                                                    local_path: file.path(),
                                                    local_version: file.version() + 1, // 成功后应当从当前版本向前推进1
                                                    local_digest: current_md5,
                                                });
                                                local.file_uuids.insert(file.path(), uuid);
                                            }
                                        }
                                        LocalFileMap::update(&local);
                                    }
                                    Err(_) => {

                                        // 记录文件到失败表
                                        failed_files.push(record_file_path);
                                    }
                                }
                            }
                            Deny(_) => {

                                // 记录文件到失败表
                                failed_files.push(record_file_path);
                                continue;
                            }
                            _ => {

                                // 记录文件到失败表
                                failed_files.push(record_file_path);
                            }
                        }
                    }
                }
            }

            // 打印计数器
            if all_count > 0 {
                if success_count >= all_count {
                    println!("Ok: Commited {} files", all_count);
                } else {
                    eprintln!("Err: Commited {} files, Success {} files.", all_count, success_count);
                }

                // 打印成功的文件
                if success_files.len() > 0 {
                    println!("Success: ");
                    for success_file in success_files {
                        println!("{}", success_file);
                    }
                }

                // 打印失败的文件
                if failed_files.len() > 0 {
                    eprintln!("Failed: ");
                    for failed_file in failed_files {
                        eprintln!("{}", failed_file);
                    }
                }
            } else {

                // 无文件
                eprintln!("Err: No files committed.");
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

                                // 若不是长期锁，则直接丢弃
                                // (**不是我要的长期锁，直接丢弃**)
                                if !file.is_longer_lock_unchecked() {
                                    file.throw_locker();
                                }

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