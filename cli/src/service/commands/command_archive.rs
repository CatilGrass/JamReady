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

pub struct ArchiveCommand;

#[async_trait]
impl Command for ArchiveCommand {

    async fn local(&self, stream: &mut TcpStream, _args: Vec<&str>) {

        // 验证 Leader 身份通过
        if ! verify(stream).await { return; }
        println!("Ok: Archive successfully.");
    }

    async fn remote(
        &self, stream: &mut TcpStream, _args: Vec<&str>,
        (_uuid, member): (String, &Member), database: Arc<Mutex<Database>>) {

        // 验证对方是否为 Leader
        if ! verify_duty(stream, member, Leader).await { return; }

        // 服务端直接执行归档
        let mut i = 0;
        loop {
            let path = PathBuf::from(env!("PATH_DATABASE_CONFIG_ARCHIVE")).join(format!("history_{}.yaml", i));
            if path.exists() {
                i += 1;
                continue
            }

            // 备份当前版本
            if let Some(path) = path.to_str() {
                entry_mutex_async!(database, |guard| {
                    Database::update_to(guard, path.to_string()).await;
                    guard.clean_histories();
                });

                break;
            }
        }

        entry_mutex_async!(database, |guard| {
            Database::update(guard).await;
        });
    }
}