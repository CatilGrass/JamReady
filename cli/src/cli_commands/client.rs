use crate::data::database::Database;
use crate::data::local_file_map::LocalFileMap;
use crate::data::local_folder_map::{LocalFolderMap, Node};
use crate::data::parameters::{erase_parameter, parameters, read_parameter, write_parameter};
use crate::data::workspace::Workspace;
use crate::service::jam_client::{execute, search_workspace_lan};
use clap::{Args, CommandFactory, Parser, Subcommand};
use colored::Colorize;
use jam_ready::utils::address_str_parser::parse_address_v4_str;
use jam_ready::utils::file_digest::md5_digest;
use jam_ready::utils::local_archive::LocalArchive;
use jam_ready::utils::text_process::{parse_colored_text, process_path_text};
use std::env::{args, current_dir};
use std::ops::Add;
use jam_ready::utils::file_operation::move_file;
use crate::data::client_result::{ClientResult, ClientResultQueryProcess};

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
        about = "[gray](Try to get lock?)[/] Download and view virtual file.")]
    View(ViewArgs),

    // 下载所有文件
    #[command(
        visible_alias = "c",
        visible_alias = "build",
        about = "Download all virtual files.\n\nOther")]
    Clone,

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
enum ClientQueryCommands {

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
struct StringArgs {

    #[arg(default_value = "")]
    value: String,
}

#[derive(Args, Debug)]
struct ListDirectoryArgs {

    #[arg(default_value = "")]
    value: String,

    #[arg(long, short = 'i')]
    completion_mode: bool
}

/// 新建目录
#[derive(Args, Debug)]
struct NewArgs {

    // 目录
    path: String,

    // 尝试拿到锁定
    #[arg(long, short = 'g', alias = "lock", alias = "l")]
    get: bool
}

/// 移除参数
#[derive(Args, Debug)]
struct RemoveArgs {

    // 搜索
    search: String,

    // 尝试拿到锁定
    #[arg(long, short = 'g', alias = "lock", alias = "l")]
    get: bool
}

/// 搜索 (Path or Uuid) 参数
#[derive(Args, Debug)]
struct SearchArgs {

    // 搜索
    search: String
}

#[derive(Args, Debug)]
struct ViewArgs {

    // 搜索
    search: String,

    // 指定查看的版本
    #[arg(short, long)]
    version: Option<u32>,

    // 尝试拿到锁定
    #[arg(long, short = 'g', alias = "lock", alias = "l")]
    get: bool
}

#[derive(Args, Debug)]
struct GetArgs {

    // 搜索
    search: String,

    // 是否为长期锁
    #[arg(short = 'l', long = "longer")]
    longer: bool
}

/// 搜索 (Path or Uuid) 参数
#[derive(Args, Debug)]
struct MoveArgs {

    // 搜索
    from_search: String,

    // 移动到
    to_path: String,

    // 尝试拿到锁定
    #[arg(long, short = 'g', alias = "lock")]
    get: bool,

    // 仅移动本地文件
    #[arg(long, short = 'l')]
    local: bool
}

/// 回滚参数
#[derive(Args, Debug)]
struct RollbackArgs {

    // 搜索
    search: String,

    // 回滚的版本
    version: u32,

    // 尝试拿到锁定
    #[arg(long, short = 'g', alias = "lock")]
    get: bool,

    // 完成后将该文件回滚到老版本
    #[arg(long, short = 'b')]
    back: bool,
}

#[derive(Args, Debug)]
struct CommitArgs {

    // 日志
    log: Option<String>
}

#[derive(Args, Debug)]
struct ParamArgs {

    // 键
    key: Option<String>,

    // 值
    value: Option<String>
}

#[derive(Args, Debug)]
struct StructArgs {

    // 显示本地文件
    #[arg(long)]
    local: bool,

    // 显示远程文件
    #[arg(long)]
    remote: bool,

    // -- 仅远程

    // 显示空文件
    #[arg(long = "zero", short = 'z', alias = "empty", alias = "new")]
    remote_zero: bool,

    // 显示更新的文件
    #[arg(long = "updated", short = 'u')]
    remote_updated: bool,

    // 显示持有的文件
    #[arg(long = "held", short = 'h')]
    remote_held: bool,

    // 显示锁定的文件
    #[arg(long = "lock", short = 'g')]
    remote_locked: bool,

    // 显示其他文件
    #[arg(long = "other", short = 'e')]
    remote_other: bool,

    // -- 仅本地

    // 显示删除(但本地仍存在)的文件
    #[arg(long = "removed", short = 'd')]
    local_removed: bool,

    // 显示删除的文件
    #[arg(long = "untracked", short = 'n')]
    local_untracked: bool,

    // -- 通用

    // 显示移动的文件 (根据 remote 和 local 的开关选择显示侧)
    #[arg(long = "moved", short = 'm')]
    moved: bool,
}

#[derive(Args, Debug)]
struct RedirectArgs {

    // 用户登录口令，用于识别身份
    #[arg(short, long = "code")]
    login_code: Option<String>,

    // 目标地址 (直接指定)
    #[arg(short, long)]
    target: Option<String>,

    // 工作区名称 (由网络发现获取目标地址)
    #[arg(short, long)]
    workspace: Option<String>,
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
        ClientCommands::Update => {
            print_client_result(exec(vec!["update".to_string()]).await);
        }

        // 提交
        ClientCommands::Commit(args) => {
            if let Some(log) = args.log {
                print_client_result(exec(vec!["commit".to_string(), log]).await);
            } else {
                print_client_result(exec(vec!["commit".to_string()]).await);
            }
        }

        // 列出结构
        ClientCommands::Struct(args) => {
            let mut env_flags = String::new();
            let mut flags = String::new();

            if args.local { env_flags.push_str("l"); }
            if args.remote { env_flags.push_str("r"); }
            if env_flags.is_empty() {
                env_flags = "lr".to_string();
            }

            if args.remote_zero { flags.push_str("z"); }
            if args.remote_held { flags.push_str("h"); }
            if args.remote_updated { flags.push_str("u"); }
            if args.local_untracked { flags.push_str("n"); }
            if args.local_removed { flags.push_str("d"); }
            if args.remote_locked { flags.push_str("g"); }
            if args.moved { flags.push_str("m"); }
            if args.remote_other { flags.push_str("e"); }
            if flags.is_empty() {
                flags = "zhundgme".to_string();
            }

            print_client_result(exec(vec!["struct".to_string(), env_flags, flags]).await);
        },

        // 归档
        ClientCommands::Archive => {
            print_client_result(exec(vec!["archive".to_string()]).await);
        }

        // 添加文件
        ClientCommands::Add(args) => {
            let mut result = ClientResult::result().await;

            // 添加文件
            result.combine_unchecked(exec(vec!["file".to_string(), "add".to_string(), args.path.clone()]).await);
            if args.get {
                // 获得文件的锁
                result.combine_unchecked(exec(vec!["file".to_string(), "get".to_string(), args.path]).await);
            }

            result.end_print();
        },

        // 移除文件
        ClientCommands::Remove(args) => {
            let mut result = ClientResult::result().await;

            if args.get {
                // 获得文件的锁
                result.combine_unchecked(exec(vec!["file".to_string(), "get".to_string(), (&args.search).clone()]).await);
            }
            // 移除文件
            result.combine_unchecked(exec(vec!["file".to_string(), "remove".to_string(), args.search]).await);

            result.end_print();
        },

        // 移动文件
        ClientCommands::Move(args) => {
            let mut result = ClientResult::result().await;

            if args.get {
                // 获得文件的锁
                result.combine_unchecked(exec(vec!["file".to_string(), "get".to_string(), (&args.from_search).clone()]).await);
            }

            if args.local {
                // 移动本地文件
                result.combine_unchecked(client_move_local_file(args).await);

            } else {

                // 移动远程文件
                result.combine_unchecked(exec(vec!["file".to_string(), "move".to_string(), args.from_search, args.to_path]).await);
            }

            result.end_print();
        },

        // 回滚文件
        ClientCommands::Rollback(args) => {
            let mut result = ClientResult::result().await;

            if args.get {
                // 获得文件的锁
                result.combine_unchecked(exec(vec!["file".to_string(), "get".to_string(), (&args.search).clone()]).await);
            }
            // 回滚版本
            result.combine_unchecked(exec(vec!["file".to_string(), "rollback".to_string(), (&args.search).clone(), (&args.version).to_string()]).await);

            // 直接重新下载文件
            if args.back {
                result.combine_unchecked(exec(vec!["view".to_string(), args.search, args.version.to_string()]).await);
            }

            result.end_print();
        }

        // 获得锁
        ClientCommands::Get(args) => {
            print_client_result(exec(vec!["file".to_string(), if args.longer { "get_longer".to_string() } else { "get".to_string() }, args.search]).await);
        }

        // 丢掉锁
        ClientCommands::Throw(args) => {
            print_client_result(exec(vec!["file".to_string(), "throw".to_string(), args.search]).await);
        }

        // 查看锁
        ClientCommands::View(args) => {
            let mut result = ClientResult::result().await;

            if let Some(version) = args.version {
                result.combine_unchecked(exec(vec!["view".to_string(), (&args.search).clone(), version.to_string()]).await);
            } else {
                result.combine_unchecked(exec(vec!["view".to_string(), (&args.search).clone()]).await);
            }

            if args.get {
                // 获得文件的锁
                result.combine_unchecked(exec(vec!["file".to_string(), "get".to_string(), (&args.search).clone()]).await);
            }

            result.end_print();
        },

        // 参数
        ClientCommands::Param(args) => {
            if let Some(key) = args.key {
                match args.value {
                    None => client_query_param(key),
                    Some(content) => if content.trim() == "null" || content.trim() == "none" {
                        erase_parameter(key)
                    } else {
                        write_parameter(key, content)
                    }
                }
            } else {
                let mut result = ClientResult::query(ClientResultQueryProcess::line_by_line).await;
                for parameter in parameters() {
                    let parameter = parameter
                        .split("/")
                        .last().unwrap_or("")
                        .to_string();
                    if parameter.is_empty() { continue }
                    result.log(format!("{} = \"{}\"",
                             parameter.clone(),
                             read_parameter(parameter)
                                 .unwrap_or("".to_string())
                                 .replace("\n", "\\n")
                                 .replace("\t", "\\t")
                                 .replace("\r", "\\r")
                    ).as_str());
                }
                result.end_print();
            }
        }

        // 克隆
        ClientCommands::Clone => client_clone().await,

        // 格洛克？？？
        ClientCommands::Glock => print_glock_xd(),
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
    let target_path = current_dir.join(mv.to_path.clone());

    if ! target_path.exists() {

        // 修改本地文件地址
        let new_path = process_path_text(mv.to_path);
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

/// 将所有文件克隆到本地
async fn client_clone() {
    let mut result = ClientResult::result().await;
    let database = Database::read().await;
    for file in database.files() {
        println!("Checking {}", format!("\"{}\"", file.path()).cyan());
        result.combine_unchecked(exec(vec!["view".to_string(), file.path()]).await);
    }
    result.end_print();
}

/// 重定向
async fn client_redirect(args: RedirectArgs) {
    let mut workspace = Workspace::read().await;

    if let Some(client) = &mut workspace.client {

        // 重定向账户
        if let Some(login_code) = args.login_code {
            client.login_code = login_code;
            println!("Trying to change login code to {}", client.login_code);
        }

        // 此处：若同时指定工作区名称和地址，仅更新地址
        if let Some(target_addr) = args.target {

            // 若成功
            if let Ok(addr) = parse_address_v4_str(target_addr).await {
                client.target_addr = addr;

                println!("Changed target address to {}", &client.target_addr);

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

            println!("Redirected {} to {}.", client.workspace_name, addr);

            // 并保存工作区信息
            Workspace::update(&workspace).await;
            return;
        }
    }
    println!("Redirect failed.");
}

/// 客户端查询
async fn client_query(command: ClientQueryCommands) {
    match command {

        // 列出某个目录下的结构
        ClientQueryCommands::ListDirectory(args) => {
            let mut result = ClientResult::query(ClientResultQueryProcess::line_by_line).await;
            if args.completion_mode { result.set_debug(false); }
            let folder_map = LocalFolderMap::read().await;
            let database = Database::read().await;
            let current = args.value
                .trim()
                .trim_start_matches("./")
                .trim_start_matches("/");

            // 本地文件
            if let Ok(current_dir) = current_dir() {
                let current_folder = current_dir.join(current);
                if current_folder.exists() {
                    if let Ok(dir) = current_folder.read_dir() {
                        for dir in dir.into_iter() {
                            if dir.is_err() { continue; }
                            let dir = dir.unwrap().path();
                            if let Some(os_name) = dir.file_name() {
                                if let Some(name) = os_name.to_str() {
                                    let mut path = format!("{}{}", current, name);
                                    if dir.is_dir() {
                                        path = format!("{}/", path);
                                        if path == env!("PATH_WORKSPACE_ROOT") { continue }
                                        if ! folder_map.folder_files.contains_key(&path) {
                                            result.log(format!("{}/", name).as_str());
                                        }
                                    } else {
                                        if ! database.contains_path(&path) {
                                            result.log(format!("{}", name).as_str());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // 远程文件
            let list = folder_map.folder_files.get(current);
            if let Some(list) = list {
                for item in list {
                    match item {
                        Node::Jump(directory_str) => {
                            let v = process_path(directory_str.trim().trim_end_matches('/'))
                                .to_string().add("/");
                            result.log(v.as_str());
                        }
                        Node::File(virtual_file_path_str) => {
                            let v = process_path(virtual_file_path_str);
                            result.log(v.as_str());
                        }
                        _ => { continue; }
                    }
                }
            }
            
            // 短名称
            if args.completion_mode {
                for (k, _v) in folder_map.short_file_map {
                    result.log(format!(":{}", k).as_str());
                }
            }
            
            result.end_print();
        }

        // 查询虚拟文件的Uuid
        ClientQueryCommands::FileUuid(args) => {
            let mut result = ClientResult::query(ClientResultQueryProcess::direct).await;
            let database = Database::read().await;
            if let Some(file) = database.search_file(args.value.trim().to_string()) {
                if let Some(uuid) = database.uuid_of_path(file.path()) {
                    result.log(uuid.as_str());
                    result.end_print();
                }
            }
        }

        // 查询虚拟文件的版本
        ClientQueryCommands::FileVersion(args) => {
            let mut result = ClientResult::query(ClientResultQueryProcess::direct).await;
            let database = Database::read().await;
            if let Some(file) = database.search_file(args.value.trim().to_string()) {
                result.log(format!("{}", file.version()).as_str());
                result.end_print();
            }
        }

        // 查询虚拟文件的路径
        ClientQueryCommands::FilePath(args) => {
            let mut result = ClientResult::query(ClientResultQueryProcess::direct).await;
            let database = Database::read().await;
            if let Some(file) = database.search_file(args.value.trim().to_string()) {
                result.log(format!("{}", file.path()).as_str());
                result.end_print();
            }
        }

        // 查询虚拟文件的名称
        ClientQueryCommands::FileName(args) => {
            let mut result = ClientResult::query(ClientResultQueryProcess::direct).await;
            let database = Database::read().await;
            if let Some(file) = database.search_file(args.value.trim().to_string()) {
                result.log(format!("{}", process_path(file.path().as_str())).as_str());
                result.end_print();
            }
        }

        // 查询虚拟文件的锁定状态
        ClientQueryCommands::FileLockStatus(args) => {
            let mut result = ClientResult::query(ClientResultQueryProcess::direct).await;
            let database = Database::read().await;
            let workspace = Workspace::read().await;
            if let Some(file) = database.search_file(args.value.trim().to_string()) {
                if let Some(locker_owner) = file.get_locker_owner_uuid() {
                    if locker_owner == workspace.client.unwrap().uuid {
                        if file.is_longer_lock_unchecked() {
                            result.log("HELD");
                        } else {
                            result.log("held")
                        }
                    } else {
                        if file.is_longer_lock_unchecked() {
                            result.log("LOCK")
                        } else {
                            result.log("lock")
                        }
                    }
                } else {
                    result.log("Available")
                }
            }
            result.end_print();
        }

        // 查询自己的Uuid
        ClientQueryCommands::SelfUuid => {
            let mut result = ClientResult::query(ClientResultQueryProcess::direct).await;
            result.log(format!("{}", Workspace::read().await.client.unwrap().uuid).as_str());
            result.end_print();
        }

        // 查询目标工作区地址
        ClientQueryCommands::TargetAddress => {
            let mut result = ClientResult::query(ClientResultQueryProcess::direct).await;
            result.log(format!("{}", Workspace::read().await.client.unwrap().target_addr).as_str());
            result.end_print();
        }

        // 查询目标工作区名称
        ClientQueryCommands::Workspace => {
            let mut result = ClientResult::query(ClientResultQueryProcess::direct).await;
            result.log(format!("{}", Workspace::read().await.client.unwrap().workspace_name).as_str());
            result.end_print();
        }

        // 查询虚拟文件是否在本地
        ClientQueryCommands::ContainLocal(args) => {
            let mut result = ClientResult::query(ClientResultQueryProcess::direct).await;
            let database = Database::read().await;
            let local = LocalFileMap::read().await;
            if let Some(file) = database.search_file(args.value.trim().to_string()) {
                if let Some(uuid) = database.uuid_of_path(file.path()) {
                    if let Some(_) = local.file_paths.get(uuid.as_str()) {
                        result.log("true");
                    } else {
                        result.log("false");
                    }
                }
            }
            result.end_print();
        }

        // 查询本地文件映射的虚拟文件
        ClientQueryCommands::LocalToRemote(args) => {
            let mut result = ClientResult::query(ClientResultQueryProcess::direct).await;
            let database = Database::read().await;
            let local = LocalFileMap::read().await;
            if let Some(uuid) = local.local_path_to_uuid(args.value.trim().to_string()) {
                if let Some(file) = database.search_file(uuid.trim().to_string()) {
                    if file.path().is_empty() {
                        result.log(format!("{}", uuid).as_str());
                    } else {
                        result.log(format!("{}", file.path()).as_str());
                    }
                }
            }
            result.end_print();
        }

        // 查询虚拟文件映射的本地文件
        ClientQueryCommands::RemoteToLocal(args) => {
            let mut result = ClientResult::query(ClientResultQueryProcess::direct).await;
            let database = Database::read().await;
            let local = LocalFileMap::read().await;
            if let Some(file) = database.search_file(args.value.trim().to_string()) {
                if let Some(local_file) = local.search_to_local(&database, file.path()) {
                    result.log(format!("{}", local_file.local_path).as_str());
                }
            }
            result.end_print();
        }

        // 查询本地文件是否被更改
        ClientQueryCommands::Changed(args) => {
            let mut result = ClientResult::query(ClientResultQueryProcess::direct).await;
            let database = Database::read().await;
            let local = LocalFileMap::read().await;
            if let Some(file) = database.search_file(args.value.trim().to_string()) {
                if let Some(local_file) = local.search_to_local(&database, file.path()) {
                    let local_digest = &local_file.local_digest;
                    let current_digest = if let Some(path_buf) = local.search_to_path(&database, args.value.trim().to_string()) {
                        if path_buf.exists() {
                            Some(md5_digest(path_buf).unwrap_or(local_digest.clone()))
                        } else {
                            Some(local_digest.clone())
                        }
                    } else {
                        None
                    };
                    if let Some(digest) = current_digest {
                        if digest.trim() == local_digest {
                            result.log("false");
                        } else {
                            result.log("true");
                        }
                    }
                }
            }
            result.end_print();
        }

        // 查询本地文件的版本号
        ClientQueryCommands::LocalVersion(args) => {
            let mut result = ClientResult::query(ClientResultQueryProcess::direct).await;
            let database = Database::read().await;
            let local = LocalFileMap::read().await;
            if let Some(local_file) = local.search_to_local(&database, args.value.trim().to_string()) {
                result.log(format!("{}", local_file.local_version).as_str());
            }
            result.end_print();
        }
    }

    fn process_path(input: &str) -> String {
        let binding = input.to_string();
        binding.split("/").last().unwrap_or("").to_string()
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
async fn exec(args: Vec<String>) -> Option<ClientResult> {
    execute(args).await
}

/// 查询参数
fn client_query_param(param_name: String) {
    print!("{}", read_parameter(param_name.clone()).unwrap_or("".to_string()));
}

fn print_client_result(result : Option<ClientResult>) {
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