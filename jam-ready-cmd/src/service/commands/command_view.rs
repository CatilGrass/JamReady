use crate::data::database::Database;
use crate::data::local_file_map::LocalFileMap;
use crate::data::member::Member;
use crate::service::commands::database_sync::{sync_local, sync_remote};
use crate::service::commands::file_transmitter::{read_file, send_file};
use crate::service::jam_command::Command;
use async_trait::async_trait;
use jam_ready::utils::local_archive::LocalArchive;
use tokio::net::TcpStream;

pub struct ViewCommand;

#[async_trait]
impl Command for ViewCommand {

    async fn local(&self, stream: &mut TcpStream, args: Vec<&str>) {

        // 同步数据库
        sync_local(stream).await;
        let database = Database::read();

        // 加载本地映射表
        let mut local = LocalFileMap::read();

        // 检查参数数量
        if args.len() < 2 { return; } // <搜索>

        // 文件目录
        let file_path_str = args[1].to_string();
        let file = database.search_file(file_path_str.clone());
        if let Some(file) = file {
            if let Some(client_path) = file.client_path() {
                match read_file(stream, client_path).await {
                    Ok(_) => {
                        if let Some(uuid) = database.uuid_of_path(file.path()) {
                            local.file_uuids.insert(file.path(), uuid.clone());
                            local.file_paths.insert(uuid.clone(), file.path());
                        }
                    }
                    Err(_) => { }
                }
            }
        }

        LocalFileMap::update(&local);
    }

    async fn remote(&self, stream: &mut TcpStream, args: Vec<&str>, (_uuid, _member): (String, &Member), database: &mut Database) -> bool {

        // 首先同步数据库
        sync_remote(stream, database).await;

        // 检查参数数量
        if args.len() < 2 { return false; } // <搜索>

        // 文件路径
        let file_path_str = args[1].to_string();
        let file = database.search_file(file_path_str.clone());
        if let Some(file) = file {
            if let Some(server_path) = file.server_path() {
                match send_file(stream,server_path).await {
                    Ok(_) => { }
                    Err(_) => { }
                }
            }
        }

        false
    }
}