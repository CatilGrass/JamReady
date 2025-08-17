use crate::cli_commands::client::{exec, print_client_result, CommitArgs};

pub async fn client_commit(args: CommitArgs) {
    if let Some(log) = args.log {
        print_client_result(exec(vec!["commit".to_string(), log]).await);
    } else {
        print_client_result(exec(vec!["commit".to_string()]).await);
    }
}