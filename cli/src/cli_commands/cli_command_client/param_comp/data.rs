use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use jam_ready::utils::local_archive::LocalArchive;
use crate::cli_commands::cli_command_client::param_comp::SPLIT_CHAR;
use crate::data::database::Database;
use crate::data::local_folder_map::LocalFolderMap;

pub struct CompConfig {
    /// Whether the expression allows multiple paths
    pub allow_multi_path: bool,

    /// Context environment used by the expression
    pub local_folder_map: LocalFolderMap,

    /// Database used by the expression
    pub database: Database
}

impl CompConfig {
    pub async fn read() -> CompConfig {
        Self {
            allow_multi_path: true,
            local_folder_map: LocalFolderMap::read().await,
            database: Database::read().await,
        }
    }
}

#[derive(Default, Clone)]
pub struct CompContext {
    /// Input
    pub input: String,

    /// Context directory
    pub ctx: String,

    /// Output paths
    pub final_paths: Vec<String>,
}

impl Display for CompContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut result = String::new();
        for context_path in &self.final_paths {
            result.push_str(format!("{}{}", context_path.trim(), SPLIT_CHAR).as_str());
        }
        write!(f, "{}", result.trim_end_matches(SPLIT_CHAR).trim())
    }
}

#[allow(dead_code)]
impl CompContext {
    pub fn input(str: &str) -> Self {
        Self::input_string(str.to_string())
    }

    pub fn input_string(string: String) -> Self {
        Self {
            input: string,
            ..CompContext::default()
        }
    }

    pub fn next(self, input: &str) -> Self {
        self.next_with_string(input.to_string())
    }

    pub fn next_with_string(self, input: String) -> Self {
        Self {
            input,
            ..self
        }
    }
}

#[derive(Debug)]
pub struct CompError {
    pub err: String
}

impl Display for CompError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.err)
    }
}

impl Error for CompError {}

impl Default for CompError {
    fn default() -> Self {
        Self {
            err: String::new(),
        }
    }
}

impl CompError {
    pub fn err(err: &str) -> Self {
        Self { err: err.to_string() }
    }
}