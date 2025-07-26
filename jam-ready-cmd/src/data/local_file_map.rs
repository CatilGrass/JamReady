use crate::data::database::{Database, VirtualFile};
use bincode::{Decode, Encode};
use jam_ready::utils::local_archive::LocalArchive;
use jam_ready::utils::text_process::process_path_text;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env::current_dir;
use std::path::PathBuf;

/// 本地文件映射
#[derive(Default, Serialize, Deserialize, Encode, Decode, Clone, Debug, PartialEq)]
pub struct LocalFileMap {

    /// Uuid 和本地路径的映射
    pub file_paths: HashMap<String, LocalFile>,

    /// 路径 和 Uuid 的映射
    pub file_uuids: HashMap<String, String>
}

#[derive(Default, Serialize, Deserialize, Encode, Decode, Clone, Debug, PartialEq)]
pub struct LocalFile {

    /// 本地路径
    pub local_path: String,

    /// 本地持有的版本
    pub local_version: u32,

    /// 本地文件摘要
    pub local_digest: String,
}

impl LocalArchive for LocalFileMap {
    type DataType = LocalFileMap;

    fn relative_path() -> String {
        env!("FILE_LOCAL_FILE_MAP").to_string()
    }
}

impl LocalFileMap {

    /// 从数据库的搜索中获得本地文件的 LocalFile
    pub fn search_to_local(&self, database: &Database, search: String) -> Option<&LocalFile> {

        // 尝试拿到 VirtualFile
        let file = database.search_file(search);
        if file.is_none() { return None; }
        let file = file.unwrap();

        // 从 VirtualFile 中拿到 LocalFile
        if let Some(uuid) = database.uuid_of_path(file.path()) {
            return self.file_paths.get(&uuid);
        }
        None
    }

    /// 从数据库的搜索中获得本地文件的 PathBuf
    pub fn search_to_path(&self, database: &Database, search: String) -> Option<PathBuf> {
        let result = self.search_to_local(database, search);
        if let Some(local_file) = result {
            if let Ok(current_dir) = current_dir() {
                let path = current_dir.join(local_file.local_path.clone());
                return Some(path);
            }
            None
        } else {
            None
        }
    }

    /// 从 VirtualFile 获得本地文件的 PathBuf
    pub fn file_to_path(&self, database: &Database, file: &VirtualFile) -> Option<PathBuf> {
        let local_file = self.search_to_path(&database, file.path());
        let mut client_path = None;
        if let Some(local_file) = local_file {
            client_path = Some(local_file);
        } else {
            if let Some(client_path_found) = file.client_path() {
                client_path = Some(client_path_found);
            }
        }
        client_path
    }

    /// 从数据库的搜索中获得本地文件的 PathBuf (相对路径)
    pub fn search_to_path_relative(&self, database: &Database, search: String) -> Option<PathBuf> {
        let result = self.search_to_local(database, search);
        if let Some(local_file) = result {
            let path = PathBuf::new().join(local_file.local_path.clone());
            Some(path)
        } else {
            None
        }
    }

    /// 从本地文件的 相对路径 中获得 Uuid
    pub fn local_path_to_uuid(&self, path: String) -> Option<&String> {
        let path = process_path_text(path);
        self.file_uuids.get(&path)
    }
}