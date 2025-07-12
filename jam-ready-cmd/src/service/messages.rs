use bincode::{config, Decode, Encode};
use crate::data::database::Database;
use bincode::config::Configuration;
use std::fmt::Debug;
use bincode::error::EncodeError;
use serde::{Deserialize, Serialize};

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

// --------------------------------------------------------------------------- //

pub const BINCODE_CONFIG : Configuration = config::standard();

pub trait MessageEncoder<M: Encode + Decode<()> + Default + Debug> {

    fn en(&self) -> Result<Vec<u8>, EncodeError> where Self : Encode {
         bincode::encode_to_vec(self, BINCODE_CONFIG)
    }

    fn de(encoded : Vec<u8>) -> Result<M, bincode::error::DecodeError> {
        match bincode::decode_from_slice(&encoded[..], BINCODE_CONFIG) {
            Ok((decoded, _)) => Ok(decoded),
            Err(err) => Err(err)
        }
    }
}

#[macro_export]
macro_rules! encoder {
    ($($msg:ident),+) => {
        $(
            impl MessageEncoder<$msg> for $msg {}
        )+
    };
}

encoder!(
    ServerMessage, ClientMessage, Database
);