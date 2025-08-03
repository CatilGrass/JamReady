use crate::cli_commands::client::{client_workspace_main, ClientWorkspaceEntry};
use crate::cli_commands::server::server_workspace_main;
use crate::data::local_file_map::LocalFileMap;
use crate::data::workspace::WorkspaceType::{Client, Server, Unknown};
use crate::data::workspace::{ClientWorkspace, ServerWorkspace, Workspace};
use crate::service::jam_client::search_workspace_lan;
use clap::{Args, CommandFactory, Parser, Subcommand};
use clap_complete::generate;
use jam_ready::utils::address_str_parser::parse_address_v4_str;
use jam_ready::utils::hide_folder::hide_folder;
use jam_ready::utils::local_archive::LocalArchive;
use jam_ready::utils::text_process::{parse_colored_text, process_id_text_not_to_lower};
use std::collections::HashMap;
use std::env::{args, current_dir};
use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

/// 建立工作区入口
#[derive(Parser, Debug)]
#[command(
    disable_help_flag = true,
    disable_version_flag = true,
    disable_help_subcommand = true,
    help_template = "{all-args}"
)]
struct WorkspaceSetup {
    #[command(subcommand)]
    command: WorkspaceSetupCommands,
}

/// 建立工作区用指令
#[derive(Subcommand, Debug)]
enum WorkspaceSetupCommands {

    // 生成补全脚本
    GenerateClientCompletions {
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },

    // 登录到工作区
    #[command(about = "Login to workspace")]
    Login(ClientSetupArgs),

    // 建立新的工作区
    #[command(about = "Setup workspace")]
    Setup(ServerSetupArgs),
}

/// 客户端建立参数
#[derive(Args, Debug)]
struct ClientSetupArgs {

    // 用户登录口令，用于识别身份
    login_code: String,

    // 目标地址 (直接指定)
    #[arg(short, long)]
    target: Option<String>,

    // 工作区名称 (由网络发现获取目标地址)
    #[arg(short, long)]
    workspace: Option<String>,
}

/// 服务端建立参数
#[derive(Args, Debug)]
struct ServerSetupArgs {

    // 工作区名称，服务端必填
    workspace: String,

    // 端口设定，可选
    #[arg(short, long)]
    port: Option<u16>
}

/// 建立工作区
async fn setup_workspace_main(workspace: Workspace) {

    if args().len() <= 1 {
        setup_print_help();
        return;
    }

    let cmd = WorkspaceSetup::parse();
    match cmd.command {

        // 生成补全脚本
        WorkspaceSetupCommands::GenerateClientCompletions { shell } => {
            let mut cmd = ClientWorkspaceEntry::command();
            generate(shell, &mut cmd, "jam", &mut io::stdout());
        }

        // 建立客户端工作区
        WorkspaceSetupCommands::Login(args) => setup_client_workspace(args, workspace).await,

        // 建立服务端工作区
        WorkspaceSetupCommands::Setup(args) => setup_server_workspace(args, workspace).await,
    }
}

fn setup_print_help() {
    println!("{}", parse_colored_text("\
Login to Workspace
==================
Use either of these commands:

1. By workspace name:
   [green]jam login[/] [yellow]<YOUR_LOGIN_CODE>[/] [cyan]--workspace[/] [yellow]<TARGET_WORKSPACE_NAME>[/]

2. By workspace address:
   [green]jam login[/] [yellow]<YOUR_LOGIN_CODE>[/] [cyan]--target[/] [yellow]<TARGET_WORKSPACE_ADDRESS>[/]

Examples:
• [gray]jam login XXXX-XXXX[/] [cyan]--workspace[/] MyWorkspace
• [gray]jam login XXXX-XXXX[/] [cyan]--target[/] localhost:5011

Setup Workspace
===============
Create a new workspace with:
   [green]jam setup[/] [yellow]<YOUR_WORKSPACE_NAME>[/]

Example:
   [gray]jam setup[/] MyWorkspace
   "))
}

async fn setup_client_workspace(args: ClientSetupArgs, mut workspace: Workspace) {
    workspace.workspace_type = Client;

    // 如果 目标地址 不存在，且 工作区 也没有指定，则无法创建工作区
    if args.target.is_none() && args.workspace.is_none() {
        eprintln!("You need to specify a target or workspace");
        eprintln!("\"--workspace <NAME>\" or \"--target <ADDRESS>\"");
        return;
    }

    // 工作区名称
    let workspace_name =
        process_id_text_not_to_lower(args.workspace.unwrap_or("Workspace".to_string()));

    let client = ClientWorkspace {

        // 工作区名称
        workspace_name: workspace_name.clone(),

        // 目标地址
        target_addr: if let Some(addr) = args.target {

            // 知道地址，通过 DNS 解析具体地址
            parse_address_v4_str(addr).await
                .unwrap_or(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 5011))
        } else {

            // 未知, 尝试网络发现
            let addr = search_workspace_lan(workspace_name).await;
            if let Ok(addr) = addr {
                addr
            } else {

                // 返回默认地址
                SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 5011)
            }
        },

        // 登录口令
        login_code: args.login_code.trim().to_string(),

        uuid: "".to_string()
    };
    workspace.client = Some(client);

    println!("Client workspace has been established");

    // 写入
    Workspace::update(&mut workspace).await;

    // 开始构建本地映射表
    LocalFileMap::update(&LocalFileMap::read().await).await;

    // 隐藏 .jam 文件夹
    let jam_folder = current_dir().unwrap().join(env!("PATH_WORKSPACE_ROOT"));
    let _ = hide_folder(&jam_folder);
}

async fn setup_server_workspace(args: ServerSetupArgs, mut workspace: Workspace) {
    workspace.workspace_type = Server;
    let server = ServerWorkspace {

        // 工作区名称
        workspace_name: args.workspace,

        members: HashMap::new(),
        member_uuids: HashMap::new(),
        login_code_map: HashMap::new(),
        enable_debug_logger: false,
    };
    workspace.server = Some(server);

    println!("Server workspace has been established");

    // 写入
    Workspace::update(&mut workspace).await;

    // 隐藏 .jam 文件夹
    let jam_folder = current_dir().unwrap().join(env!("PATH_WORKSPACE_ROOT"));
    let _ = hide_folder(&jam_folder);
}

pub async fn cli_entry() {

    // 加载工作区
    let workspace = Workspace::read().await;

    // 初始化颜色库
    colored::control::set_virtual_terminal(true).unwrap();

    // 若未初始化工作区，则引导用户初始化
    if workspace.workspace_type == Unknown {
        setup_workspace_main(workspace).await;
    } else if workspace.workspace_type == Client {
        client_workspace_main().await;
    } else if workspace.workspace_type == Server {
        server_workspace_main().await;
    }
}