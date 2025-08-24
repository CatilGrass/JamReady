use crate::data::database::Database;
use crate::data::local_folder_map::LocalFolderMap;
use crate::service::messages::ServerMessage::Sync;
use crate::service::service_utils::{read_large_msg, send_large_msg};
use indicatif::ProgressBar;
use jam_ready::utils::local_archive::LocalArchive;
use tokio::net::TcpStream;

pub async fn sync_local(stream: &mut TcpStream) {
    let progress_bar = None;
    let database_sync = &read_large_msg(stream, progress_bar).await;
    if let Ok(database) = database_sync {
        Database::update(database).await;
        LocalFolderMap::update(&database.into()).await;
    }
}

pub async fn sync_remote(stream: &mut TcpStream, database: &Database) {
    let database = Sync(database.clone());
    let progress_bar = None;
    let _ = send_large_msg(stream, &database, progress_bar).await;
}


pub async fn sync_local_with_progress(stream: &mut TcpStream) {
    let progress_bar = Some(ProgressBar::new(0));
    let database_sync = &read_large_msg(stream, progress_bar).await;
    if let Ok(database) = database_sync {
        Database::update(database).await;
        LocalFolderMap::update(&database.into()).await;
    }
}

pub async fn sync_remote_with_progress(stream: &mut TcpStream, database: &Database) {
    let database = Sync(database.clone());
    let progress_bar = Some(ProgressBar::new(0));
    let _ = send_large_msg(stream, &database, progress_bar).await;
}