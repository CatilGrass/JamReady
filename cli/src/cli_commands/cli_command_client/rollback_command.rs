use crate::cli_commands::cli_command_client::param_comp::comp::comp_param_from;
use crate::cli_commands::cli_command_client::param_comp::data::{CompConfig, CompContext};
use crate::cli_commands::client::{exec, RollbackArgs};
use crate::data::client_result::ClientResult;

pub async fn client_rollback(args: RollbackArgs) {

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

    // Theoretically Rollback shouldn't support multi-directory operations, but I still want to add it :)))))

    if args.get {
        // Acquire file lock
        result.combine_unchecked(exec(vec!["file".to_string(), "get".to_string(), from.to_string()]).await);
    }
    // Rollback version
    result.combine_unchecked(exec(vec!["file".to_string(), "rollback".to_string(), from.to_string(), (&args.to_version).to_string()]).await);

    // Directly re-download the file
    if args.back {
        result.combine_unchecked(exec(vec!["view".to_string(), from.to_string(), args.to_version.to_string()]).await);
    }

    // When no results
    if result.has_result() {
        result.end_print();
    } else {
        result.err_and_end("No result");
    }
}