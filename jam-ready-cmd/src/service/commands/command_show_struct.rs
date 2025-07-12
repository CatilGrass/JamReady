use std::collections::BTreeMap;
use async_trait::async_trait;
use colored::Colorize;
use tokio::net::TcpStream;
use jam_ready::utils::local_archive::LocalArchive;
use crate::data::database::Database;
use crate::data::member::Member;
use crate::data::workspace::Workspace;
use crate::service::commands::database_sync::{sync_local, sync_remote};
use crate::service::jam_command::Command;

pub struct ShowFileStructCommand;

#[async_trait]
impl Command for ShowFileStructCommand {

    async fn local(&self, stream: &mut TcpStream, _args: Vec<&str>) {

        // 从服务端接收最新的同步
        sync_local(stream).await;

        // 读取本地同步的树
        let database = Database::read();
        let mut paths = Vec::new();

        if let Some(client) = Workspace::read().client {
            for file in database.files() {
                let mut info = format!("{} ", &file.path());

                // 版本
                info = format!("{}[{}]", info, file.version());

                // 是否为空
                if file.real_path().is_empty() {
                    info = format!("{}{}", info, "[Empty]".truecolor(128, 128, 128));
                }

                // 锁定状态
                if let Some(uuid) = file.get_locker_owner_uuid() {
                    // 自己锁定
                    if uuid == client.uuid.trim() {
                        info = format!("{}{}", info, "[Owned]".bright_green());
                    } else {
                        info = format!("{}{}", info, "[Locked]".bright_red());
                    }
                }
                paths.push(info)
            }
        }

        // 显示
        print!("{}", show_tree(paths));
    }

    async fn remote(&self, stream: &mut TcpStream, _args: Vec<&str>, _member: (String, &Member), database: &mut Database) -> bool {

        // 发送数据
        sync_remote(stream, database).await;
        false
    }
}

/// 显示文件树
fn show_tree(paths: Vec<String>) -> String {
    #[derive(Default)]
    struct Node {
        is_file: bool,
        children: BTreeMap<String, Node>,
    }

    let mut root = Node::default();

    for path in paths {
        let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        let mut current = &mut root;

        // 遍历路径
        for (i, part) in parts.iter().enumerate() {

            // 判断是否为路径的最后一部分
            let is_file = i == parts.len() - 1;
            let child = current.children.entry((*part).to_string())
                .or_insert_with(Node::default);

            // 如果是文件
            if is_file {

                // 标记为文件
                child.is_file = true;
            }
            current = child;
        }
    }

    // 生成树形结构的文本
    fn generate_tree_lines(children: &BTreeMap<String, Node>, indent: usize) -> Vec<String> {
        let mut lines = Vec::new();
        let indent_str = "  ".repeat(indent * 2);

        // 遍历所有有子节点的节点
        for (name, node) in children.iter() {

            // 如果有子节点，则当作目录处理
            if !node.children.is_empty() {

                // 目录添加斜杠
                lines.push(format!("{}{}/", indent_str, name));

                // 递归处理子节点
                lines.extend(generate_tree_lines(&node.children, indent + 1));
            }
        }

        // 处理所有文件节点
        for (name, node) in children.iter() {

            // 只输出文件节点
            if node.is_file {

                // 文件节点不添加斜杠
                lines.push(format!("{}{}", indent_str, name));
            }
        }

        lines
    }

    // 从根节点的子节点开始生成
    generate_tree_lines(&root.children, 0).join("\n")
}