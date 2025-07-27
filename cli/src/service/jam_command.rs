use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use colored::Colorize;
use log::info;
use tokio::net::TcpStream;
use crate::data::database::Database;
use crate::data::member::Member;

pub type CommandRegistry = HashMap<&'static str, Arc<dyn Command + Send + Sync>>;

#[async_trait]
pub trait Command {

    /// 客户端上的操作
    async fn local(&self, stream: &mut TcpStream, args: Vec<&str>);

    /// 服务器上的操作 (返回是否修改过数据库)
    async fn remote(&self, stream: &mut TcpStream, args: Vec<&str>, member: (String, &Member), database: &mut Database) -> bool;
}

/// 执行本地命令
pub async fn execute_local_command(
    registry: &CommandRegistry,
    stream: &mut TcpStream,
    args: Vec<&str>,
) {
    if let Some(command_name) = args.get(0) {
        let command_name = command_name.trim().to_lowercase();
        if let Some(command) = registry.get(command_name.as_str()) {

            // 执行命令
            command.local(stream, args).await;
        } else {
            eprintln!("Unknown command: {}", command_name);
        }
    }
}

/// 执行远程命令
pub async fn execute_remote_command(
    registry: &CommandRegistry,
    stream: &mut TcpStream,
    args: Vec<&str>,
    (uuid, member): (String, &Member),
    database: &mut Database
) -> bool {
    info!("{} Exec {}", &member.member_name.yellow(), format!("{:?}", &args).cyan());

    if let Some(command_name) = args.get(0) {
        let command_name = command_name.trim().to_lowercase();
        if let Some(command) = registry.get(command_name.as_str()) {

            // 执行命令
            let changed = command.remote(stream, args, (uuid, member), database).await;
            return changed
        } else {
            eprintln!("Unknown command: {}", command_name);
            return false
        }
    }
    false
}