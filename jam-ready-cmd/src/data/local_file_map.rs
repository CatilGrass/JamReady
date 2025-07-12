use std::collections::HashMap;
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use jam_ready::utils::local_archive::LocalArchive;

/// 本地文件映射
#[derive(Default, Serialize, Deserialize, Encode, Decode, Clone, Debug, PartialEq)]
pub struct LocalFileMap {
    pub file_uuids: HashMap<String, String>,
    pub file_paths: HashMap<String, String>
}

impl LocalArchive for LocalFileMap {
    type DataType = LocalFileMap;

    fn relative_path() -> String {
        env!("FILE_LOCAL_FILE_MAP").to_string()
    }
}