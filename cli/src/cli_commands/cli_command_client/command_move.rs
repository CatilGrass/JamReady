use crate::cli_commands::client::{exec, MoveArgs};
use crate::data::client_result::ClientResult;
use crate::data::database::Database;
use crate::data::local_file_map::LocalFileMap;
use jam_ready::utils::file_operation::move_file;
use jam_ready::utils::local_archive::LocalArchive;
use jam_ready::utils::text_process::process_path_text;
use std::env::current_dir;
use crate::cli_commands::cli_command_client::param_comp::comp::{comp_param_from, comp_param_to};
use crate::cli_commands::cli_command_client::param_comp::data::{CompConfig, CompContext};

pub async fn client_move(args: MoveArgs) -> Option<ClientResult> {

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

    // Compile TO input
    let to = comp_param_to(&config, from.clone().next_with_string(args.to_search.clone()));
    let Ok(to) = to else {
        result.err_and_end(format!("{}", to.err().unwrap()).as_str());
        return None;
    };

    // Acquire file lock if requested
    if args.get {
        result.combine_unchecked(exec(vec!["file".to_string(), "get".to_string(), from.to_string()]).await);
    }

    // Perform move operation
    if args.local {
        // Move local file
        // TODO :: NOTE: Currently doesn't support batch moving as param_comp doesn't support local file parsing
        result.combine_unchecked(client_move_local_file(args).await);
    } else {
        // Move remote file
        result.combine_unchecked(exec(vec!["file".to_string(), "move".to_string(), from.to_string(), to.to_string()]).await);
    }

    // No results
    if result.has_result() {
        Some(result)
    } else {
        result.log("No result");
        Some(result)
    }
}

/// Move local file
async fn client_move_local_file(mv: MoveArgs) -> Option<ClientResult> {
    let mut result = ClientResult::result().await;
    let mut local = LocalFileMap::read().await;
    let database = Database::read().await;

    // Find local file to move
    let Some(local_file_mut) = local.search_to_local_mut(&database, mv.from_search.clone()) else {
        result.err(format!("Fail to move local file: '{}'", &mv.from_search).as_str());
        return Some(result);
    };

    let Ok(current_dir) = current_dir() else { return None; };
    let raw_path = current_dir.join(&local_file_mut.local_path);
    let target_path = current_dir.join(mv.to_search.clone());

    if !target_path.exists() {
        // Update local file path
        let new_path = process_path_text(mv.to_search);
        let old_path = local_file_mut.local_path.clone();
        local_file_mut.local_path = new_path.clone();

        // Remove and rebind UUID
        let Some(uuid) = local.file_uuids.remove(&old_path) else { return None; };
        local.file_uuids.insert(new_path, uuid);

        // Perform actual file move
        if move_file(&raw_path, &target_path).is_ok() {
            // Update configuration after successful move
            LocalFileMap::update(&local).await;
            result.log(format!("Moved local file '{}' to '{}'", raw_path.display(), target_path.display()).as_str());
            return Some(result);
        }
    } else {
        result.err(format!("Cannot move file: file '{}' exists.", target_path.display()).as_str());
        return Some(result);
    }

    None
}