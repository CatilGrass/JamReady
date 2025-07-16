use std::collections::HashMap;
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use jam_ready::utils::local_archive::LocalArchive;

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
    pub local_version: u32
}

impl LocalArchive for LocalFileMap {
    type DataType = LocalFileMap;

    fn relative_path() -> String {
        env!("FILE_LOCAL_FILE_MAP").to_string()
    }
}