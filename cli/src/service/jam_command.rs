use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use colored::{ColoredString, Colorize};
use log::info;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use crate::data::client_result::ClientResult;
use crate::data::database::Database;
use crate::data::member::Member;

pub type CommandRegistry = HashMap<&'static str, Arc<dyn Command + Send + Sync>>;

#[async_trait]
pub trait Command {
    /// Client-side operation
    async fn local(&self, stream: &mut TcpStream, args: Vec<&str>) -> Option<ClientResult>;

    /// Server-side operation (returns whether database was modified)
    async fn remote(&self, stream: &mut TcpStream, args: Vec<&str>, member: (String, &Member), database: Arc<Mutex<Database>>);
}

/// Execute local command
pub async fn execute_local_command(
    registry: &CommandRegistry,
    stream: &mut TcpStream,
    args: Vec<&str>,
) -> Option<ClientResult> {
    if let Some(command_name) = args.get(0) {
        let command_name = command_name.trim().to_lowercase();
        if let Some(command) = registry.get(command_name.as_str()) {
            // Execute command
            return command.local(stream, args).await;
        } else {
            eprintln!("Unknown command: {}", command_name);
        }
    }
    None
}

/// Execute remote command
pub async fn execute_remote_command(
    registry: &CommandRegistry,
    stream: &mut TcpStream,
    args: Vec<&str>,
    (uuid, member): (String, &Member),
    database: Arc<Mutex<Database>>
) {
    info!("{}: {}", &member.member_name.yellow(), display_args(&args));

    if let Some(command_name) = args.get(0) {
        let command_name = command_name.trim().to_lowercase();
        if let Some(command) = registry.get(command_name.as_str()) {
            // Execute command
            command.remote(stream, args, (uuid, member), database.clone()).await;
        } else {
            eprintln!("Unknown command: {}", command_name);
        }
    }
}

/// Format command arguments for display
fn display_args(args: &Vec<&str>) -> ColoredString {
    let mut result = String::default();

    for arg in args {
        if arg.contains(" ") {
            result = format!("{} \"{}\"", result, arg);
        } else {
            result = format!("{} {}", result, arg);
        }
    }

    format!("{}", result.trim()).cyan()
}