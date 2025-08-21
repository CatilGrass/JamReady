use jam_ready::utils::file_digest::md5_digest;
use jam_ready::utils::local_archive::LocalArchive;
use crate::cli_commands::cli_command_client::param_comp::comp::comp_param_from;
use crate::cli_commands::cli_command_client::param_comp::data::{CompConfig, CompContext};
use crate::cli_commands::client::CompleteArgs;
use crate::data::client_result::ClientResult;
use crate::data::database::Database;
use crate::data::local_file_map::LocalFileMap;

pub async fn client_complete(args: CompleteArgs) {
    let mut result = ClientResult::result().await;
    let config = CompConfig::read().await;

    let from = comp_param_from(&config, CompContext::input(&args.from_search));
    let Ok(from) = from else {
        result.err_and_end(format!("{}", from.err().unwrap()).as_str());
        return;
    };

    let mut local = LocalFileMap::read().await;
    let database = Database::read().await;
    let commit = args.info.unwrap_or("Update".to_string());

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

    let _ = LocalFileMap::update(&local).await;

    // No results
    if result.has_result() {
        result.end_print();
    } else {
        result.err_and_end("No result");
    }
}