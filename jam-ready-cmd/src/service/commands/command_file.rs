use crate::data::database::{Database, VirtualFile};
use crate::data::member::Member;
use crate::service::commands::database_sync::{sync_local, sync_remote};
use crate::service::jam_command::Command;
use crate::service::messages::ServerMessage;
use crate::service::messages::ServerMessage::{Deny, Pass};
use crate::service::service_utils::{read_msg, send_msg};
use async_trait::async_trait;
use jam_ready::utils::text_process::process_path_text;
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
            Pass => {
                // 成功后，从服务端接收最新的同步
                sync_local(stream).await;

                println!("Ok");
            }
            Deny(msg) => {
                eprintln!("{}", msg)
            }
            _ => {}
        }
    }

    async fn remote(
        &self,
        stream: &mut TcpStream, args: Vec<&str>,
        (uuid, _member): (String, &Member), database: &mut Database) -> bool {

        // 检查参数数量
        if args.len() < 3 { return false; } // <操作符> <地址> <地址2: 可选>

        let operation_str = args[1];
        let virtual_file_str = args[2];

        let virtual_file = database.file_mut(virtual_file_str.to_string());

        match operation_str.to_lowercase().trim() {

            // 添加文件
            "add" => {

                // 文件未存在时
                if let None = virtual_file {
                    if let Ok(success) = database.insert_virtual_file(
                        VirtualFile::new(virtual_file_str.to_string())) {
                        if success {

                            // 成功
                            send_msg(stream, &Pass).await;

                            // 发送同步
                            sync_remote(stream, database).await;
                            return true;
                        }
                    }
                }

                // 失败
                send_msg(stream, &Deny("Failed to create file.".to_string())).await;
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
                        send_msg(stream, &Pass).await;

                        // 发送同步
                        sync_remote(stream, database).await;
                        return true;
                    }

                    send_msg(stream, &Deny("Remove file failed!".to_string())).await;
                    false
                } else {

                    send_msg(stream, &Deny("Remove file failed!".to_string())).await;
                    false
                }
            }

            // 移动文件
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

                    if let Ok(()) = database.move_file(process_path_text(args[2].to_string()), move_to_path) {

                        // 成功
                        send_msg(stream, &Pass).await;

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
                    return if file.give_uuid_locker(uuid) {

                        // 成功
                        send_msg(stream, &Pass).await;

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
                            send_msg(stream, &Pass).await;

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