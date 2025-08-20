use jam_ready::utils::address_str_parser::parse_address_v4_str;
use jam_ready::utils::local_archive::LocalArchive;
use crate::cli_commands::client::RedirectArgs;
use crate::data::client_result::ClientResult;
use crate::data::workspace::Workspace;
use crate::service::jam_client::search_workspace_lan;

/// Redirect
pub async fn client_redirect(args: RedirectArgs) {
    let mut result = ClientResult::result().await;
    let mut workspace = Workspace::read().await;

    if let Some(client) = &mut workspace.client {

        // Redirect account
        if let Some(login_code) = args.login_code {
            client.login_code = login_code;
            result.log(format!("Trying to change login code to {}", client.login_code).as_str());
        }

        // Note: If both workspace name and address are specified, only update the address
        if let Some(target_addr) = args.target {

            // If successful
            if let Ok(addr) = parse_address_v4_str(target_addr).await {
                client.target_addr = addr;

                result.log(format!("Changed target address to {}", &client.target_addr).as_str());

                // And save workspace info
                Workspace::update(&workspace).await;
                return;
            }
            // If failed, continue with workspace search
        }

        // If workspace name data exists
        if let Some(workspace_name) = args.workspace {

            // Then update workspace data
            client.workspace_name = workspace_name;
        }

        // Refresh address based on current workspace
        if let Ok(addr) = search_workspace_lan(client.workspace_name.clone()).await {
            client.target_addr = addr;

            result.log(format!("Redirected {} to {}.", client.workspace_name, addr).as_str());

            // And save workspace info
            Workspace::update(&workspace).await;
            return;
        }
    }
    result.err("Redirect failed.");
    result.end_print();
}