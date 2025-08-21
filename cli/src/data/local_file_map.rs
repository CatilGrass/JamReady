use crate::data::database::{Database, VirtualFile};
use bincode::{Decode, Encode};
use jam_ready::utils::local_archive::LocalArchive;
use jam_ready::utils::text_process::process_path_text;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env::current_dir;
use std::path::PathBuf;

/// Local file mapping
#[derive(Default, Serialize, Deserialize, Encode, Decode, Clone, Debug, PartialEq)]
pub struct LocalFileMap {
    /// Uuid to local path mapping
    #[serde(rename = "files")]
    pub file_paths: HashMap<String, LocalFile>,

    /// Path to Uuid mapping
    #[serde(rename = "uuids")]
    pub file_uuids: HashMap<String, String>
}

#[derive(Default, Serialize, Deserialize, Encode, Decode, Clone, Debug, PartialEq)]
pub struct LocalFile {
    /// Local path
    #[serde(rename = "path")]
    pub local_path: String,

    /// Local version
    #[serde(rename = "version")]
    pub local_version: u32,

    /// File digest
    #[serde(rename = "digest")]
    pub local_digest: String,

    /// Is the file completed
    #[serde(rename = "cmpl")]
    pub completed: bool,

    /// Summary information when the file is completed
    #[serde(rename = "cmpl_digest")]
    pub completed_digest: String,

    /// File commit information
    #[serde(rename = "cmpl_commit")]
    pub completed_commit: String,
}

impl LocalArchive for LocalFileMap {
    type DataType = LocalFileMap;

    fn relative_path() -> String {
        env!("FILE_LOCAL_FILE_MAP").to_string()
    }
}

impl LocalFileMap {
    /// Get LocalFile from database search
    pub fn search_to_local(&self, database: &Database, search: String) -> Option<&LocalFile> {
        // Try to get VirtualFile
        let file = database.search_file(search);
        if file.is_none() { return None; }
        let file = file.unwrap();

        // Get LocalFile from VirtualFile
        if let Some(uuid) = database.uuid_of_path(file.path()) {
            return self.file_paths.get(&uuid);
        }
        None
    }

    /// Get mutable LocalFile from database search
    pub fn search_to_local_mut(&mut self, database: &Database, search: String) -> Option<&mut LocalFile> {
        // Try to get VirtualFile
        let file = database.search_file(search);
        if file.is_none() { return None; }
        let file = file.unwrap();

        // Get LocalFile from VirtualFile
        if let Some(uuid) = database.uuid_of_path(file.path()) {
            return self.file_paths.get_mut(&uuid);
        }
        None
    }

    /// Get PathBuf from database search
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

    /// Get PathBuf from VirtualFile
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

    /// Get relative PathBuf from database search
    pub fn search_to_path_relative(&self, database: &Database, search: String) -> Option<PathBuf> {
        let result = self.search_to_local(database, search);
        if let Some(local_file) = result {
            let path = PathBuf::new().join(local_file.local_path.clone());
            Some(path)
        } else {
            None
        }
    }

    /// Get Uuid from relative local path
    pub fn local_path_to_uuid(&self, path: String) -> Option<&String> {
        let path = process_path_text(path);
        self.file_uuids.get(&path)
    }
}