use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env::current_dir;
use std::path::PathBuf;
use uuid::Uuid;
use jam_ready::utils::local_archive::LocalArchive;
use jam_ready::utils::text_process::process_path_text;
use crate::data::database::VirtualFileState::{Available, Lock};
use crate::data::member::Member;
use crate::data::workspace::Workspace;

/// # Database - File Database
/// The database is the file storage station in Jam Ready
#[derive(Serialize, Deserialize, Encode, Decode, Clone, Debug, PartialEq)]
pub struct Database {
    /// All files (Uid, File)
    #[serde(rename = "files")]
    virtual_files: HashMap<String, VirtualFile>,

    /// File path to Uuid mapping (Path, Uuid)
    #[serde(rename = "uuids")]
    virtual_uuids: HashMap<String, String>
}

/// Virtual File
/// Used to map a specific location in the workspace and record its version, description, status etc.
#[derive(Serialize, Deserialize, Encode, Decode, Clone, Debug, PartialEq)]
pub struct VirtualFile {
    /// Path
    #[serde(rename = "path")]
    path: String,

    /// Mapped local file
    #[serde(rename = "real")]
    real: String,

    /// File version
    #[serde(rename = "version")]
    version: u32,

    /// File change history
    #[serde(rename = "changes")]
    change_histories: HashMap<u32, String>,

    /// File version history mapping
    #[serde(rename = "history_real")]
    real_histories: HashMap<u32, String>,

    /// File status
    #[serde(rename = "status")]
    state: VirtualFileState,

    /// Whether the lock is long-term
    #[serde(rename = "long")]
    longer_lock: bool
}

/// Virtual File Status
#[derive(Serialize, Deserialize, Encode, Decode, Clone, Debug, PartialEq)]
pub enum VirtualFileState {
    /// Available (read-write allowed)
    Available,

    /// Being edited (parameter is member Uuid, indicating holder can read-write, others read-only)
    Lock(String)
}

/// Database constructor
impl Default for Database {
    fn default() -> Self {
        Self {
            virtual_files: HashMap::new(),
            virtual_uuids: HashMap::new()
        }
    }
}

/// Loading and updating functionality
impl LocalArchive for Database {
    type DataType = Database;

    fn relative_path() -> String {
        env!("FILE_DATABASE").to_string()
    }
}

impl Database {
    /// Check if path exists (has path mapping)
    pub fn contains_path(&self, path: &str) -> bool {
        self.virtual_uuids.contains_key(&process_path_text(path.to_string()))
    }

    /// Remove a path mapping to make it inaccessible via path
    pub fn remove_file_map(&mut self, path: String) -> Result<String, ()> {
        let path = process_path_text(path);
        // Get Uuid to remove
        let uuid = self.virtual_uuids.get(path.as_str());
        // Path to remove has Uuid mapping
        if let Some(uuid) = uuid {
            // File can be found via Uuid
            let file = self.virtual_files.get_mut(uuid);
            if let Some(file) = file {
                // Force release file lock
                file.throw_locker();

                // Remove file path mapping
                file.path = "".to_string();
                if let Some(uuid) = self.virtual_uuids.remove(path.as_str()) {
                    return Ok(uuid)
                }
            }
        }
        Err(())
    }

    /// Rebuild path mapping for a Uuid
    pub fn rebuild_path_to_uuid(&mut self, uuid: String, path: String) -> Result<(), ()> {
        let path = process_path_text(path);
        // Target path doesn't exist
        if ! self.contains_path(path.as_str()) {
            // Uuid exists
            if let Some(file) = self.virtual_files.get_mut(&uuid) {
                // File has no path binding
                if file.path.is_empty() {
                    // Assign path to file
                    file.path = path.clone();
                    // Create path mapping
                    self.virtual_uuids.insert(uuid, path);
                    return Ok(())
                }
            }
        }
        Err(())
    }

    /// Get references to all files
    pub fn files(&self) -> Vec<&VirtualFile> {
        let mut file_list = Vec::new();
        for (_uuid, file) in self.virtual_files.iter() {
            // Only list files with paths
            if !file.path.trim().is_empty() {
                file_list.push(file);
            }
        }
        file_list
    }

    /// Get mutable references to all files
    pub fn files_mut(&mut self) -> Vec<&mut VirtualFile> {
        let mut file_list = Vec::new();
        for (_uuid, file) in self.virtual_files.iter_mut() {
            file_list.push(file);
        }
        file_list
    }

    /// Insert virtual file
    /// Returns the file back if failed
    pub fn insert_virtual_file(&mut self, file: VirtualFile) -> Result<bool, VirtualFile> {
        // Check if path already exists
        if self.contains_path(file.path.as_str().trim()) {
            // Exists, insertion failed
            return Err(file);
        }

        // Doesn't exist, create Uuid and modify database
        let uuid = Uuid::new_v4();
        self.virtual_files.insert(uuid.to_string(), file.clone());
        self.virtual_uuids.insert(file.path, uuid.to_string());

        Ok(true)
    }

    /// Get virtual file by Uuid
    pub fn file_with_uuid(&self, uuid: String) -> Option<&VirtualFile> {
        self.virtual_files.get(&uuid)
    }

    /// Get virtual file by path
    pub fn file(&self, path: String) -> Option<&VirtualFile> {
        let uuid = self.virtual_uuids.get(path.as_str());
        if let Some(uuid) = uuid {
            self.file_with_uuid(uuid.clone())
        } else {
            None
        }
    }

    /// Search for file
    pub fn search_file(&self, search: String) -> Option<&VirtualFile> {
        if let Some(file) = self.file_with_uuid(search.trim().to_string()) {
            return Some(file);
        } else if let Some(file) = self.file(process_path_text(search)) {
            return Some(file)
        }
        None
    }

    /// Search for file (mutable)
    pub fn search_file_mut(&mut self, search: String) -> Option<&mut VirtualFile> {
        if let Some(_) = self.file_with_uuid(search.trim().to_string()) {
            return self.file_mut_with_uuid(search);
        } else if let Some(_) = self.file(process_path_text(search.clone())) {
            return self.file_mut(search);
        }
        None
    }

    /// Get virtual file by Uuid (mutable)
    pub fn file_mut_with_uuid(&mut self, uuid: String) -> Option<&mut VirtualFile> {
        self.virtual_files.get_mut(&uuid)
    }

    /// Get virtual file by path (mutable)
    pub fn file_mut(&mut self, path: String) -> Option<&mut VirtualFile> {
        let uuid = self.virtual_uuids.get(path.as_str());
        if let Some(uuid) = uuid {
            self.file_mut_with_uuid(uuid.clone())
        } else {
            None
        }
    }

    /// Change file path
    pub fn move_file(&mut self, old_path: String, new_path: String) -> Result<(), ()> {
        let uuid = self.virtual_uuids.get(&process_path_text(old_path));
        if let Some(uuid) = uuid {
            self.move_file_with_uuid(uuid.clone(), new_path)
        } else {
            Err(())
        }
    }

    /// Change path by Uuid
    pub fn move_file_with_uuid(&mut self, uuid: String, new_path: String) -> Result<(), ()> {
        // Process new path string
        let new_path = process_path_text(new_path);

        // Execute if new path doesn't exist
        if ! self.contains_path(new_path.as_str()) {
            // Get file
            let file = self.virtual_files.get_mut(&uuid);
            if let Some(file) = file {
                // Remove path mapping
                self.virtual_uuids.remove(file.path.as_str());

                // Update own path
                file.path = new_path.clone();

                // Rebuild path mapping
                self.virtual_uuids.insert(new_path, uuid);

                // Release lock if not long-term
                if !file.is_longer_lock_unchecked() {
                    file.throw_locker();
                }

                return Ok(())
            }
            Err(())
        } else {
            Err(())
        }
    }

    /// Clean version history
    pub fn clean_histories(&mut self) {
        for (_uuid, file) in self.virtual_files.iter_mut() {
            file.change_histories = HashMap::new();
            file.real_histories = HashMap::new();
        }
    }

    /// Get Uuid of a path
    pub fn uuid_of_path(&self, path: String) -> Option<String> {
        if let Some(uuid) = self.virtual_uuids.get(path.as_str()) {
            return Some(uuid.to_string());
        }
        None
    }
}

impl VirtualFile {
    /// Create new by path
    pub fn new(path: String) -> Self {
        let mut new = Self {
            path: process_path_text(path),
            real: "".to_string(),
            version: 0,
            change_histories: Default::default(),
            real_histories: Default::default(),
            state: Available,
            longer_lock: false
        };

        // Add version 0 data
        new.change_histories.insert(0, "First Version".to_string());
        new.real_histories.insert(0, "".to_string());

        new
    }

    /// Get file name
    pub fn name(&self) -> Option<String> {
        if let Some(name) = self.path.split('/').last() {
            return Some(name.to_string())
        }
        None
    }

    /// Get file path
    pub fn path(&self) -> String {
        self.path.clone()
    }

    /// Get real file path
    pub fn real_path(&self) -> String {
        self.real.clone()
    }

    /// Get file version
    pub fn version(&self) -> u32 {
        self.version
    }

    /// Try to get corresponding client file path
    pub fn client_path(&self) -> Option<PathBuf> {
        match current_dir() {
            Ok(current) => {
                let current = current.join(self.path.as_str());
                Some(current)
            }
            Err(_) => None
        }
    }

    /// Get corresponding server file path
    pub fn server_path(&self) -> Option<PathBuf> {
        match current_dir() {
            Ok(current) => {
                let current = current.join(env!("PATH_DATABASE")).join(self.real.as_str());
                Some(current)
            }
            Err(_) => None
        }
    }

    /// Get corresponding server file path for specific version
    pub fn server_path_version(&self, version: u32) -> Option<PathBuf> {
        match current_dir() {
            Ok(current) => {
                if let Some(real) = self.real_histories.get(&version) {
                    let current = current.join(env!("PATH_DATABASE")).join(real);
                    return Some(current);
                }
                None
            }
            Err(_) => None
        }
    }

    /// Get corresponding server file path for temp file
    pub fn server_path_temp(&self, temp_real: String) -> Option<PathBuf> {
        match current_dir() {
            Ok(current) => {
                let current = current.join(env!("PATH_DATABASE")).join(temp_real);
                Some(current)
            }
            Err(_) => None
        }
    }

    /// Get real path for specific version
    pub fn real_path_version(&self, version: u32) -> Option<PathBuf> {
        if let Some(real) = self.real_histories.get(&version) {
            let current = PathBuf::from(real);
            return Some(current)
        }
        None
    }

    /// Update real path
    pub fn update(&mut self, new_real_path: String, changes_info: String) {
        // Increment version
        self.version += 1;

        // Add current version data
        self.change_histories.insert(self.version, changes_info);
        self.real_histories.insert(self.version, new_real_path.clone());

        // Update current real path
        self.real = new_real_path;
    }

    /// Rollback to specific version
    pub fn rollback_to_version(&mut self, version: u32) -> bool {
        if let Some(old_real) = self.real_histories.get(&version) {
            self.version = version;
            self.real = old_real.clone();

            // Release lock if not long-term
            if !self.is_longer_lock_unchecked() {
                self.throw_locker();
            }

            return true;
        }
        false
    }

    /// Try to acquire lock for member (by Uuid)
    pub async fn give_uuid_locker(&mut self, member_uuid: String, longer: bool) -> bool {
        let workspace = Workspace::read().await;
        if let Some(server) = workspace.server {
            // Self is available
            if self.state == Available {
                // Member exists
                if server.members.contains_key(&member_uuid) {
                    // Lock
                    self.state = Lock(member_uuid);
                    self.longer_lock = longer;
                    return true;
                }
            } else if let Lock(guid) = &self.state {
                // Or member already holds lock
                if guid == &member_uuid.trim() {
                    return true;
                }
            }
        }
        false
    }

    /// Try to acquire lock for member (by Uuid)
    pub async fn give_locker(&mut self, member: &Member, longer: bool) -> bool {
        let workspace = Workspace::read().await;
        if let Some(server) = workspace.server {
            // Self is available
            if self.state == Available {
                // Get Uuid
                let uuid = server.member_uuids.get(&member.member_name);
                if let Some(uuid) = uuid {
                    // Lock
                    self.state = Lock(uuid.clone());
                    self.longer_lock = longer;
                    return true;
                }
            } else if let Lock(locker_owner) = &self.state {
                // Get Uuid
                let uuid = server.member_uuids.get(&member.member_name);
                if let Some(member_uuid) = uuid {
                    // Or member already holds lock
                    if locker_owner == member_uuid {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Release own lock
    pub fn throw_locker(&mut self) {
        self.state = Available;
        self.longer_lock = false;
    }

    /// Get lock owner
    pub async fn get_locker_owner(&self) -> Option<(String, Member)> {
        if let Lock(uuid) = &self.state {
            let workspace = Workspace::read().await;
            if let Some(mut server) = workspace.server {
                if let Some(member) = server.members.remove(uuid.trim()) {
                    return Some((uuid.clone(), member));
                }
            }
        }
        None
    }

    /// Get lock owner (for client environment)
    pub fn get_locker_owner_uuid(&self) -> Option<String> {
        if let Lock(uuid) = &self.state {
            return Some(uuid.clone());
        }
        None
    }

    /// Check if lock is long-term
    pub fn is_longer_lock(&self) -> Option<bool> {
        if let Lock(_) = &self.state {
            return Some(self.longer_lock);
        }
        None
    }

    /// Check if lock is long-term (unchecked)
    pub fn is_longer_lock_unchecked(&self) -> bool {
        self.longer_lock
    }
}