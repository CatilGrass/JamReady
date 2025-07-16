use crate::data::database::Database;
use crate::data::member::Member;
use crate::service::commands::database_sync::{sync_local_with_progress, sync_remote_with_progress};
use crate::service::jam_command::Command;
use async_trait::async_trait;
use tokio::net::TcpStream;

pub struct SyncCommand;

#[async_trait]
impl Command for SyncCommand {

    async fn local(&self, stream: &mut TcpStream, _args: Vec<&str>) {

        // 同步数据库
        sync_local_with_progress(stream).await;

        // TODO :: 同步关联、关注 （待制作）
        println!("Ok");
    }

    async fn remote(
        &self,
        stream: &mut TcpStream, _args: Vec<&str>,
        (_uuid, _member): (String, &Member), database: &mut Database) -> bool {

        // 同步数据库
        sync_remote_with_progress(stream, database).await;
        false
    }
}