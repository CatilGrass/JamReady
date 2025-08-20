use crate::cli_commands::cli_command_client::param_comp::comp::comp_param_from;
use crate::cli_commands::cli_command_client::param_comp::data::{CompConfig, CompContext};
use crate::cli_commands::client::{exec, RollbackArgs};
use crate::data::client_result::ClientResult;

pub async fn client_rollback (args: RollbackArgs) {

    let mut result = ClientResult::result().await;

    let config = CompConfig::read().await;
    let from = comp_param_from(&config, CompContext::input(&args.from_search));
    let Ok(from) = from else {
        result.err_and_end(format!("{}", from.err().unwrap()).as_str());
        return;
    };

    // 理论上 Rollback 不应该支持多目录操作的，但是我还是想加

    if args.get {
        // 获得文件的锁
        result.combine_unchecked(exec(vec!["file".to_string(), "get".to_string(), from.to_string()]).await);
    }
    // 回滚版本
    result.combine_unchecked(exec(vec!["file".to_string(), "rollback".to_string(), from.to_string(), (&args.to_version).to_string()]).await);

    // 直接重新下载文件
    if args.back {
        result.combine_unchecked(exec(vec!["view".to_string(), from.to_string(), args.to_version.to_string()]).await);
    }

    // 无结果时
    if result.has_result() {
        result.end_print();
    } else {
        result.err_and_end("No result");
    }
}