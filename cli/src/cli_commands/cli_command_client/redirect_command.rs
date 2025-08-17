use jam_ready::utils::address_str_parser::parse_address_v4_str;
use jam_ready::utils::local_archive::LocalArchive;
use crate::cli_commands::client::RedirectArgs;
use crate::data::client_result::ClientResult;
use crate::data::workspace::Workspace;
use crate::service::jam_client::search_workspace_lan;

/// 重定向
pub async fn client_redirect(args: RedirectArgs) {
    let mut result = ClientResult::result().await;
    let mut workspace = Workspace::read().await;

    if let Some(client) = &mut workspace.client {

        // 重定向账户
        if let Some(login_code) = args.login_code {
            client.login_code = login_code;
            result.log(format!("Trying to change login code to {}", client.login_code).as_str());
        }

        // 此处：若同时指定工作区名称和地址，仅更新地址
        if let Some(target_addr) = args.target {

            // 若成功
            if let Ok(addr) = parse_address_v4_str(target_addr).await {
                client.target_addr = addr;

                result.log(format!("Changed target address to {}", &client.target_addr).as_str());

                // 并保存工作区信息
                Workspace::update(&workspace).await;
                return;
            }
            // 失败则继续工作区的查询
        }

        // 若存在工作区名称数据
        if let Some(workspace_name) = args.workspace {

            // 则更新工作区数据
            client.workspace_name = workspace_name;
        }

        // 根据当前工作区刷新地址
        if let Ok(addr) = search_workspace_lan(client.workspace_name.clone()).await {
            client.target_addr = addr;

            result.log(format!("Redirected {} to {}.", client.workspace_name, addr).as_str());

            // 并保存工作区信息
            Workspace::update(&workspace).await;
            return;
        }
    }
    result.err("Redirect failed.");
    result.end_print();
}
