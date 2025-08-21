use crate::cli_commands::client::{exec, print_client_result};

pub async fn client_commit() {
    print_client_result(exec(vec!["commit".to_string()]).await);
}