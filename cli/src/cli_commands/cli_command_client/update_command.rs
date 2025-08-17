use crate::cli_commands::client::{exec, print_client_result};

pub async fn client_update() {
    print_client_result(exec(vec!["update".to_string()]).await);
}