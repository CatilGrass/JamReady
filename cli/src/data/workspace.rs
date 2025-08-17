use crate::data::member::Member;
use crate::data::workspace::WorkspaceType::Unknown;
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use jam_ready::utils::local_archive::LocalArchive;

/// Workspace - 工作区
/// 工作区是 服务端、客户端 都会存在的目录
/// 它是用来确认当前目录的信息的
#[derive(Serialize, Deserialize, Encode, Decode, Clone, Debug, PartialEq)]
pub struct Workspace {

    /// 工作区类型
    #[serde(rename = "Type")]
    pub workspace_type: WorkspaceType,

    /// 成员工作区
    #[serde(rename = "Client")]
    pub client: Option<ClientWorkspace>,

    /// 服务器工作区
    #[serde(rename = "Server")]
    pub server: Option<ServerWorkspace>,
}

/// 工作区类型
#[derive(Serialize, Deserialize, Encode, Decode, Clone, Debug, PartialEq)]
pub enum WorkspaceType {

    /// 未知
    Unknown,

    /// 服务器
    Server,

    /// 成员
    Client
}

#[derive(Serialize, Deserialize, Encode, Decode, Clone, Debug, PartialEq)]
pub struct ClientWorkspace {

    /// 连接到的工作区名称 (无法连接会尝试网络发现)
    #[serde(rename = "Workspace")]
    pub workspace_name: String,

    /// 成员连接到的地址
    #[serde(rename = "Address")]
    pub target_addr: SocketAddr,

    /// 登录口令
    #[serde(rename = "LoginCode")]
    pub login_code: String,

    /// 成员自身的 Uuid
    #[serde(rename = "SelfUuid")]
    pub uuid: String,

    /// 调试输出
    #[serde(rename = "Debug")]
    pub debug: bool,
}

#[derive(Serialize, Deserialize, Encode, Decode, Clone, Debug, PartialEq)]
pub struct ServerWorkspace {

    /// 工作区名称 (用于网络发现)
    #[serde(rename = "Name")]
    pub workspace_name: String,

    /// 成员表
    #[serde(rename = "Members")]
    pub members: HashMap<String, Member>,

    /// 成员 ID 和 UUID 映射
    #[serde(rename = "MemUuids")]
    pub member_uuids: HashMap<String, String>,

    /// 登录代码映射
    #[serde(rename = "LoginCodes")]
    pub login_code_map: HashMap<String, String>,

    /// 是否启用 Debug 级别 Logger
    #[serde(rename = "Debug")]
    pub enable_debug_logger: bool,
}

impl Default for Workspace {

    /// 初始化工作区
    fn default() -> Self {
        Self {
            workspace_type: Unknown,
            client: None,
            server: None,
        }
    }
}

/// 加载和更新功能
impl LocalArchive for Workspace {
    type DataType = Workspace;

    fn relative_path() -> String {
        env!("FILE_WORKSPACE_SERVER_DATA").to_string()
    }
}