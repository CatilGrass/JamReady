use crate::cli_commands::cli_command_client::param_comp::comp::comp_param_from;
use crate::cli_commands::cli_command_client::param_comp::data::{CompConfig, CompContext};
use crate::cli_commands::client::{exec, ViewArgs};
use crate::data::client_result::ClientResult;

pub async fn client_view(args: ViewArgs) -> Option<ClientResult> {

    let mut result = ClientResult::result().await;

    let config = CompConfig::read().await;
    let from = comp_param_from(&config, CompContext::input(&args.from_search));
    let Ok(from) = from else {
        result.err_and_end(format!("{}", from.err().unwrap()).as_str());
        return None;
    };

    if let Some(version) = args.version {
        result.combine_unchecked(exec(vec!["view".to_string(), from.to_string(), version.to_string()]).await);
    } else {
        result.combine_unchecked(exec(vec!["view".to_string(), from.to_string()]).await);
    }

    if args.get {
        // Acquire file lock
        result.combine_unchecked(exec(vec!["file".to_string(), "get".to_string(), from.to_string()]).await);
    }

    // No results
    if result.has_result() {
        Some(result)
    } else {
        result.log("No result");
        Some(result)
    }
}