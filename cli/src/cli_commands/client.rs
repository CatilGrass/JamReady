use crate::cli_commands::cli_command_client::{
    add_command::client_add,
    archive_command::client_archive,
    complete_command::client_complete,
    commit_command::client_commit,
    get_command::client_get,
    move_command::client_move,
    param_command::client_param,
    query_command::client_query,
    redirect_command::client_redirect,
    remove_command::client_remove,
    rollback_command::client_rollback,
    struct_command::client_struct,
    throw_command::client_throw,
    update_command::client_update,
    view_command::client_view,
    doc_command::client_doc,
};
use crate::data::client_result::ClientResult;
use crate::help::help_docs::get_help_docs;
use crate::service::jam_client::execute;
use clap::{Args, Parser, Subcommand};
use std::env::args;

/// Client command line interface
#[derive(Parser, Debug)]
#[command(
    disable_help_flag = true,
    disable_version_flag = true,
    disable_help_subcommand = true,
    help_template = "{all-args}"
)]
pub struct ClientWorkspaceEntry {
    #[command(subcommand)]
    command: ClientCommands,
}

/// Client commands
#[derive(Subcommand, Debug)]
enum ClientCommands {

    #[command(
        hide = true,
        short_flag = 'h',
        long_flag = "help")]
    Help,

    // Query tool
    #[command(
        subcommand,
        visible_alias = "q")]
    Query(ClientQueryCommands),

    // List file structure
    #[command(
        visible_alias = "tree",
        visible_alias = "list",
        visible_alias = "ls")]
    Struct(StructArgs),

    // ---------------------------
    // Workspace related

    // Redirect to workspace
    #[command(visible_alias = "red")]
    Redirect(RedirectArgs),

    // Sync file structure
    #[command(visible_alias = "sync")]
    Update(UpdateArgs),

    // ---------------------------
    // File operations

    // Mark a file as completed
    #[command(
        visible_alias = "cmpl",
        visible_alias = "c",
        visible_alias = "done",
        visible_alias = "d",
    )]
    Complete(CompleteArgs),

    // Commit locked local files
    #[command(
        visible_alias = "cmt",
        visible_alias = "save",
        visible_alias = "sv"
    )]
    Commit,

    // Archive database version (Leader only)
    Archive,

    // Add file
    #[command(
        visible_alias = "new",
        visible_alias = "create"
    )]
    Add(NewArgs),

    // Remove file
    #[command(
        visible_alias = "rm",
        visible_alias = "delete",
        visible_alias = "del"
    )]
    Remove(RemoveArgs),

    // Move, rename, or remap file
    #[command(
        visible_alias = "mv",
        visible_alias = "rename"
    )]
    Move(MoveArgs),

    // Rollback file version
    #[command(
        visible_alias = "rb",
        visible_alias = "restore"
    )]
    Rollback(RollbackArgs),

    // Acquire file lock
    #[command(
        visible_alias = "g",
        visible_alias = "lock"
    )]
    Get(GetArgs),

    // Release file lock
    #[command(
        visible_alias = "t",
        visible_alias = "unlock",
        visible_alias = "release"
    )]
    Throw(SearchArgs),

    // Download and view file
    #[command(
        visible_alias = "v",
        visible_alias = "download",
        visible_alias = "dl"
    )]
    View(ViewArgs),

    // Query documentation
    Doc(DocArgs),

    // ---------------------------
    // Other operations

    // Manage parameters
    #[command(
        visible_alias = "set"
    )]
    Param(ParamArgs),

    #[command(hide = true)]
    Glock
}

// Client query commands
#[derive(Subcommand, Debug)]
pub enum ClientQueryCommands {

    // List directory structure
    #[command(
        visible_alias = "list",
        visible_alias = "ll"
    )]
    ListDirectory(ListDirectoryArgs),

    // Query virtual file Uuid
    #[command(
        visible_alias = "uuid",
        visible_alias = "uid",
        visible_alias = "id",
        visible_alias = "u",
        visible_alias = "i"
    )]
    FileUuid(StringArgs),

    // Query virtual file version
    #[command(
        visible_alias = "version",
        visible_alias = "vsn",
        visible_alias = "v"
    )]
    FileVersion(StringArgs),

    // Query virtual file path
    #[command(
        visible_alias = "path",
        visible_alias = "fp",
        visible_alias = "p",
    )]
    FilePath(StringArgs),

    // Query virtual file name
    #[command(
        visible_alias = "name",
        visible_alias = "fn",
        visible_alias = "n",
    )]
    FileName(StringArgs),

    // Query virtual file lock status
    #[command(
        visible_alias = "lock-status",
        visible_alias = "ls",
    )]
    FileLockStatus(StringArgs),

    // Query self Uuid
    #[command(
        visible_alias = "me",
    )]
    SelfUuid,

    // Query target workspace address
    #[command(
        visible_alias = "target-addr",
        visible_alias = "addr",
        visible_alias = "target",
        visible_alias = "t",
    )]
    TargetAddress,

    // Query target workspace name
    #[command(
        visible_alias = "ws",
        visible_alias = "w",
    )]
    Workspace,

    // Check if virtual file exists locally
    #[command(
        visible_alias = "cl",
    )]
    ContainLocal(StringArgs),

    // Map local file to virtual file
    #[command(
        visible_alias = "ltr",
    )]
    LocalToRemote(StringArgs),

    // Map virtual file to local file
    #[command(
        visible_alias = "rtl",
    )]
    RemoteToLocal(StringArgs),

    // Check if local file has changed
    #[command(
        visible_alias = "change",
        visible_alias = "c",
    )]
    Changed(StringArgs),

    // Query local file version
    #[command(
        visible_alias = "lv",
    )]
    LocalVersion(StringArgs)
}

#[derive(Args, Debug)]
pub struct StringArgs {
    #[arg(default_value = "")]
    pub value: String,
}

#[derive(Args, Debug)]
pub struct ListDirectoryArgs {
    #[arg(default_value = "")]
    pub value: String,

    #[arg(long, short = 'i')]
    pub completion_mode: bool
}

/// Create new directory
#[derive(Args, Debug)]
pub struct NewArgs {
    // Directory path
    pub path: String,

    // Attempt to acquire lock
    #[arg(long, short = 'g', alias = "lock", alias = "l")]
    pub get: bool
}

/// Remove parameters
#[derive(Args, Debug)]
pub struct RemoveArgs {
    // Search term
    pub from_search: String,

    // Attempt to acquire lock
    #[arg(long, short = 'g', alias = "lock", alias = "l")]
    pub get: bool
}

/// Search (Path or Uuid) parameters
#[derive(Args, Debug)]
pub struct SearchArgs {
    // Search term
    pub search: String
}

#[derive(Args, Debug)]
pub struct ViewArgs {
    // Search term
    pub from_search: String,

    // Specific version to view
    #[arg(short, long)]
    pub version: Option<u32>,

    // Attempt to acquire lock
    #[arg(long, short = 'g', alias = "lock", alias = "l")]
    pub get: bool
}

#[derive(Args, Debug)]
pub struct DocArgs {
    // Documentation name
    pub doc_name: String
}

#[derive(Args, Debug)]
pub struct GetArgs {
    // Search term
    pub from_search: String,

    // Long-term lock
    #[arg(short = 'l', long = "longer")]
    pub longer: bool
}

/// Search (Path or Uuid) parameters
#[derive(Args, Debug)]
pub struct MoveArgs {
    // Search term
    pub from_search: String,

    // Destination
    pub to_search: String,

    // Attempt to acquire lock
    #[arg(long, short = 'g', alias = "lock")]
    pub get: bool,

    // Only move local file
    #[arg(long, short = 'l')]
    pub local: bool
}

/// Rollback parameters
#[derive(Args, Debug)]
pub struct RollbackArgs {
    // Search term
    pub from_search: String,

    // Version to rollback to
    pub to_version: u32,

    // Attempt to acquire lock
    #[arg(long, short = 'g', alias = "lock")]
    pub get: bool,

    // Download file after rollback
    #[arg(long, short = 'b')]
    pub back: bool,
}

#[derive(Args, Debug)]
pub struct CompleteArgs {
    // Search term
    pub from_search: String,

    // Commit message
    pub info: Option<String>
}

#[derive(Args, Debug)]
pub struct ParamArgs {
    // Key
    pub key: Option<String>,

    // Value
    pub value: Option<String>
}

#[derive(Args, Debug)]
pub struct StructArgs {
    // Show local files
    #[arg(long)]
    pub local: bool,

    // Show remote files
    #[arg(long)]
    pub remote: bool,

    // -- Remote only

    // Show empty files
    #[arg(long = "zero", short = 'z', alias = "empty", alias = "new")]
    pub remote_zero: bool,

    // Show updated files
    #[arg(long = "updated", short = 'u')]
    pub remote_updated: bool,

    // Show held files
    #[arg(long = "held", short = 'h')]
    pub remote_held: bool,

    // Show locked files
    #[arg(long = "lock", short = 'g')]
    pub remote_locked: bool,

    // Show other files
    #[arg(long = "other", short = 'e')]
    pub remote_other: bool,

    // -- Local only

    // Show removed (but still existing locally) files
    #[arg(long = "removed", short = 'd')]
    pub local_removed: bool,

    // Show untracked files
    #[arg(long = "untracked", short = 'n')]
    pub local_untracked: bool,

    // Show untracked files
    #[arg(long = "completed", short = 'c')]
    pub local_completed: bool,

    // -- General

    // Show moved files (based on remote/local switches)
    #[arg(long = "moved", short = 'm')]
    pub moved: bool,
}

#[derive(Args, Debug)]
pub struct RedirectArgs {
    // User login code for authentication
    #[arg(short, long = "code")]
    pub login_code: Option<String>,

    // Target address (direct specification)
    #[arg(short, long)]
    pub target: Option<String>,

    // Workspace name (discover target address via network)
    #[arg(short, long)]
    pub workspace: Option<String>,
}

#[derive(Args, Debug)]
pub struct UpdateArgs {

    #[arg(short = 's', long = "struct")]
    pub file_struct : bool,

    #[arg(short = 'd', long = "database", alias = "db", default_value = "true")]
    pub database : bool,
}

pub async fn client_workspace_main() {
    if args().count() <= 1 {
        client_print_helps();
        return;
    }

    let cmd = ClientWorkspaceEntry::parse();

    match cmd.command {
        ClientCommands::Help => client_print_helps(),

        ClientCommands::Query(command) => client_query(command).await,

        // Redirect to workspace
        ClientCommands::Redirect(args) => client_redirect(args).await,

        ClientCommands::Update(args) => client_update(args).await,

        ClientCommands::Complete(args) => client_complete(args).await,

        ClientCommands::Commit => client_commit().await,

        ClientCommands::Struct(args) => client_struct(args).await,

        ClientCommands::Archive => client_archive().await,

        ClientCommands::Add(args) => client_add(args).await,

        ClientCommands::Remove(args) => client_remove(args).await,

        ClientCommands::Move(args) => client_move(args).await,

        ClientCommands::Rollback(args) => client_rollback(args).await,

        ClientCommands::Get(args) => client_get(args).await,

        ClientCommands::Throw(args) => client_throw(args).await,

        ClientCommands::View(args) => client_view(args).await,

        ClientCommands::Param(args) => client_param(args).await,

        ClientCommands::Doc(args) => client_doc(args).await,

        // Glock???
        ClientCommands::Glock => print_glock_xd(),
    }
}

/// Print client help
fn client_print_helps() {
    println!("{}", get_help_docs("client_help"));
}

/// Execute client command
pub async fn exec(args: Vec<String>) -> Option<ClientResult> {
    execute(args).await
}

/// Print client result
pub fn print_client_result(result : Option<ClientResult>) {
    if let Some(result) = result {
        result.end_print()
    }
}

fn print_glock_xd() {
    println!("{}", "\
It's a glock :)
    ▄▬▬█▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬█▬▄
   ▌▓▌▌▌▌▌▌▌▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▌
   ▌▓▌▌▌▌▌▌▌▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▐
   ▌▓▌▌▌▌▌▌▌▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▐
   ▌▓▌▌▌▌▌▌▌▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▌
  ▄█▬▬▬▬▬▄▄▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▐
    █▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▄▬▀
     █▒▓▓▓▓▓▓▓▓▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▌
      █▒▓▓▓▓▓▓█▬▄▬▬▬▬▬▬▄▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▬▀
      █▒▓▓▓▓▓▓▓▓█  ▐      ▌
     █▒▓▓▓▓▓▓▓▓█ ▌  ▌     ▌
     █▒▓▓▓▓▓▓▓▓█  ▬▬      ▐
     █▒▓▓▓▓▓▓▓▓█▀▬▬▬▬▬▬▬▬▬▀
    █▒▓▓▓▓▓▓▓▓█
    █▒▓▓▓▓▓▓▓▓█
   █▒▓▓▓▓▓▓▓▓█
   █▒▓▓▓▓▓▓▓▓█
   █▒▓▓▓▓▓▓▓▓█
  █▒▓▓▓▓▓▓▓▓█
  ▀▬▄▬▬▬▬▬▬▄█
    ▀▬▬▬▬▬▬▀
    ");
}