use crate::cli_commands::client::{exec, NewArgs};
use crate::data::client_result::ClientResult;

pub async fn client_add(args: NewArgs) {
    let mut result = ClientResult::result().await;

    // Add file
    result.combine_unchecked(exec(vec!["file".to_string(), "add".to_string(), args.path.clone()]).await);

    if args.get {
        // Acquire file lock
        result.combine_unchecked(exec(vec!["file".to_string(), "get".to_string(), args.path]).await);
    }

    result.end_print();
}