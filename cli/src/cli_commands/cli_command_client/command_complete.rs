use jam_ready::utils::file_digest::md5_digest;
use jam_ready::utils::local_archive::LocalArchive;
use crate::cli_commands::cli_command_client::param_comp::comp::comp_param_from;
use crate::cli_commands::cli_command_client::param_comp::data::{CompConfig, CompContext};
use crate::cli_commands::client::CompleteArgs;
use crate::data::client_result::ClientResult;
use crate::data::database::Database;
use crate::data::local_file_map::LocalFileMap;

pub async fn client_complete(args: CompleteArgs) -> Option<ClientResult> {

    // Create result struct
    let mut result = ClientResult::result().await;

    // Create compile config
    let config = CompConfig::read().await;

    // Compile FROM input
    let from = comp_param_from(&config, CompContext::input(&args.from_search));
    let Ok(from) = from else {
        result.err_and_end(format!("{}", from.err().unwrap()).as_str());
        return None;
    };

    // Read local, remote database
    let mut local = LocalFileMap::read().await;
    let database = Database::read().await;

    // Read commit info
    let commit = args.info.unwrap_or("Update".to_string());

    // Mark file completed (If it's modified.)
    for final_path in from.final_paths {
        let Some(path) = local.search_to_path(&database, final_path.clone()) else { continue; };
        let Some(local_file) = local.search_to_local_mut(&database, final_path.clone()) else { continue; };
        let Ok(digest) = md5_digest(path) else { continue; };
        if digest == local_file.local_digest { continue; };
        local_file.completed = true;
        local_file.completed_commit = commit.clone();
        local_file.completed_digest = digest;
        result.log(format!("Completed {}", local_file.local_path).as_str());
    }

    // Update local database
    let _ = LocalFileMap::update(&local).await;

    // No results
    if result.has_result() {
        Some(result)
    } else {
        result.log("No result");
        Some(result)
    }
}