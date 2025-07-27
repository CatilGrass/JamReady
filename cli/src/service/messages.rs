use crate::data::database::Database;
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

// --------------------------------------------------------------------------- //

#[derive(Default, Serialize, Deserialize, Encode, Decode, PartialEq, Debug, Clone)]
pub enum ClientMessage {

    #[default]
    Unknown,

    // ------ 基础部分 ------

    /// 验证自己的身份 (登录代码)
    Verify(String),

    /// 文本
    Text(String),

    /// 表示自己操作完成
    Done,

    /// 表示自己准备就绪
    Ready,

    /// 表示自己结束操作
    NotReady,

    // ------ 命令部分 ------

    /// 发送字符串命令
    Command(Vec<String>)
}

#[derive(Default, Serialize, Deserialize, Encode, Decode, PartialEq, Debug, Clone)]
pub enum ServerMessage {

    #[default]
    Unknown,

    // ------ 基础部分 ------

    /// 表示同意
    Pass,

    /// 表示拒绝 (原因)
    Deny(String),

    /// 表示自己操作完成
    Done,

    // ---- 返回内容部分 ----

    /// 发送数据库的拷贝
    Sync(Database),

    /// 文本
    Text(String),

    /// Uuid
    Uuid(String)
}