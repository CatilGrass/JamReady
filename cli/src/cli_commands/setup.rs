use crate::cli_commands::client::client_workspace_main;
use crate::cli_commands::server::server_workspace_main;
use crate::data::local_file_map::LocalFileMap;
use crate::data::workspace::WorkspaceType::{Client, Server, Unknown};
use crate::data::workspace::{ClientWorkspace, ServerWorkspace, Workspace};
use crate::service::jam_client::search_workspace_lan;
use clap::{Args, Parser, Subcommand};
use jam_ready::utils::address_str_parser::parse_address_v4_str;
use jam_ready::utils::hide_folder::hide_folder;
use jam_ready::utils::local_archive::LocalArchive;
use jam_ready::utils::text_process::{process_id_text_not_to_lower};
use std::collections::HashMap;
use std::env::{args, current_dir};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use crate::help::help_docs::get_help_docs;

/// Workspace setup entry point
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

/// Workspace setup commands
#[derive(Subcommand, Debug)]
enum WorkspaceSetupCommands {
    // Login to workspace
    Login(ClientSetupArgs),

    // Create new workspace
    Setup(ServerSetupArgs),
}

/// Client setup arguments
#[derive(Args, Debug)]
struct ClientSetupArgs {
    // User login code for identity verification
    login_code: String,

    // Target address (direct specification)
    #[arg(short, long)]
    target: Option<String>,

    // Workspace name (discover target address via network)
    #[arg(short, long)]
    workspace: Option<String>,

    // Enable debug mode
    #[arg(long)]
    debug: bool
}

/// Server setup arguments
#[derive(Args, Debug)]
struct ServerSetupArgs {
    // Workspace name (required for server)
    workspace: String,

    // Port setting (optional)
    #[arg(short, long)]
    port: Option<u16>
}

/// Setup workspace
async fn setup_workspace_main(workspace: Workspace) {
    if args().len() <= 1 {
        setup_print_help();
        return;
    }

    let cmd = WorkspaceSetup::parse();
    match cmd.command {
        // Setup client workspace
        WorkspaceSetupCommands::Login(args) => setup_client_workspace(args, workspace).await,

        // Setup server workspace
        WorkspaceSetupCommands::Setup(args) => setup_server_workspace(args, workspace).await,
    }
}

fn setup_print_help() {
    println!("{}", get_help_docs("setup_help"))
}

async fn setup_client_workspace(args: ClientSetupArgs, mut workspace: Workspace) {
    workspace.workspace_type = Client;

    // If target address is not specified and workspace name is also not specified, cannot create workspace
    if args.target.is_none() && args.workspace.is_none() {
        eprintln!("You need to specify a target or workspace");
        eprintln!("\"--workspace <NAME>\" or \"--target <ADDRESS>\"");
        return;
    }

    // Workspace name
    let workspace_name =
        process_id_text_not_to_lower(args.workspace.unwrap_or("Workspace".to_string()));

    let client = ClientWorkspace {
        // Workspace name
        workspace_name: workspace_name.clone(),

        // Target address
        target_addr: if let Some(addr) = args.target {
            // Known address, resolve via DNS
            parse_address_v4_str(addr).await
                .unwrap_or(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 5011))
        } else {
            // Unknown, try network discovery
            let addr = search_workspace_lan(workspace_name).await;
            if let Ok(addr) = addr {
                addr
            } else {
                // Fallback to default address
                SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 5011)
            }
        },

        // Login code
        login_code: args.login_code.trim().to_string(),

        uuid: "".to_string(),

        debug: args.debug,
    };
    workspace.client = Some(client);

    println!("Client workspace has been established");

    // Write configuration
    Workspace::update(&mut workspace).await;

    // Initialize local file mapping
    LocalFileMap::update(&LocalFileMap::read().await).await;

    // Hide .jam folder
    let jam_folder = current_dir().unwrap().join(env!("PATH_WORKSPACE_ROOT"));
    let _ = hide_folder(&jam_folder);
}

async fn setup_server_workspace(args: ServerSetupArgs, mut workspace: Workspace) {
    workspace.workspace_type = Server;
    let server = ServerWorkspace {
        // Workspace name
        workspace_name: args.workspace,

        members: HashMap::new(),
        member_uuids: HashMap::new(),
        login_code_map: HashMap::new(),
        enable_debug_logger: true,
    };
    workspace.server = Some(server);

    println!("Server workspace has been established");

    // Write configuration
    Workspace::update(&mut workspace).await;

    // Hide .jam folder
    let jam_folder = current_dir().unwrap().join(env!("PATH_WORKSPACE_ROOT"));
    let _ = hide_folder(&jam_folder);
}

pub async fn cli_entry() {
    // Load workspace
    let workspace = Workspace::read().await;

    // Initialize color library (Windows only)
    #[cfg(windows)]
    colored::control::set_virtual_terminal(true).unwrap();

    // If workspace is not initialized, guide user through setup
    if workspace.workspace_type == Unknown {
        setup_workspace_main(workspace).await;
    } else if workspace.workspace_type == Client {
        client_workspace_main().await;
    } else if workspace.workspace_type == Server {
        server_workspace_main().await;
    }
}