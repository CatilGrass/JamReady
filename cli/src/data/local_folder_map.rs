use crate::data::database::Database;
use bincode::{Decode, Encode};
use jam_ready::utils::local_archive::LocalArchive;
use jam_ready::utils::text_process::split_path_text;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Local folder mapping
#[derive(Default, Serialize, Deserialize, Encode, Decode, Clone, Debug, PartialEq)]
pub struct LocalFolderMap {
    /// Complete directory path to file info mapping
    #[serde(rename = "Mapping")]
    pub folder_files: HashMap<String, Vec<Node>>,

    /// Simplified file search
    #[serde(rename = "Short")]
    pub short_file_map: HashMap<String, String>,
}

/// Node type
#[derive(Default, Serialize, Deserialize, Encode, Decode, Clone, Debug, PartialEq)]
pub enum Node {
    #[default]
    Unknown,

    /// Jump to directory (complete path Folder/)
    Jump(String),

    /// Parent directory navigation
    Parent(String),

    /// File (complete path Folder/file.txt)
    /// Why no Uuid? This data exists only to improve client UI file structure query speed,
    /// the real data is stored in database.yaml
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

        // Process all files and collect all directories
        for file in database.files() {
            let file_path = file.path();
            let (dir_path, _) = split_path_text(&file_path);

            // Add file to corresponding directory
            folder_files.entry(dir_path.clone())
                .or_default()
                .push(Node::File(file_path.clone()));

            // Collect all directory paths
            let mut current_dir = dir_path.clone();
            while !current_dir.is_empty() {
                all_dirs.insert(current_dir.clone());
                current_dir = get_parent_dir(&current_dir).unwrap_or_default();
            }
        }

        // Add root directory
        all_dirs.insert("".to_string());

        // Build directory structure
        for dir_path in &all_dirs {
            // Ensure directory exists in mapping
            let entry = folder_files.entry(dir_path.clone()).or_default();

            // Add Parent node
            if !dir_path.is_empty() {
                if let Some(parent_dir) = get_parent_dir(dir_path) {
                    let parent_node = Node::Parent(parent_dir);
                    if !entry.contains(&parent_node) {
                        entry.push(parent_node);
                    }
                }
            }

            // Get all direct subdirectories
            let sub_dirs: Vec<_> = all_dirs.iter()
                .filter(|d| d.starts_with(dir_path) && *d != dir_path)
                .filter(|d| {
                    let relative = d.trim_start_matches(dir_path);
                    !relative.contains('/') || relative.matches('/').count() == 1
                })
                .cloned()
                .collect();

            // Add Jump nodes to subdirectories
            for sub_dir in sub_dirs {
                if let Some(nodes) = folder_files.get_mut(dir_path) {
                    if !nodes.contains(&Node::Jump(sub_dir.clone())) {
                        nodes.push(Node::Jump(sub_dir));
                    }
                }
            }
        }

        Self {
            folder_files,
            short_file_map: generate_short_mapping(database.files().iter().map(|f| f.path()).collect()),
        }
    }
}

/// Get parent directory path
fn get_parent_dir(dir_path: &str) -> Option<String> {
    if dir_path.is_empty() {
        return None;
    }

    // Remove trailing slash
    let trimmed = dir_path.trim_end_matches('/');

    if let Some(last_idx) = trimmed.rfind('/') {
        // Return parent directory path
        Some(trimmed[..=last_idx].to_string())
    } else {
        // Parent of top-level directory is root
        Some("".to_string())
    }
}

/// Generate short path mappings
fn generate_short_mapping(virtual_file_paths: Vec<String>) -> HashMap<String, String> {
    // Build frequency map for suffixes
    let mut freq_map = HashMap::new();
    for path in &virtual_file_paths {
        let comps: Vec<_> = path.split('/').collect();
        let mut suffix = String::new();

        // Generate all possible suffixes
        for i in (0..comps.len()).rev() {
            suffix = if suffix.is_empty() {
                comps[i].to_string()
            } else {
                format!("{}/{}", comps[i], suffix)
            };
            *freq_map.entry(suffix.clone()).or_insert(0) += 1;
        }
    }

    // Find unique shortest suffix for each path
    let mut result = HashMap::new();
    for path in virtual_file_paths {
        let comps: Vec<_> = path.split('/').collect();
        let mut suffix = String::new();

        // Generate suffix list
        let mut suffixes = Vec::new();
        for i in (0..comps.len()).rev() {
            suffix = if suffix.is_empty() {
                comps[i].to_string()
            } else {
                format!("{}/{}", comps[i], suffix)
            };
            suffixes.push(suffix.clone());
        }

        for candidate in &suffixes {
            if freq_map.get(candidate).copied() == Some(1) {
                // Skip if key equals value
                if candidate != &path {
                    result.insert(format!("{}", candidate.clone()), path.clone());
                }
                break;
            }
        }
    }
    result
}