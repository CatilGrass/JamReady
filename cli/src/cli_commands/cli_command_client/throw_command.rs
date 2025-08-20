use crate::cli_commands::cli_command_client::param_comp::comp::comp_param_from;
use crate::cli_commands::cli_command_client::param_comp::data::{CompConfig, CompContext};
use crate::cli_commands::client::{exec, SearchArgs};
use crate::data::client_result::ClientResult;

pub async fn client_throw(args: SearchArgs) {

    let mut result = ClientResult::result().await;

    let config = CompConfig::read().await;
    let from = comp_param_from(&config, CompContext::input(&args.search));
    let Ok(from) = from else {
        result.err_and_end(format!("{}", from.err().unwrap()).as_str());
        return;
    };

    result.combine_unchecked(exec(vec!["file".to_string(), "throw".to_string(), from.to_string()]).await);

    // No results
    if result.has_result() {
        result.end_print();
    } else {
        result.err_and_end("No result");
    }
}