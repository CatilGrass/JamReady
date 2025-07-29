use std::collections::BTreeMap;
use std::env::current_dir;
use async_trait::async_trait;
use colored::Colorize;
use tokio::net::TcpStream;
use jam_ready::utils::local_archive::LocalArchive;
use crate::data::database::Database;
use crate::data::local_file_map::LocalFileMap;
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
        let local = LocalFileMap::read();
        let mut paths = Vec::new();

        if let Some(client) = Workspace::read().client {
            for file in database.files() {

                // 本地文件
                let local_file = local.search_to_local(&database, file.path());

                // 起始的
                let mut info = format!("{}", &file.path());

                // 是否为空
                if file.real_path().is_empty() {

                    // 显示空版本
                    info = format!("{} {}", info, "[Empty]".truecolor(128, 128, 128));
                } else {

                    // 若存在本地文件，显示本地版本
                    if let Some(local_file) = local_file {
                        if let Ok(current) = current_dir() {
                            let local_file_path_buf = current.join(&local_file.local_path);
                            if local_file_path_buf.exists() {

                                // 本地版本
                                let local_version = local_file.local_version;

                                // 对比版本
                                if local_version < file.version() {
                                    // 本地文件更旧，显示需要更新
                                    info = format!("{} {}", info, format!("[v{}↓]", local_version).bright_red());
                                } else {

                                    // 本地文件版本同步
                                    info = format!("{} {}", info, format!("[v{}]", local_version).bright_green());
                                }
                            }
                        }
                    }

                    // 本地不存在此文件时，不打印版本，因为并不关注
                    // if !added {
                    //     // 显示当前版本
                    //     info = format!("{} [v{}]", info, file.version());
                    // }
                }

                // 锁定状态
                if let Some(uuid) = file.get_locker_owner_uuid() {
                    let longer_lock = file.is_longer_lock_unchecked();
                    // 自己锁定
                    if uuid == client.uuid.trim() {
                        if longer_lock {
                            // 自己的长锁
                            info = format!("{} {}", info, "[HELD]".bright_green());
                        } else {
                            // 自己的短锁
                            info = format!("{} {}", info, "[held]".green());
                        }
                    } else {
                        if longer_lock {
                            // 他人的长锁
                            info = format!("{} {}", info, "[LOCKED]".bright_red());
                        } else {
                            // 他人的短锁
                            info = format!("{} {}", info, "[locked]".bright_yellow());
                        }
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
    fn generate_tree_lines(children: &BTreeMap<String, Node>, prefix: &str) -> Vec<String> {
        let mut lines = Vec::new();
        let last_index = children.len().saturating_sub(1);

        let child_prefix = format!("{}│   ", prefix);

        for (index, (name, node)) in children.iter().enumerate() {
            let is_last_child = index == last_index;
            let connector = if is_last_child { "└── " } else { "├── " };

            if !node.children.is_empty() {
                lines.push(format!("{}{}{}/", prefix, connector, name));

                if !node.children.is_empty() {
                    let child_lines = generate_tree_lines(
                        &node.children,
                        &child_prefix
                    );
                    lines.extend(child_lines);
                }
            } else {
                // 文件
                lines.push(format!("{}{}{}", prefix, connector, name));
            }
        }

        lines
    }

    // 从根节点的子节点开始生成
    generate_tree_lines(&root.children, "").join("\n")
}