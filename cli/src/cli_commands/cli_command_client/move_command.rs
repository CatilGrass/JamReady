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

pub async fn client_move(args: MoveArgs) {

    let mut result = ClientResult::result().await;
    let config = CompConfig::read().await;
    let from = comp_param_from(&config, CompContext::input(&args.from_search));
    let Ok(from) = from else {
        result.err_and_end(format!("{}", from.err().unwrap()).as_str());
        return;
    };

    let to = comp_param_to(&config, from.clone().next_with_string(args.to_search.clone()));
    let Ok(to) = to else {
        result.err_and_end(format!("{}", to.err().unwrap()).as_str());
        return;
    };

    if args.get {
        // 获得文件的锁
        result.combine_unchecked(exec(vec!["file".to_string(), "get".to_string(), from.to_string()]).await);
    }

    if args.local {
        // 移动本地文件
        // TODO :: 移动本地文件并不支持批量移动
        result.combine_unchecked(client_move_local_file(args).await);

    } else {

        // 移动远程文件
        result.combine_unchecked(exec(vec!["file".to_string(), "move".to_string(), from.to_string(), to.to_string()]).await);
    }

    // 无结果时
    if result.has_result() {
        result.end_print();
    } else {
        result.err_and_end("No result");
    }
}

/// 移动本地文件
async fn client_move_local_file(mv: MoveArgs) -> Option<ClientResult> {
    let mut result = ClientResult::result().await;

    let mut local = LocalFileMap::read().await;
    let database = Database::read().await;

    let Some(local_file_mut) = local.search_to_local_mut(&database, mv.from_search.clone()) else {
        result.err(format!("Fail to move local file: '{}'", &mv.from_search).as_str());
        return Some(result);
    };
    let Ok(current_dir) = current_dir() else { return None; };

    let raw_path = current_dir.join(&local_file_mut.local_path);
    let target_path = current_dir.join(mv.to_search.clone());

    if ! target_path.exists() {

        // 修改本地文件地址
        let new_path = process_path_text(mv.to_search);
        let old_path = local_file_mut.local_path.clone();
        local_file_mut.local_path = new_path.clone();

        // 移除 Uuid
        let Some(uuid) = local.file_uuids.remove(&old_path) else { return None; };

        // 重新绑定 Uuid 到新地址
        local.file_uuids.insert(new_path, uuid);

        // 全部成功后，移动文件
        if let Ok(_) = move_file(&raw_path, &target_path) {

            // 移动文件成功后，更新配置
            LocalFileMap::update(&local).await;
            result.log(format!("Moved local file '{}' to '{}'", raw_path.display(), target_path.display()).as_str());
            return Some(result)
        }

    } else {
        result.err(format!("Cannot move file: file '{}' exists.", target_path.display()).as_str());
        return Some(result)
    }
    None
}