use crate::cli_commands::client::exec;
use crate::data::client_result::ClientResult;

pub async fn client_commit() -> Option<ClientResult> {
    exec(vec!["commit".to_string()]).await
}