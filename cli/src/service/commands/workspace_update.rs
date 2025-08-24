use crate::data::client_result::ClientResult;
use crate::data::database::Database;
use crate::data::member::Member;
use crate::service::commands::utils_database_sync::{sync_local, sync_local_with_progress, sync_remote_with_progress};
use crate::service::jam_command::Command;
use async_trait::async_trait;
use jam_ready::entry_mutex_async;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

pub struct UpdateCommand;

#[async_trait]
impl Command for UpdateCommand {

    async fn local(&self, stream: &mut TcpStream, _args: Vec<&str>) -> Option<ClientResult> {

        let debug = ClientResult::debug_mode().await;
        let mut command_result = ClientResult::result().await;

        command_result.log("Sync Database.");
        // Sync database
        if debug {
            sync_local(stream).await;
        } else {
            sync_local_with_progress(stream).await;
        }
        command_result.log("Ok");

        Some(command_result)
    }

    async fn remote(
        &self,
        stream: &mut TcpStream, _args: Vec<&str>,
        (_uuid, _member): (String, &Member), database: Arc<Mutex<Database>>) {

        // Sync database
        entry_mutex_async!(database, |guard| {
            sync_remote_with_progress(stream, guard).await;
        });
    }
}

impl UpdateCommand {


}