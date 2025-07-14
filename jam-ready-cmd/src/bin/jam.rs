use clap::{Args, Parser, Subcommand};
use jam_ready::utils::address_str_parser::parse_address_v4_str;
use jam_ready::utils::levenshtein_distance::levenshtein_distance;
use jam_ready::utils::local_archive::LocalArchive;
use jam_ready::utils::text_process::{process_id_text, process_id_text_not_to_lower};
use jam_ready_cmd::data::member::{Member, MemberDuty};
use jam_ready_cmd::data::parameters::{erase_parameter, read_parameter, write_parameter};
use jam_ready_cmd::data::workspace::WorkspaceType::{Client, Server, Unknown};
use jam_ready_cmd::data::workspace::{ClientWorkspace, ServerWorkspace, Workspace};
use jam_ready_cmd::service::jam_client::{execute, search_workspace_lan};
use jam_ready_cmd::service::jam_server::jam_server_entry;
use rand::Rng;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use strum::IntoEnumIterator;
use jam_ready_cmd::service::service_utils::get_self_address;
// --------------------------------------------------------------------------- //

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

    /// 检查当前工作区类型
    #[command(about = "Get workspace type")]
    Type,

    /// 建立客户端环境
    #[command(about = "Setup as client")]
    Client(ClientSetupArgs),

    /// 建立服务端环境
    #[command(about = "Setup as server")]
    Server(ServerSetupArgs),
}

/// 客户端建立参数
#[derive(Args, Debug)]
struct ClientSetupArgs {

    /// 用户登录口令，用于识别身份
    login_code: String,

    /// 目标地址 (直接指定)
    #[arg(short, long)]
    target: Option<String>,

    /// 工作区名称 (由网络发现获取目标地址)
    #[arg(short, long)]
    workspace: Option<String>,
}

/// 服务端建立参数
#[derive(Args, Debug)]
struct ServerSetupArgs {

    /// 工作区名称，服务端必填
    workspace: String,

    /// 端口设定，可选
    #[arg(short, long)]
    port: Option<u16>
}

/// 建立工作区
async fn setup_workspace_main(workspace: Workspace) {
    let cmd = WorkspaceSetup::parse();
    match cmd.command {

        // 检查工作区类型
        WorkspaceSetupCommands::Type => print!("null"),

        // 建立客户端工作区
        WorkspaceSetupCommands::Client(args) => setup_client_workspace(args, workspace).await,

        // 建立服务端工作区
        WorkspaceSetupCommands::Server(args) => setup_server_workspace(args, workspace).await,
    }
}

async fn setup_client_workspace(args: ClientSetupArgs, mut workspace: Workspace) {
    workspace.workspace_type = Client;

    // 如果 目标地址 不存在，且 工作区 也没有指定，则无法创建工作区
    if args.target.is_none() && args.workspace.is_none() {
        eprintln!("You need to specify a target or workspace");
        eprintln!("\"--workspace\" or \"--target\"");
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
    Workspace::update(&mut workspace);
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
    Workspace::update(&mut workspace);
}

// --------------------------------------------------------------------------- //

/// 客户端命令行
#[derive(Parser, Debug)]
#[command(
    disable_help_flag = true,
    disable_version_flag = true,
    disable_help_subcommand = true,
    help_template = "{all-args}"
)]
struct ClientWorkspaceEntry {
    #[command(subcommand)]
    command: ClientCommands,
}

/// 客户端命令
#[derive(Subcommand, Debug)]
enum ClientCommands {

    /// 检查当前工作区类型
    #[command(about = "Get workspace type")]
    Type,

    /// 重新连接至工作区
    #[command(about = "Redirect to workspace")]
    Redirect,

    /// 同步数据库内容
    #[command(about = "Synchronize database content")]
    Sync,

    /// 下载并查看文件
    #[command(about = "Download and view file")]
    View(SearchArgs),

    /// 提交取得锁的本地文件
    #[command(about = "Commit locked local files")]
    Commit(CommitArgs),

    /// 列出文件结构
    #[command(about = "List file structure")]
    Struct,

    /// 归档数据库版本 (仅 Leader)
    #[command(about = "Archive database version (Leader only)")]
    Archive,

    /// 文件操作
    #[command(subcommand, about = "File operations")]
    File(FileOperationCommands),
}


/// 文件操作命令
#[derive(Subcommand, Debug)]
enum FileOperationCommands {

    /// 添加文件
    #[command(about = "Add new visual file")]
    Add(PathArgs),

    /// 移除文件
    #[command(about = "Remove existing visual file")]
    Remove(SearchArgs),

    /// 移动、重命名、或为文件重建映射
    #[command(about = "Move/Rename/Remap visual file")]
    Move(MoveArgs),

    /// 拿到文件的锁
    #[command(about = "Get file lock")]
    Get(SearchArgs),

    /// 丢掉文件的锁
    #[command(about = "Throw file lock")]
    Throw(SearchArgs),
}

/// 目录参数
#[derive(Args, Debug)]
struct PathArgs {

    /// 目录
    path: String
}

/// 搜索 (Path or Uuid) 参数
#[derive(Args, Debug)]
struct SearchArgs {

    /// 搜索
    search: String
}

/// 搜索 (Path or Uuid) 参数
#[derive(Args, Debug)]
struct MoveArgs {

    /// 搜索
    from_search: String,

    /// 移动到
    to_path: String
}

#[derive(Args, Debug)]
struct CommitArgs {

    /// 日志
    log: Option<String>
}

/// 运行命令参数
#[derive(Args, Debug)]
struct ExecuteCommandArgs {

    /// 命令
    command: String
}


/// Logger 相关命令
#[derive(Subcommand, Debug)]
enum LoggerCommands {

    /// 启用 Logger
    #[command(about = "Enable logger")]
    Enable,

    /// 禁用 Logger
    #[command(about = "Disable logger")]
    Disable
}

async fn client_workspace_main() {
    let cmd = ClientWorkspaceEntry::parse();

    match cmd.command {

        // 检查工作区类型
        ClientCommands::Type => print!("client"),

        // 重新连接至工作区
        ClientCommands::Redirect => {
            let mut workspace = Workspace::read();
            if let Some(client) = &mut workspace.client {
                if let Ok(addr) = search_workspace_lan(client.workspace_name.clone()).await {
                    client.target_addr = addr;
                }
            }
            Workspace::update(&workspace);
        }

        ClientCommands::Sync => client_execute_command(vec!["sync".to_string()]).await,
        ClientCommands::View(args) => client_execute_command(vec!["view".to_string(), args.search]).await,
        ClientCommands::Commit(args) => {
            if let Some(log) = args.log {
                client_execute_command(vec!["commit".to_string(), log]).await
            } else {
                client_execute_command(vec!["commit".to_string()]).await
            }
        }
        ClientCommands::Struct => client_execute_command(vec!["struct".to_string()]).await,
        ClientCommands::Archive => client_execute_command(vec!["archive".to_string()]).await,
        ClientCommands::File(commands) => {
            match commands {
                FileOperationCommands::Add(args) => client_execute_command(vec!["file".to_string(), "add".to_string(), args.path]).await,
                FileOperationCommands::Remove(args) => client_execute_command(vec!["file".to_string(), "remove".to_string(), args.search]).await,
                FileOperationCommands::Move(args) => client_execute_command(vec!["file".to_string(), "move".to_string(), args.from_search, args.to_path]).await,
                FileOperationCommands::Get(args) => client_execute_command(vec!["file".to_string(), "get".to_string(), args.search]).await,
                FileOperationCommands::Throw(args) => client_execute_command(vec!["file".to_string(), "throw".to_string(), args.search]).await,
            }
        }
    }
}

/// 客户端运行命令
async fn client_execute_command(args: Vec<String>) {
    // 运行命令
    execute(args).await;
}

// --------------------------------------------------------------------------- //

/// 服务端命令行
#[derive(Parser, Debug)]
#[command(
    disable_help_flag = true,
    disable_version_flag = true,
    disable_help_subcommand = true,
    help_template = "{all-args}"
)]
struct ServerWorkspaceEntry {
    #[command(subcommand)]
    command: ServerOperationCommands,
}

/// 服务端操作类命令
#[derive(Subcommand, Debug)]
enum ServerOperationCommands {

    /// 检查当前工作区类型
    #[command(about = "Get workspace type")]
    Type,

    /// 启动服务器，并监听客户端消息
    #[command(about = "Run server")]
    Run(RunArgs),

    /// 添加
    #[command(subcommand, about = "Add something")]
    Add(ServerOperationTargetCommands),

    /// 删除
    #[command(subcommand, about = "Remove something")]
    Remove(ServerOperationTargetCommands),

    /// 列表
    #[command(subcommand, about = "List something")]
    List(ServerListCommands),

    /// 查询
    #[command(subcommand, about = "Query something")]
    Query(ServerQueryCommands),

    /// 设置
    #[command(subcommand, about = "Set something")]
    Set(ServerSetCommands),
}

/// 服务器运行参数
#[derive(Args, Debug)]
struct RunArgs {

    /// 简短的 Logger
    #[arg(short = 'S', long = "short-logger")]
    short_logger: bool
}

/// 服务端操作指向
#[derive(Subcommand, Debug)]
enum ServerOperationTargetCommands {

    /// 操作成员
    #[command(about = "Operate members")]
    Member(MemberArgs),

    /// 操作职责
    #[command(about = "Operate duties")]
    Duty(DutyOperationArgs),

    /// 操作参数
    #[command(about = "Operate params")]
    Param(ParamArgs),

    /// 调试等级的 Logger
    #[command(about = "Operate debug")]
    Debug
}

/// 服务器列表命令
#[derive(Subcommand, Debug)]
enum ServerListCommands {

    /// 列出成员
    #[command(about = "List members")]
    Member
}

/// 服务器查询命令
#[derive(Subcommand, Debug)]
enum ServerQueryCommands {

    /// 查询成员的职责
    #[command(about = "Query duties of the member")]
    Duty(MemberArgs),

    /// 查询成员的 Uuid
    #[command(about = "Query uuid of the member")]
    Uuid(MemberArgs),

    /// 查询成员的 登录代码
    #[command(about = "Query login code of the member")]
    LoginCode(MemberArgs),

    /// 查询参数
    #[command(about = "Query param")]
    Param(ParamQueryArgs),

    /// 查询工作区名称
    #[command(about = "Query workspace name")]
    Workspace,

    /// 查询本地地址
    #[command(about = "Query lan address")]
    LocalAddress
}

/// 服务器设置命令
#[derive(Subcommand, Debug)]
enum ServerSetCommands {

    /// 设置成员
    #[command(subcommand, about = "Set member")]
    Member(ServerSetMemberCommands),
}

/// 服务器设置命令
#[derive(Subcommand, Debug)]
enum ServerSetMemberCommands {

    /// 设置成员的职责
    #[command(about = "Set duties of the member")]
    Duties(DutiesSetArgs),

    /// 设置成员名称
    #[command(about = "Set member")]
    Name(MemberRenameArgs),
}

/// 成员操作参数
#[derive(Args, Debug)]
struct MemberArgs {

    /// 成员名称
    member: String
}

/// 成员操作参数
#[derive(Args, Debug)]
struct MemberRenameArgs {

    /// 成员名称
    old_name: String,

    /// 新名称
    new_name: String
}

/// 职责操作参数
#[derive(Args, Debug)]
struct DutyOperationArgs {

    /// 职责
    duties: String,

    /// 成员名称
    member: String
}

/// 职责操作参数
#[derive(Args, Debug)]
struct DutiesSetArgs {

    /// 成员名称
    member: String,

    /// 职责
    duties: String
}

/// 参数操作
#[derive(Args, Debug)]
struct ParamArgs {

    /// 键
    key: String,

    /// 值
    value: Option<String>
}

#[derive(Args, Debug)]
struct ParamQueryArgs {

    /// 键
    key: String,
}

async fn server_workspace_main() {
    let cmd = ServerWorkspaceEntry::parse();

    match cmd.command {

        // 检查工作区类型
        ServerOperationCommands::Type => print!("server"),

        ServerOperationCommands::Run(args) => server_run(args).await,

        ServerOperationCommands::Add(op) => {
            match op {
                ServerOperationTargetCommands::Member(args) => server_add_member(args.member),
                ServerOperationTargetCommands::Duty(args) => server_add_duty_to_member(args.duties, args.member),
                ServerOperationTargetCommands::Param(args) => server_add_param(args.key, args.value),
                ServerOperationTargetCommands::Debug => {
                    let mut workspace = Workspace::read();
                    if let Some(server) = &mut workspace.server {
                        server.enable_debug_logger = true
                    }
                    Workspace::update(&workspace);
                }
            }
        }
        ServerOperationCommands::Remove(op) => {
            match op {
                ServerOperationTargetCommands::Member(args) => server_remove_member(args.member),
                ServerOperationTargetCommands::Duty(args) => server_remove_duty_from_member(args.duties, args.member),
                ServerOperationTargetCommands::Param(args) => server_remove_param(args.key, args.value),
                ServerOperationTargetCommands::Debug => {
                    let mut workspace = Workspace::read();
                    if let Some(server) = &mut workspace.server {
                        server.enable_debug_logger = false
                    }
                    Workspace::update(&workspace);
                }
            }
        }
        ServerOperationCommands::List(op) => {
            match op {
                ServerListCommands::Member => server_list_members()
            }
        }
        ServerOperationCommands::Query(op) => {
            match op {
                ServerQueryCommands::Duty(args) => server_query_duties_of_member(args.member),
                ServerQueryCommands::Uuid(args) => server_query_uuid_of_member(args.member),
                ServerQueryCommands::Param(args) => server_query_param(args.key),
                ServerQueryCommands::LoginCode(args) => server_query_login_code(args.member),
                ServerQueryCommands::Workspace => server_query_workspace(),
                ServerQueryCommands::LocalAddress => print!("{}", get_self_address())
            }
        }
        ServerOperationCommands::Set(op) => {
            match op {
                ServerSetCommands::Member(op) => {
                    match op {
                        ServerSetMemberCommands::Duties(args) => server_set_duties_to_member(args.member, args.duties),
                        ServerSetMemberCommands::Name(args) => server_set_member_name(args)
                    }
                }
            }
        }
    }
}

async fn server_run(args: RunArgs) {
    jam_server_entry(args.short_logger).await
}

/// 添加成员
fn server_add_member (member_name: String) {
    let member_name = process_id_text(member_name);
    let mut workspace = Workspace::read();
    if let Some(server) = &mut workspace.server {
        for (_uuid, member) in server.members.iter() {
            if member.member_name.trim() == member_name.trim() {
                eprintln!("Failed: Contains duplicate member name");
                return;
            }
        }
        let uuid = uuid::Uuid::new_v4().to_string();
        server.members.insert(
            uuid.clone(),
            Member {
                member_name: member_name.clone(),
                member_duties: vec![],
            }
        );
        server.member_uuids.insert(
            member_name.clone(),
            uuid.clone()
        );
        server.login_code_map.insert(
            generate_login_code(),
            uuid
        );
        println!("Member \"{}\" has been added to the workspace", member_name);
        Workspace::update(&mut workspace);
    }
}

/// 生成登录代码
fn generate_login_code() -> String {
    let charset: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".chars().collect();
    let mut rng = rand::rng();

    let mut code = String::with_capacity(9);
    for _ in 0..4 {
        let idx = rng.random_range(0..charset.len());
        code.push(charset[idx]);
    }
    code.push('-');
    for _ in 0..4 {
        let idx = rng.random_range(0..charset.len());
        code.push(charset[idx]);
    }
    code
}

/// 移除成员
fn server_remove_member(member_name: String) {
    let member_name = process_id_text(member_name);
    let mut workspace = Workspace::read();
    if let Some(server) = &mut workspace.server {
        let mut found = false;
        let mut uuid_to_remove = None;
        let mut login_code_to_remove = None;
        for (uuid, member) in &server.members {
            if member.member_name.trim() == member_name {
                uuid_to_remove = Some(uuid.clone());
                login_code_to_remove = server.login_code_map.get(uuid);
                found = true;
                break;
            }
        }
        if let Some(login_code) = login_code_to_remove {
            let _ = server.login_code_map.remove(&login_code.clone());
        }
        if let Some(uuid) = uuid_to_remove {
            let _ = server.member_uuids.remove(&member_name);
            if server.members.remove(&uuid).is_some() {
                println!("Member \"{}\" has been removed from the workspace", member_name);
                Workspace::update(&mut workspace);
            } else {
                eprintln!("Failed to remove member \"{}\"", member_name);
            }
        } else if !found {
            eprintln!("Failed: Member \"{}\" does not exist in this workspace.", member_name);
        }
    }
}

/// 添加成员职责
fn server_add_duty_to_member (duty_name: String, member_name: String) {
    let member_name = process_id_text(member_name);
    let mut workspace = Workspace::read();
    if let Some(server) = &mut workspace.server {
        for (_, member) in &mut server.members {
            if member.member_name.trim() == member_name.trim() {
                let duty = search_duty_by_str(duty_name.clone());
                match duty {
                    Ok(duty) => {
                        if ! member.member_duties.contains(&duty) {
                            member.add_duty(duty.clone());
                            println!("Added duty \"{:?}\" for member \"{}\"", duty.clone(), member_name);
                            Workspace::update(&mut workspace);
                            return;
                        }
                    }
                    Err(maybe) => {
                        print_maybe(maybe, duty_name.clone());
                        return;
                    }
                }
            }
        }
    }
}

/// 设置成员的职责
fn server_set_duties_to_member (member_name: String, duties_str: String) {
    let mut workspace = Workspace::read();
    if let Some(server) = &mut workspace.server {

        // 清除成员职责
        if let Some(member_uuid) = server.member_uuids.get(member_name.as_str()) {
            if let Some(member) = server.members.get_mut(member_uuid) {
                member.member_duties.clear();
            }
        }
        Workspace::update(&mut workspace);
    }

    // 遍历添加
    for duty_str in duties_str.split(",") {
        let duty_str = duty_str.trim();
        server_add_duty_to_member(duty_str.to_string(), member_name.clone());
    }
}

/// 设置成员的名称
fn server_set_member_name(args: MemberRenameArgs) {
    let old_name = process_id_text(args.old_name);
    let new_name = process_id_text(args.new_name);
    if new_name.is_empty() {
        return;
    }

    let mut workspace = Workspace::read();
    let mut found_uuid = None;
    if let Some(server) = &mut workspace.server {

        // 新名称不存在
        if let None = server.member_uuids.get(new_name.trim()) {

            // 拿出旧的 Uuid，并尝试拿到原来的成员
            if let Some(uuid) = server.member_uuids.remove(old_name.trim()) {
                if let Some(member) = server.members.get_mut(uuid.as_str()) {

                    // 设置新的名称
                    member.member_name = new_name.clone();

                    // 记录旧的 Uuid
                    found_uuid = Some(uuid);
                }
            }
        }
    }

    // 若找到旧的 Uuid，说明设置名称成功，此时开始重建映射，并保存工作区
    if let Some(server) = &mut workspace.server {
        if let Some(uuid) = found_uuid {
            server.member_uuids.insert(new_name, uuid);
        }
        Workspace::update(&mut workspace);
    }
}

/// 移除成员职责
fn server_remove_duty_from_member (duty_name: String, member_name: String) {
    let member_name = process_id_text(member_name);
    let mut workspace = Workspace::read();
    if let Some(server) = &mut workspace.server {
        for (_, member) in &mut server.members {
            if member.member_name.trim() == member_name.trim() {
                let duty = search_duty_by_str(duty_name.clone());
                match duty {
                    Ok(duty) => {
                        if member.member_duties.contains(&duty) {
                            member.remove_duty(duty.clone());
                            println!("Removed duty \"{:?}\" from member \"{}\"", duty.clone(), member_name);
                            Workspace::update(&mut workspace);
                            return;
                        }
                    }
                    Err(maybe) => {
                        print_maybe(maybe, duty_name.clone());
                        return;
                    }
                }
            }
        }
    }
}

/// 添加参数
fn server_add_param(param_name: String, value: Option<String>) {
    let value = value.unwrap_or("".to_string());
    write_parameter(param_name, value);
}

/// 擦除参数
fn server_remove_param(param_name: String, _value: Option<String>) {
    erase_parameter(param_name);
}

fn print_maybe(maybe: Option<MemberDuty>, duty_name: String) {
    match maybe {
        None => {
            eprintln!("Unable to find a duty named \"{}\"", duty_name.trim());
        }
        Some(mean) => {
            eprintln!("Unable to find a duty named \"{}\". Did you mean \"{:?}\"?", duty_name.trim(), mean);
        }
    }
}

/// 搜索 Duty (结果，可能是)
fn search_duty_by_str (input: String) -> Result<MemberDuty, Option<MemberDuty>> {
    let mut vec = Vec::new();
    let (mut index, mut current, mut min) = (0, 0, 1000);
    for duty in MemberDuty::iter() {
        let name = format!("{:?}", duty);
        let dist = levenshtein_distance(
            name.to_lowercase().as_str(),
            input.to_lowercase().as_str());
        if dist < min {
            min = dist;
            current = index;
        }
        vec.push(duty);
        index += 1;
    }
    if min >= 3 {
        Err(None)
    } else if min > 0 {
        Err(Some(vec.get(current).unwrap().clone()))
    } else {
        Ok(vec.get(current).unwrap().clone())
    }
}

/// 列出成员
fn server_list_members () {
    let workspace = Workspace::read();
    if let Some(server) = workspace.server {
        let mut result : String = "".to_string();
        for (_uuid, member) in server.members {
            result += format!("{}, ", member.member_name).as_str();
        }
        print!("{}", result.trim().trim_end_matches(","))
    }
}

/// 列出成员的职责
fn server_query_duties_of_member (member_name: String) {
    let member_name = process_id_text(member_name);
    let workspace = Workspace::read();
    if let Some(server) = workspace.server {
        for (_uuid, member) in server.members {
            if member.member_name.trim() == member_name {
                let mut result : String = "".to_string();
                for member_duty in member.member_duties {
                    result += format!("{:?}, ", member_duty).as_str();
                }
                print!("{}", result.trim().trim_end_matches(","));
                break
            }
        }
    }
}

/// 查询成员的 Uuid
fn server_query_uuid_of_member (member_name: String) {
    let member_name = process_id_text(member_name);
    let workspace = Workspace::read();
    if let Some(server) = workspace.server {
        for (uuid, member) in server.members {
            if member.member_name.trim() == member_name {
                print!("{}", uuid);
                break
            }
        }
    }
}

/// 查询参数
fn server_query_param(param_name: String) {
    print!("{}", read_parameter(param_name.clone()).unwrap_or("".to_string()));
}

/// 查询成员的 登录代码
fn server_query_login_code(member_name: String) {
    let member_name = process_id_text(member_name);
    let workspace = Workspace::read();
    if let Some(server) = workspace.server {
        for (code, uuid) in server.login_code_map {
            if let Some(member_uuid) = server.member_uuids.get(&member_name) {
                if member_uuid.trim() == uuid.trim() {
                    print!("{}", code);
                }
            }
        }
    }
}

/// 查询工作区名称
fn server_query_workspace() {
    let workspace = Workspace::read();
    if let Some(server) = workspace.server {
        print!("{}", server.workspace_name);
    }
}

// --------------------------------------------------------------------------- //

#[tokio::main]
async fn main() {

    // 加载工作区
    let workspace = Workspace::read();

    // 若未初始化工作区，则引导用户初始化
    if workspace.workspace_type == Unknown {
        setup_workspace_main(workspace).await;
    } else if workspace.workspace_type == Client {
        client_workspace_main().await;
    } else if workspace.workspace_type == Server {
        server_workspace_main().await;
    }
}