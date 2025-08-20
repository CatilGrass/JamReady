use crate::data::database::Database;
use crate::data::member::Member;
use crate::data::member::MemberDuty::Leader;
use crate::service::commands::duty_verifier::{verify, verify_duty};
use crate::service::jam_command::Command;
use async_trait::async_trait;
use jam_ready::utils::local_archive::LocalArchive;
use jam_ready::entry_mutex_async;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use crate::data::client_result::ClientResult;

pub struct ArchiveCommand;

#[async_trait]
impl Command for ArchiveCommand {
    async fn local(&self, stream: &mut TcpStream, _args: Vec<&str>) -> Option<ClientResult> {
        let mut command_result = ClientResult::result().await;

        // Verify leader identity
        if !verify(stream).await {
            command_result.err("You are not the leader and cannot execute this command.");
            return Some(command_result);
        }

        command_result.log("Archive Success.");
        Some(command_result)
    }

    async fn remote(
        &self,
        stream: &mut TcpStream,
        _args: Vec<&str>,
        (_uuid, member): (String, &Member),
        database: Arc<Mutex<Database>>
    ) {
        // Verify leader duty
        if !verify_duty(stream, member, Leader).await {
            return;
        }

        // Find available archive filename
        let mut i = 0;
        let archive_path = loop {
            let path = PathBuf::from(env!("PATH_DATABASE_CONFIG_ARCHIVE"))
                .join(format!("history_{}.yaml", i));

            if !path.exists() {
                break path;
            }
            i += 1;
        };

        // Create archive backup
        if let Some(path_str) = archive_path.to_str() {
            entry_mutex_async!(database, |guard| {
                Database::update_to(guard, path_str.to_string()).await;
                guard.clean_histories();
            });
        }

        // Update main database
        entry_mutex_async!(database, |guard| {
            Database::update(guard).await;
        });
    }
}