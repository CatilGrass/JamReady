use crate::data::database::Database;
use crate::data::member::Member;
use crate::data::member::MemberDuty::Leader;
use crate::service::commands::duty_verifier::{verify, verify_duty};
use crate::service::jam_command::Command;
use async_trait::async_trait;
use jam_ready::utils::local_archive::LocalArchive;
use std::path::PathBuf;
use tokio::net::TcpStream;

pub struct CleanCommand;

#[async_trait]
impl Command for CleanCommand {

    async fn local(&self, stream: &mut TcpStream, _args: Vec<&str>) {

        // 验证 Leader 身份通过
        if ! verify(stream).await { return; }
    }

    async fn remote(
        &self, stream: &mut TcpStream, _args: Vec<&str>,
        (_uuid, member): (String, &Member), database: &mut Database)
        -> bool {

        // 验证对方是否为 Leader
        if ! verify_duty(stream, member, Leader).await { return false; }

        // 服务端直接执行清理
        let mut i = 0;
        loop {
            let path = PathBuf::from(env!("PATH_DATABASE_CONFIG_ARCHIVE")).join(format!("history_{}.yaml", i));
            if path.exists() {
                i += 1;
                continue
            }

            // 备份当前版本
            if let Some(path) = path.to_str() {
                Database::update_to(database, path.to_string());
                database.clean_histories();
                break;
            }
        }

        true
    }
}