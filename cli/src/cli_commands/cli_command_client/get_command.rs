use crate::cli_commands::cli_command_client::param_comp::comp::comp_param_from;
use crate::cli_commands::cli_command_client::param_comp::data::{CompConfig, CompContext};
use crate::cli_commands::client::{exec, GetArgs};
use crate::data::client_result::ClientResult;

pub async fn client_get (args: GetArgs) {

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

    // Combine result returned
    result.combine_unchecked(
        // Exec get command and ↑
        exec(
            vec![
                "file".to_string(),
                 if args.longer {
                     "get_longer".to_string()
                 } else {
                     "get".to_string()
                 },
                from.to_string()
            ]
        ).await
    );

    // No results
    result.end_print();
}