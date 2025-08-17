use crate::cli_commands::client::{exec, print_client_result};

pub async fn client_archive() {
    print_client_result(exec(vec!["archive".to_string()]).await);
}