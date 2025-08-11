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
use crate::data::database::{Database, VirtualFile};
use crate::data::local_file_map::LocalFileMap;
use crate::data::member::Member;
use crate::data::workspace::{ClientWorkspace, Workspace};
use crate::service::commands::database_sync::{sync_local, sync_remote};
use crate::service::jam_command::Command;

const REMOTE_ENV_FLAG: char = 'r';
const LOCAL_ENV_FLAG: char = 'l';
const ZERO_VERSION_FLAG: char = 'z';
const UPDATED_FLAG: char = 'u';
const OTHER_FLAG: char = 'e';
const MOVED_FLAG: char = 'm';
const HELD_FLAG: char = 'h';
const OTHER_LOCK_FLAG: char = 'g';
const UNTRACKED_FLAG: char = 'n';
const REMOVED_FLAG: char = 'd';

pub struct ShowFileStructCommand;

#[async_trait]
impl Command for ShowFileStructCommand {

    async fn local(&self, stream: &mut TcpStream, args: Vec<&str>) {

        // 参数检查
        if args.len() < 3 {
            return;
        }

        sync_local(stream).await;

        let database = Database::read().await;
        let local = LocalFileMap::read().await;
        let env = args[1];
        let switches = args[2];
        let mut paths = Vec::new();

        if let Some(client) = Workspace::read().await.client {

            let show_remote = env.contains(REMOTE_ENV_FLAG);
            let show_local = env.contains(LOCAL_ENV_FLAG);

            let show_zero_version = switches.contains(ZERO_VERSION_FLAG);
            let show_updated = switches.contains(UPDATED_FLAG);
            let show_other = switches.contains(OTHER_FLAG);
            let show_moved = switches.contains(MOVED_FLAG);
            let show_held = switches.contains(HELD_FLAG);
            let show_other_lock = switches.contains(OTHER_LOCK_FLAG);
            let show_untracked = switches.contains(UNTRACKED_FLAG);
            let show_removed = switches.contains(REMOVED_FLAG);

            // 处理工作区文件
            if show_remote {
                for file in database.files() {
                    if let Some(info) = build_remote_file_info(
                        &file, &database, &local, &client,
                        show_zero_version, show_updated, show_other, show_moved,
                        show_held, show_other_lock
                    ) {
                        paths.push(info);
                    }
                }
            }

            // 处理本地文件
            if show_local {
                paths.extend(get_local_file_info(
                    &local, &database,
                    show_moved, show_removed, show_untracked
                ));
            }
        }

        println!("{}", show_tree(paths));
    }

    async fn remote(&self, stream: &mut TcpStream, _args: Vec<&str>, _member: (String, &Member), database: Arc<Mutex<Database>>) {
        entry_mutex_async!(database, |guard| {
            sync_remote(stream, guard).await;
        });
    }
}

fn build_remote_file_info(
    file: &VirtualFile,
    database: &Database,
    local: &LocalFileMap,
    client: &ClientWorkspace,
    show_zero_version: bool,
    show_updated: bool,
    show_other: bool,
    show_moved: bool,
    show_held: bool,
    show_other_lock: bool
) -> Option<String> {
    let mut info = file.path().to_string();
    let mut should_display = false;
    let local_file = local.search_to_local(database, file.path());

    // 空文件
    if file.real_path().is_empty() {
        if show_zero_version {
            info.push_str(&format!(" {}", "[Empty]".truecolor(128, 128, 128)));
            should_display = true;
        }
    }
    // 本地文件存在
    else if let (Ok(current_dir), Some(local_file)) = (current_dir(), local_file) {
        let local_path = current_dir.join(&local_file.local_path);

        if local_path.exists() {
            let local_version = local_file.local_version;
            let file_version = file.version();

            // 版本
            match local_version.cmp(&file_version) {
                std::cmp::Ordering::Less if show_updated => {
                    info.push_str(&format!(" {}", format!("[v{}↓]", local_version).bright_red()));
                    should_display = true;
                },
                std::cmp::Ordering::Greater if show_updated => {
                    info.push_str(&format!(" {}", format!("[v{}↑]", local_version).bright_red()));
                    should_display = true;
                },
                std::cmp::Ordering::Equal if show_other => {
                    info.push_str(&format!(" {}", format!("[v{}]", local_version).bright_green()));
                    should_display = true;
                },
                _ => {}
            }

            // 文件移动
            if show_moved && local_file.local_path != file.path() {
                let formatted_path = local_file.local_path.replace("/", "\\");
                info.push_str(&format!(" {}", format!("-> {}", formatted_path).truecolor(128, 128, 128)));
            }
        }
    }
    // 本地文件不存在
    else {
        if show_other {
            should_display = true;
        }
    }

    // 文件锁定
    if let Some(locker_uuid) = file.get_locker_owner_uuid() {
        let is_long_lock = file.is_longer_lock_unchecked();
        let is_held = locker_uuid == client.uuid.trim();
        let is_other_lock = locker_uuid != client.uuid.trim();

        if is_held && show_held {
            let lock_tag = if is_long_lock { "[HELD]".bright_green() } else { "[held]".green() };
            info.push_str(&format!(" {}", lock_tag));
            should_display = true;
        }
        if is_other_lock && show_other_lock {
            let lock_tag = if is_long_lock { "[LOCKED]".bright_red() } else { "[locked]".bright_yellow() };
            info.push_str(&format!(" {}", lock_tag));
            should_display = true;
        }
    }

    if should_display { Some(info) } else { None }
}

fn get_local_file_info(
    local: &LocalFileMap,
    database: &Database,
    show_moved: bool,
    show_removed: bool,
    show_untracked: bool
) -> Vec<String> {
    let mut paths : Vec<String> = Vec::new();
    let workspace_root = env!("PATH_WORKSPACE_ROOT");

    for path in get_all_file_paths() {

        // 跳过工作区配置目录
        if path.starts_with(workspace_root) {
            continue;
        }

        match local.file_uuids.get(&path) {
            Some(uuid) => {
                if let Some(file) = database.file_with_uuid(uuid.clone()) {
                    // 路径变动时
                    if file.path() != path {
                        // 路径不为空且显示移动
                        if !file.path().is_empty() && show_moved {
                            // 文件移动
                            let moved_info = format!(
                                "{} {} {}",
                                path,
                                "[Moved]".yellow(),
                                format!("-> {}", file.path().replace("/", "\\")).truecolor(128, 128, 128)
                            );
                            paths.push(format!("{}", moved_info));
                        }
                        // 路径为空且显示移除
                        if file.path().is_empty() && show_removed {
                            // 文件移除
                            paths.push(format!(
                                "{} {} {}",
                                path,
                                "[Removed]".red(),
                                uuid.red()
                            ));
                        }
                    }
                }
            }
            None if show_untracked => {
                // 未追踪
                paths.push(format!(
                    "{} {}",
                    path,
                    "[Untracked]".cyan()
                ));
            }
            _ => {}
        }
    }

    paths
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