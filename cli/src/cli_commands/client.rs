use crate::cli_commands::cli_command_client::{
    query_command::client_query,
    redirect_command::client_redirect,
    update_command::client_update,
    commit_command::client_commit,
    struct_command::client_struct,
    archive_command::client_archive,
    add_command::client_add,
    remove_command::client_remove,
    move_command::client_move,
    rollback_command::client_rollback,
    get_command::client_get,
    throw_command::client_throw,
    view_command::client_view,
    param_command::client_param,
};
use crate::data::client_result::ClientResult;
use crate::service::jam_client::execute;
use clap::{Args, CommandFactory, Parser, Subcommand};
use colored::Colorize;
use jam_ready::utils::text_process::parse_colored_text;
use std::env::args;

/// 客户端命令行
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

/// 客户端命令
#[derive(Subcommand, Debug)]
enum ClientCommands {

    #[command(
        hide = true,
        short_flag = 'h',
        long_flag = "help",
        about = "\nQuery commands")]
    Help,

    // 查询器
    #[command(
        subcommand,
        visible_alias = "q",
        about = "Query something")]
    Query(ClientQueryCommands),

    // 列出文件结构
    #[command(
        visible_alias = "tree",
        visible_alias = "list",
        visible_alias = "ls",
        about = "List the file struct of the workspace.\n\nLocal file operation commands")]
    Struct(StructArgs),

    // ---------------------------
    // 工作区相关

    // 重新定向至工作区
    #[command(
        visible_alias = "red",
        about = "Redirect to workspace."
    )]
    Redirect(RedirectArgs),

    // 同步文件结构
    #[command(
        visible_alias = "sync",
        about = "Sync the workspace file struct to local.\n\nWorkspace file operation commands")]
    Update,

    // ---------------------------
    // 文件操作

    // 提交取得锁的本地文件
    #[command(
        visible_alias = "cmt",
        visible_alias = "save",
        visible_alias = "sv",
        about = "Commit all modified files.")]
    Commit(CommitArgs),

    // 归档数据库版本 (仅 Leader)
    #[command(about = "Archive and backup workspace. [red](Leader only)[/]")]
    Archive,

    // 添加文件
    #[command(
        visible_alias = "new",
        visible_alias = "create",
        about = "Add a virtual file [gray](And get lock?)[/].")]
    Add(NewArgs),

    // 移除文件
    #[command(
        visible_alias = "rm",
        visible_alias = "delete",
        visible_alias = "del",
        about = "[gray](Try to get lock?)[/] Remove the virtual file.")]
    Remove(RemoveArgs),

    // 移动、重命名、或为文件重建映射
    #[command(
        visible_alias = "mv",
        visible_alias = "rename",
        about = "[gray](Try to get lock?)[/] Rename, move, or restore virtual file.")]
    Move(MoveArgs),

    // 移动、重命名、或为文件重建映射
    #[command(
        visible_alias = "rb",
        visible_alias = "restore",
        about = "[gray](Try to get lock?)[/] Rollback virtual file.")]
    Rollback(RollbackArgs),

    // 拿到文件的锁
    #[command(
        visible_alias = "g",
        visible_alias = "lock",
        about = "Get a [gray](longer?)[/] lock on a virtual file.")]
    Get(GetArgs),

    // 丢掉文件的锁
    #[command(
        visible_alias = "t",
        visible_alias = "unlock",
        visible_alias = "release",
        about = "Throw the lock on a virtual file.")]
    Throw(SearchArgs),

    // 下载并查看文件
    #[command(
        visible_alias = "v",
        visible_alias = "download",
        visible_alias = "dl",
        about = "[gray](Try to get lock?)[/] Download and view virtual file.\n\nOther")]
    View(ViewArgs),

    // ---------------------------
    // 其他操作

    // 操作参数
    #[command(
        visible_alias = "set",
        about = "Edit or view query parameters.")]
    Param(ParamArgs),

    #[command(hide = true)]
    Glock
}

/// 客户端查询命令
#[derive(Subcommand, Debug)]
pub enum ClientQueryCommands {

    // 列出某个目录下的结构
    #[command(
        visible_alias = "list",
        visible_alias = "ll",
        about = "List the structure under a specific directory")]
    ListDirectory(ListDirectoryArgs),

    // 查询虚拟文件的 Uuid
    #[command(
        visible_alias = "uuid",
        visible_alias = "uid",
        visible_alias = "id",
        visible_alias = "u",
        visible_alias = "i",
        about = "Query the Uuid of the virtual file")]
    FileUuid(StringArgs),

    // 查询虚拟文件的版本
    #[command(
        visible_alias = "version",
        visible_alias = "vsn",
        visible_alias = "v",
        about = "Query the version of the virtual file")]
    FileVersion(StringArgs),

    // 查询虚拟文件的路径
    #[command(
        visible_alias = "path",
        visible_alias = "fp",
        visible_alias = "p",
        about = "Query the path of the virtual file")]
    FilePath(StringArgs),

    // 查询虚拟文件的名称
    #[command(
        visible_alias = "name",
        visible_alias = "fn",
        visible_alias = "n",
        about = "Query the name of the virtual file")]
    FileName(StringArgs),

    // 查询虚拟文件的锁定状态
    #[command(
        visible_alias = "lock-status",
        visible_alias = "ls",
        about = "Query the lock status of the virtual file")]
    FileLockStatus(StringArgs),

    // 查询自己的 Uuid
    #[command(
        visible_alias = "me",
        about = "Query your Uuid")]
    SelfUuid,

    // 查询目标工作区地址
    #[command(
        visible_alias = "target-addr",
        visible_alias = "addr",
        visible_alias = "target",
        visible_alias = "t",
        about = "Query the address of the target workspace")]
    TargetAddress,

    // 查询目标工作区名称
    #[command(
        visible_alias = "ws",
        visible_alias = "w",
        about = "Query the name of the target workspace")]
    Workspace,

    // 查询虚拟文件是否在本地
    #[command(
        visible_alias = "cl",
        about = "Query whether the virtual file is local")]
    ContainLocal(StringArgs),

    // 查询本地文件映射的虚拟文件
    #[command(
        visible_alias = "ltr",
        about = "Query the local file mapped to the virtual file")]
    LocalToRemote(StringArgs),

    // 查询虚拟文件映射的本地文件
    #[command(
        visible_alias = "rtl",
        about = "Query the virtual file mapped to the local file")]
    RemoteToLocal(StringArgs),

    // 查询本地文件是否被更改
    #[command(
        visible_alias = "change",
        visible_alias = "c",
        about = "Query whether the local file has been changed")]
    Changed(StringArgs),

    // 查询本地文件的版本号
    #[command(
        visible_alias = "lv",
        about = "Query the version number of the local file")]
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

/// 新建目录
#[derive(Args, Debug)]
pub struct NewArgs {

    // 目录
    pub path: String,

    // 尝试拿到锁定
    #[arg(long, short = 'g', alias = "lock", alias = "l")]
    pub get: bool
}

/// 移除参数
#[derive(Args, Debug)]
pub struct RemoveArgs {

    // 搜索
    pub search: String,

    // 尝试拿到锁定
    #[arg(long, short = 'g', alias = "lock", alias = "l")]
    pub get: bool
}

/// 搜索 (Path or Uuid) 参数
#[derive(Args, Debug)]
pub struct SearchArgs {

    // 搜索
    pub search: String
}

#[derive(Args, Debug)]
pub struct ViewArgs {

    // 搜索
    pub search: String,

    // 指定查看的版本
    #[arg(short, long)]
    pub version: Option<u32>,

    // 尝试拿到锁定
    #[arg(long, short = 'g', alias = "lock", alias = "l")]
    pub get: bool
}

#[derive(Args, Debug)]
pub struct GetArgs {

    // 搜索
    pub search: String,

    // 是否为长期锁
    #[arg(short = 'l', long = "longer")]
    pub longer: bool
}

/// 搜索 (Path or Uuid) 参数
#[derive(Args, Debug)]
pub struct MoveArgs {

    // 搜索
    pub from_search: String,

    // 移动到
    pub to_path: String,

    // 尝试拿到锁定
    #[arg(long, short = 'g', alias = "lock")]
    pub get: bool,

    // 仅移动本地文件
    #[arg(long, short = 'l')]
    pub local: bool
}

/// 回滚参数
#[derive(Args, Debug)]
pub struct RollbackArgs {

    // 搜索
    pub search: String,

    // 回滚的版本
    pub version: u32,

    // 尝试拿到锁定
    #[arg(long, short = 'g', alias = "lock")]
    pub get: bool,

    // 完成后将该文件回滚到老版本
    #[arg(long, short = 'b')]
    pub back: bool,
}

#[derive(Args, Debug)]
pub struct CommitArgs {

    // 日志
    pub log: Option<String>
}

#[derive(Args, Debug)]
pub struct ParamArgs {

    // 键
    pub key: Option<String>,

    // 值
    pub value: Option<String>
}

#[derive(Args, Debug)]
pub struct StructArgs {

    // 显示本地文件
    #[arg(long)]
    pub local: bool,

    // 显示远程文件
    #[arg(long)]
    pub remote: bool,

    // -- 仅远程

    // 显示空文件
    #[arg(long = "zero", short = 'z', alias = "empty", alias = "new")]
    pub remote_zero: bool,

    // 显示更新的文件
    #[arg(long = "updated", short = 'u')]
    pub remote_updated: bool,

    // 显示持有的文件
    #[arg(long = "held", short = 'h')]
    pub remote_held: bool,

    // 显示锁定的文件
    #[arg(long = "lock", short = 'g')]
    pub remote_locked: bool,

    // 显示其他文件
    #[arg(long = "other", short = 'e')]
    pub remote_other: bool,

    // -- 仅本地

    // 显示删除(但本地仍存在)的文件
    #[arg(long = "removed", short = 'd')]
    pub local_removed: bool,

    // 显示删除的文件
    #[arg(long = "untracked", short = 'n')]
    pub local_untracked: bool,

    // -- 通用

    // 显示移动的文件 (根据 remote 和 local 的开关选择显示侧)
    #[arg(long = "moved", short = 'm')]
    pub moved: bool,
}

#[derive(Args, Debug)]
pub struct RedirectArgs {

    // 用户登录口令，用于识别身份
    #[arg(short, long = "code")]
    pub login_code: Option<String>,

    // 目标地址 (直接指定)
    #[arg(short, long)]
    pub target: Option<String>,

    // 工作区名称 (由网络发现获取目标地址)
    #[arg(short, long)]
    pub workspace: Option<String>,
}

pub async fn client_workspace_main() {

    if args().count() <= 1 {
        client_print_helps();
        return;
    }

    let cmd = ClientWorkspaceEntry::parse();

    match cmd.command {

        // 帮助
        ClientCommands::Help => client_print_helps(),

        // 查询
        ClientCommands::Query(command) => client_query(command).await,

        // 重定向至工作区
        ClientCommands::Redirect(args) => client_redirect(args).await,

        // 更新
        ClientCommands::Update => client_update().await,

        // 提交
        ClientCommands::Commit(args) => client_commit(args).await,

        // 列出结构
        ClientCommands::Struct(args) => client_struct(args).await,

        // 归档
        ClientCommands::Archive => client_archive().await,

        // 添加文件
        ClientCommands::Add(args) => client_add(args).await,

        // 移除文件
        ClientCommands::Remove(args) => client_remove(args).await,

        // 移动文件
        ClientCommands::Move(args) => client_move(args).await,

        // 回滚文件
        ClientCommands::Rollback(args) => client_rollback(args).await,

        // 获得锁
        ClientCommands::Get(args) => client_get(args).await,

        // 丢掉锁
        ClientCommands::Throw(args) => client_throw(args).await,

        // 查看锁
        ClientCommands::View(args) => client_view(args).await,

        // 参数
        ClientCommands::Param(args) => client_param(args).await,

        // 格洛克？？？
        ClientCommands::Glock => print_glock_xd(),
    }
}

/// 打印客户端帮助
fn client_print_helps() {
    let commands = ClientWorkspaceEntry::command();

    // 打印单个命令
    for subcommand in commands.get_subcommands() {

        if subcommand.is_hide_set() { continue };

        // 命令名称
        let command_name = subcommand.get_name();
        if command_name == "help" {
            println!("Query commands\n");
            continue;
        }
        print!("    {}", command_name.cyan());

        let mut args_str = String::new();
        // 命令参数
        for arg in subcommand.get_arguments() {

            // 必填参数
            if arg.is_required_set() {
                args_str.push_str(format!(" [green]<{}>[/]", arg.get_id().to_string().to_uppercase()).as_str());
            } else {

                // 可选参数
                let long = arg.get_long();
                let short = arg.get_short();
                if let Some(long) = long {
                    args_str.push_str(format!(" [yellow][--{}", long).as_str());
                }
                if let Some(short) = short {
                    let split = if long.is_some() { ", -" } else { "[yellow][" };
                    args_str.push_str(format!("{}{}", split, short).as_str());
                }
                if long.is_some() || short.is_some() {
                    args_str.push_str("][/]");
                }
            }
        }
        print!("{}", parse_colored_text(args_str.as_str()));

        // 别名
        let aliases = subcommand.get_visible_aliases();
        if aliases.count() > 0 {
            let mut aliases_str = String::new();
            aliases_str.push_str("[gray](");
            for alias in subcommand.get_visible_aliases() {
                aliases_str.push_str(format!("{}, ", alias).as_str());
            }
            aliases_str = aliases_str.trim().trim_end_matches(',').to_string();
            aliases_str.push_str(")[/]");
            print!(" {}", parse_colored_text(aliases_str.as_str()));
        }

        // 描述
        if let Some(about) = subcommand.get_about() {
            print!("\n        {}", parse_colored_text(about.to_string().as_str()));
        }

        // 末尾换行
        println!();
        println!();
    }
}

/// 客户端运行命令
pub async fn exec(args: Vec<String>) -> Option<ClientResult> {
    execute(args).await
}

/// 打印客户端结果
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