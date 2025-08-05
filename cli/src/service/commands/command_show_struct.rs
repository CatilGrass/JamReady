use std::env::current_dir;
use std::sync::Arc;
use async_trait::async_trait;
use colored::Colorize;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use walkdir::WalkDir;
use jam_ready::entry_mutex_async;
use jam_ready::utils::local_archive::LocalArchive;
use jam_ready::utils::text_process::{process_path_text, show_tree};
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
        let database = Database::read().await;
        let local = LocalFileMap::read().await;
        let mut paths = Vec::new();

        if let Some(client) = Workspace::read().await.client {
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

                                // 对比版本 (无论新旧，只要文件版本不匹配则无法提交)
                                if local_version < file.version() {
                                    // 本地文件更旧，显示需要更新
                                    info = format!("{} {}", info, format!("[v{}↓]", local_version).bright_red());
                                } else if local_version > file.version() {
                                    // 本地文件更新，说明文件已回滚
                                    info = format!("{} {}", info, format!("[v{}↑]", local_version).bright_red());
                                } else {

                                    // 本地文件版本同步
                                    info = format!("{} {}", info, format!("[v{}]", local_version).bright_green());
                                }
                            }
                        }
                    }
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

            for path in get_all_file_paths() {
                if path.starts_with(env!("PATH_WORKSPACE_ROOT")) { continue }
                if let Some(uuid) = local.file_uuids.get(&path) {
                    if let Some(file) = database.file_with_uuid(uuid.clone()) {
                        if file.path() != path {
                            if file.path() == "" {
                                let info = format!("{} {} {}", &path, "[Removed]".red(), uuid.red());
                                paths.push(info);
                                continue;
                            }
                            let info = format!("{} {}", &path, "[Moved]".yellow());
                            paths.push(info);
                        }
                    }
                } else {
                    let info = format!("{} {}", &path, "[Untracked]".cyan());
                    paths.push(info);
                }
            }
        }

        // 显示
        print!("{}", show_tree(paths));
    }

    async fn remote(&self, stream: &mut TcpStream, _args: Vec<&str>, _member: (String, &Member), database: Arc<Mutex<Database>>) {

        // 发送数据
        entry_mutex_async!(database, |guard| {
            sync_remote(stream, guard).await;
        });
    }
}

fn get_all_file_paths() -> Vec<String> {
    WalkDir::new(".")
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .map(|e| {
            e.path()
                .strip_prefix(".")
                .unwrap()
                .to_string_lossy()
                .trim_start_matches('/')
                .to_string()
        })
        .map(process_path_text)
        .collect()
}