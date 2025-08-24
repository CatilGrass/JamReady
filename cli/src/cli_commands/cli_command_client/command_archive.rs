use crate::cli_commands::client::exec;
use crate::data::client_result::ClientResult;

pub async fn client_archive() -> Option<ClientResult> {
    exec(vec!["archive".to_string()]).await
}