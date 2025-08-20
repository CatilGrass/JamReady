use crate::data::database::Database;
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

// --------------------------------------------------------------------------- //

#[derive(Default, Serialize, Deserialize, Encode, Decode, PartialEq, Debug, Clone)]
pub enum ClientMessage {
    #[default]
    Unknown,

    // ------ Basic Commands ------

    /// Verify identity (login code)
    Verify(String),

    /// Text message
    Text(String),

    /// Indicate operation completed
    Done,

    /// Indicate ready status
    Ready,

    /// Indicate not ready status
    NotReady,

    // ------ Command Operations ------

    /// Send command with arguments
    Command(Vec<String>)
}

#[derive(Default, Serialize, Deserialize, Encode, Decode, PartialEq, Debug, Clone)]
pub enum ServerMessage {
    #[default]
    Unknown,

    // ------ Basic Responses ------

    /// Indicate approval
    Pass,

    /// Indicate rejection (with reason)
    Deny(String),

    /// Indicate operation completed
    Done,

    // ------ Response Data ------

    /// Send database copy
    Sync(Database),

    /// Text message
    Text(String),

    /// UUID response
    Uuid(String)
}