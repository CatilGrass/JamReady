use crate::data::member::Member;
use crate::data::workspace::WorkspaceType::Unknown;
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use jam_ready::utils::local_archive::LocalArchive;

/// Workspace - Working Environment
/// The workspace exists for both server and client instances
/// It contains information about the current working directory
#[derive(Serialize, Deserialize, Encode, Decode, Clone, Debug, PartialEq)]
pub struct Workspace {
    /// Workspace type
    #[serde(rename = "type")]
    pub workspace_type: WorkspaceType,

    /// Client workspace configuration
    #[serde(rename = "client")]
    pub client: Option<ClientWorkspace>,

    /// Server workspace configuration
    #[serde(rename = "server")]
    pub server: Option<ServerWorkspace>,
}

/// Workspace type classification
#[derive(Serialize, Deserialize, Encode, Decode, Clone, Debug, PartialEq)]
pub enum WorkspaceType {
    /// Unknown workspace type
    Unknown,

    /// Server workspace
    Server,

    /// Client workspace
    Client,
}

/// Client workspace configuration
#[derive(Serialize, Deserialize, Encode, Decode, Clone, Debug, PartialEq)]
pub struct ClientWorkspace {
    /// Target workspace name (will attempt network discovery if connection fails)
    #[serde(rename = "workspace")]
    pub workspace_name: String,

    /// Server address to connect to
    #[serde(rename = "addr")]
    pub target_addr: SocketAddr,

    /// Authentication token
    #[serde(rename = "login_code")]
    pub login_code: String,

    /// Client's unique identifier
    #[serde(rename = "member_uuid")]
    pub uuid: String,

    /// Debug output flag
    #[serde(rename = "debug")]
    pub debug: bool,
}

/// Server workspace configuration
#[derive(Serialize, Deserialize, Encode, Decode, Clone, Debug, PartialEq)]
pub struct ServerWorkspace {
    /// Workspace name (used for network discovery)
    #[serde(rename = "workspace")]
    pub workspace_name: String,

    /// Member registry
    #[serde(rename = "members")]
    pub members: HashMap<String, Member>,

    /// Mapping between member IDs and UUIDs
    #[serde(rename = "uuids")]
    pub member_uuids: HashMap<String, String>,

    /// Authentication token mapping
    #[serde(rename = "login_code")]
    pub login_code_map: HashMap<String, String>,

    /// Debug logging flag
    #[serde(rename = "debug")]
    pub enable_debug_logger: bool,
}

impl Default for Workspace {
    /// Initialize a new workspace
    fn default() -> Self {
        Self {
            workspace_type: Unknown,
            client: None,
            server: None,
        }
    }
}

/// Loading and updating functionality
impl LocalArchive for Workspace {
    type DataType = Workspace;

    fn relative_path() -> String {
        env!("FILE_WORKSPACE_SERVER_DATA").to_string()
    }
}

pub async fn debug_mode(debug: bool) {
    let mut workspace = Workspace::read().await;
    let Some(mut client) = workspace.client.clone() else {
        return;
    };
    client.debug = debug;
    workspace.client = Some(client);
    Workspace::update(&workspace).await;
}