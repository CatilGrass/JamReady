use crate::data::database::{Database, VirtualFile};
use crate::data::local_file_map::{LocalFile, LocalFileMap};
use crate::data::member::Member;
use crate::service::commands::database_sync::{sync_local, sync_remote};
use crate::service::jam_command::Command;
use crate::service::messages::ServerMessage;
use crate::service::messages::ServerMessage::{Deny, Text};
use crate::service::service_utils::{read_msg, send_msg};
use async_trait::async_trait;
use jam_ready::utils::file_digest::md5_digest;
use jam_ready::utils::local_archive::LocalArchive;
use jam_ready::utils::text_process::process_path_text;
use std::env::current_dir;
use colored::Colorize;
use tokio::net::TcpStream;

pub struct FileOperationCommand;

#[async_trait]
impl Command for FileOperationCommand {

    async fn local(&self, stream: &mut TcpStream, args: Vec<&str>) {

        // 检查参数数量
        if args.len() < 3 { return; } // <操作符> <地址> <地址2: 可选>

        // 检查服务器返回的消息
        let message: ServerMessage = read_msg(stream).await;
        match message {
            Text(msg) => {
                // 成功后，从服务端接收最新的同步
                sync_local(stream).await;
                println!("Ok: {}", msg)
            }
            Deny(msg) => {
                // 失败后，不处理后续事项
                eprintln!("Err: {}", msg);
                return;
            }
            _ => {
                return;
            }
        }

        // 检查操作符
        match args[1].to_lowercase().trim() {

            // 添加文件 (若创建的文件地址在本地存在，则为其建立映射)
            "add" => {

                // 加载本地数据
                let mut local = LocalFileMap::read();
                let database = Database::read();

                let search = args[2];
                if let Ok(current) = current_dir() {
                    let local_file_path_buf = current.join(search);

                    // 本地确实存在该文件
                    if local_file_path_buf.exists() {

                        // 且远程确实存在该文件 (因为刚创建的所以大概率存在)
                        if let Some(file) = database.search_file(search.to_string()) {
                            let file_path = file.path();
                            if let Some(file_uuid) = database.uuid_of_path(file_path.clone()) {
                                local.file_uuids.insert(file_path, file_uuid.clone());
                                local.file_paths.insert(file_uuid, LocalFile{
                                    local_path: search.to_string(),
                                    local_version: file.version(),
                                    local_digest: md5_digest(local_file_path_buf).unwrap_or("".to_string()),
                                });
                            }
                        }
                    } else {
                        // 不存在本地文件，通知成员需要将文件存储到哪
                        println!("You created a virtual file, but the file does not exist locally.");
                        println!("Please save the completed file to the following path:");
                        println!("{}", local_file_path_buf.display().to_string().green());
                    }
                }

                // 保存本地数据
                LocalFileMap::update(&local);
            }
            _ => { }
        }
    }

    async fn remote(
        &self,
        stream: &mut TcpStream, args: Vec<&str>,
        (uuid, _member): (String, &Member), database: &mut Database) -> bool {

        // 检查参数数量
        if args.len() < 3 { return false; } // <操作符> <搜索/地址> <地址2: 可选>

        let operation_str = args[1];
        let input = args[2];

        // 搜索得到虚拟文件
        let virtual_file = database.search_file_mut(input.to_string());

        match operation_str.to_lowercase().trim() {

            // 添加文件
            "add" => {

                // 文件未存在时
                if let None = virtual_file {
                    if let Ok(success) = database.insert_virtual_file(
                        VirtualFile::new(input.to_string())) {
                        if success {

                            // 成功
                            send_msg(stream, &Text(format!("Virtual file \"{}\" created.", args[2]))).await;

                            // 发送同步
                            sync_remote(stream, database).await;
                            return true;
                        }
                    }
                }

                // 失败
                send_msg(stream, &Deny("Failed to create virtual file.".to_string())).await;
                false
            }

            // 移除文件 (仅移除映射)
            "remove" => {

                // 文件存在时
                if let Some(file) = virtual_file {

                    // 检查锁定情况
                    if !is_available(file, stream, uuid).await {
                        return false;
                    }

                    // 移除文件映射
                    if let Ok(_uuid) = database.remove_file_map(process_path_text(args[2].to_string())) {

                        // 成功
                        send_msg(stream, &Text(format!("Removed virtual file \"{}\".", args[2]))).await;

                        // 发送同步
                        sync_remote(stream, database).await;
                        return true;
                    }

                    send_msg(stream, &Deny("Remove virtual file failed!".to_string())).await;
                    false
                } else {

                    send_msg(stream, &Deny("Remove virtual file failed!".to_string())).await;
                    false
                }
            }

            // 移动文件 (重建映射)
            "move" => {

                // 再次检查参数，若缺少第三个参数，则失败
                if args.len() < 4 {
                    send_msg(stream, &Deny("Failed to move the file: Please specify the destination address.".to_string())).await;
                    return false;
                }

                // 移动到的地址
                let move_to_path = process_path_text(args[3].to_string());

                // 文件存在时
                if let Some(file) = virtual_file {

                    // 检查锁定情况
                    if !is_available(file, stream, uuid).await {
                        return false;
                    }

                    // 尝试以目录移动
                    if let Ok(()) = database.move_file(args[2].to_string(), move_to_path.clone()) {

                        // 成功
                        send_msg(stream, &Text(format!("Moved \"{}\" to \"{}\" success.", args[2], args[3]))).await;

                        // 发送同步
                        sync_remote(stream, database).await;
                        return true;
                    }

                    // 尝试以 Uuid 移动
                    if let Ok(()) = database.move_file_with_uuid(args[2].to_string(), move_to_path) {

                        // 成功
                        send_msg(stream, &Text(format!("Moved uuid \"{}\" to \"{}\" success.", args[2], args[3]))).await;

                        // 发送同步
                        sync_remote(stream, database).await;
                        return true;
                    }
                }

                // 失败
                send_msg(stream, &Deny("Move file failed!".to_string())).await;
                false
            }

            // 拿到文件的锁
            "get" => {

                // 文件存在
                if let Some(file) = virtual_file {

                    // 尝试拿锁
                    return if file.give_uuid_locker(uuid, false) {

                        // 成功
                        send_msg(stream, &Text(format!("Get locker of \"{}\" success.", args[2]))).await;

                        // 发送同步
                        sync_remote(stream, database).await;
                        true
                    } else {
                        send_msg(stream, &Deny("Get locker failed!".to_string())).await;
                        false
                    }
                }

                send_msg(stream, &Deny("Get locker failed!".to_string())).await;
                false
            }

            // 拿到文件的锁 (长期)
            "get_longer" => {

                // 文件存在
                if let Some(file) = virtual_file {

                    // 尝试拿锁
                    return if file.give_uuid_locker(uuid, true) {

                        // 成功
                        send_msg(stream, &Text(format!("Get longer locker of \"{}\" success.", args[2]))).await;

                        // 发送同步
                        sync_remote(stream, database).await;
                        true
                    } else {
                        send_msg(stream, &Deny("Get longer locker failed!".to_string())).await;
                        false
                    }
                }

                send_msg(stream, &Deny("Get longer locker failed!".to_string())).await;
                false
            }

            // 丢掉文件的锁
            "throw" => {

                // 文件存在
                if let Some(file) = virtual_file {

                    // 获得文件的锁持有者
                    if let Some((owner_uuid, _)) = file.get_locker_owner() {

                        // 如果是自己则丢掉锁
                        return if uuid == owner_uuid {
                            file.throw_locker();

                            // 成功
                            send_msg(stream, &Text(format!("Throw the locker of \"{}\" success.", args[2]))).await;

                            // 发送同步
                            sync_remote(stream, database).await;
                            true
                        } else {

                            // 其他人持有
                            send_msg(stream, &Deny("You cannot throw the locker held by others.".to_string())).await;
                            false
                        }
                    }
                }

                send_msg(stream, &Deny("Throw locker failed!".to_string())).await;
                false
            }

            _ => {
                send_msg(stream, &Deny(format!("No operation named \"{}\" exists.", operation_str))).await;
                false
            }
        }
    }
}

/// 检查锁定情况
async fn is_available(file: &VirtualFile, stream: &mut TcpStream, self_uuid: String) -> bool {
    // 获得文件的锁持有者
    if let Some((owner_uuid, _)) = file.get_locker_owner() {

        // 其他人拿到锁
        if self_uuid != owner_uuid {
            send_msg(stream, &Deny("The file has been locked by another team member!".to_string())).await;
            return false;
        }
    } else {

        // 没人拿到锁
        send_msg(stream, &Deny("Before operating on the file, please \"get\" it first!".to_string())).await;
        return false;
    }
    true
}