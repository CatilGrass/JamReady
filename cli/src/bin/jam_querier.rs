use jam_ready::utils::local_archive::LocalArchive;
use jam_ready_cli::data::workspace::Workspace;
use jam_ready_cli::data::workspace::WorkspaceType::Client;

fn main() {

    // 加载工作区
    let workspace = Workspace::read();

    // 初始化颜色库
    colored::control::set_virtual_terminal(true).unwrap();

    // 查询器仅对客户端工作区有效
    if workspace.workspace_type == Client {

    }
}