use crate::data::database::Database;
use bincode::{Decode, Encode};
use jam_ready::utils::local_archive::LocalArchive;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use jam_ready::utils::text_process::split_path_text;

/// 本地文件映射
#[derive(Default, Serialize, Deserialize, Encode, Decode, Clone, Debug, PartialEq)]
pub struct LocalFolderMap {

    /// 目录完整路径和其中的文件信息映射
    pub folder_files: HashMap<String, Vec<Node>>,
}

/// 节点
#[derive(Default, Serialize, Deserialize, Encode, Decode, Clone, Debug, PartialEq)]
pub enum Node {

    #[default]
    Unknown,

    /// 跳转 (目录完整路径 Folder/)
    Jump(String),

    /// 文件 (完整路径 Folder/file.txt)
    /// 为什么不包含 Uuid? 该数据仅为提高客户端界面查询文件结构速度而存在，真正的数据存储在 database.yaml 中
    File(String)
}

impl LocalArchive for LocalFolderMap {
    type DataType = LocalFolderMap;

    fn relative_path() -> String {
        env!("FILE_LOCAL_FOLDER_MAP").to_string()
    }
}

impl From<&Database> for LocalFolderMap {

    fn from(database: &Database) -> Self {
        let mut folder_files: HashMap<String, Vec<Node>> = HashMap::new();
        let mut all_dirs = HashSet::new();

        // 处理所有文件并收集所有目录
        for file in database.files() {
            let file_path = file.path();
            let (dir_path, _) = split_path_text(&file_path);

            // 添加文件到对应目录
            folder_files.entry(dir_path.clone())
                .or_default()
                .push(Node::File(file_path.clone()));

            // 收集所有目录路径
            let mut current_dir = dir_path.clone();
            while !current_dir.is_empty() {
                all_dirs.insert(current_dir.clone());
                current_dir = get_parent_dir(&current_dir).unwrap_or_default();
            }
        }

        // 添加根目录
        all_dirs.insert("".to_string());

        // 构建目录结构
        for dir_path in &all_dirs {
            // 确保目录在映射中存在
            folder_files.entry(dir_path.clone()).or_default();

            // 获取所有直接的子目录
            let sub_dirs: Vec<_> = all_dirs.iter()
                .filter(|d| d.starts_with(dir_path) && *d != dir_path)
                .filter(|d| {
                    let relative = d.trim_start_matches(dir_path);
                    !relative.contains('/') || relative.matches('/').count() == 1
                })
                .cloned()
                .collect();

            // 添加 Jump 节点到子目录
            for sub_dir in sub_dirs {
                if let Some(nodes) = folder_files.get_mut(dir_path) {
                    if !nodes.contains(&Node::Jump(sub_dir.clone())) {
                        nodes.push(Node::Jump(sub_dir));
                    }
                }
            }
        }

        Self { folder_files }
    }
}

// 获取父目录路径
fn get_parent_dir(dir_path: &str) -> Option<String> {
    if dir_path.is_empty() {
        return None;
    }

    // 去掉结尾的斜杠
    let trimmed = dir_path.trim_end_matches('/');

    if let Some(last_idx) = trimmed.rfind('/') {
        // 返回父目录路径
        Some(trimmed[..=last_idx].to_string())
    } else {
        // 顶级目录的父目录是根目录
        Some("".to_string())
    }
}