use crate::cli_commands::cli_command_client::param_comp::comp::comp_param_from;
use crate::cli_commands::cli_command_client::param_comp::data::{CompConfig, CompContext};
use crate::cli_commands::client::{exec, RemoveArgs};
use crate::data::client_result::ClientResult;

pub async fn client_remove(args: RemoveArgs) {

    // Create result struct
    let mut result = ClientResult::result().await;

    // Create compile config
    let config = CompConfig::read().await;

    // Compile FROM input
    let from = comp_param_from(&config, CompContext::input(&args.from_search));
    let Ok(from) = from else {
        result.err_and_end(format!("{}", from.err().unwrap()).as_str());
        return;
    };

    if args.get {
        // Acquire file lock
        result.combine_unchecked(exec(vec!["file".to_string(), "get".to_string(), from.to_string()]).await);
    }
    // Remove file
    result.combine_unchecked(exec(vec!["file".to_string(), "remove".to_string(), from.to_string()]).await);

    // When no results
    if result.has_result() {
        result.end_print();
    } else {
        result.err_and_end("No result");
    }
}