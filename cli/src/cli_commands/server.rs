use crate::data::database::Database;
use crate::data::member::{Member, MemberDuty};
use crate::data::workspace::Workspace;
use crate::service::jam_server::{jam_server_entry, refresh_monitor};
use crate::service::service_utils::get_self_address;
use clap::{Args, Parser, Subcommand};
use jam_ready::utils::levenshtein_distance::levenshtein_distance;
use jam_ready::utils::local_archive::LocalArchive;
use jam_ready::utils::text_process::process_id_text;
use rand::Rng;
use std::sync::Arc;
use strum::IntoEnumIterator;
use tokio::join;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::Mutex;

/// 服务端命令行
#[derive(Parser, Debug)]
#[command(
    disable_help_flag = true,
    disable_version_flag = true,
    help_template = "{all-args}"
)]
pub struct ServerWorkspaceEntry {
    #[command(subcommand)]
    command: ServerOperationCommands,
}

/// 服务端操作类命令
#[derive(Subcommand, Debug)]
enum ServerOperationCommands {

    /// 启动服务器，并监听客户端消息
    #[command(about = "Run server")]
    Run,

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

/// 服务端操作指向
#[derive(Subcommand, Debug)]
enum ServerOperationTargetCommands {

    /// 操作成员
    #[command(about = "Operate members")]
    Member(MemberArgs),

    /// 操作职责
    #[command(about = "Operate duties")]
    Duty(DutyOperationArgs),

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

pub async fn server_workspace_main() {
    let cmd = ServerWorkspaceEntry::parse();

    match cmd.command {

        ServerOperationCommands::Run => server_run().await,

        ServerOperationCommands::Add(op) => {
            match op {
                ServerOperationTargetCommands::Member(args) => server_add_member(args.member).await,
                ServerOperationTargetCommands::Duty(args) => server_add_duty_to_member(args.duties, args.member).await,
                ServerOperationTargetCommands::Debug => {
                    let mut workspace = Workspace::read().await;
                    if let Some(server) = &mut workspace.server {
                        server.enable_debug_logger = true
                    }
                    Workspace::update(&workspace).await;
                }
            }
        }
        ServerOperationCommands::Remove(op) => {
            match op {
                ServerOperationTargetCommands::Member(args) => server_remove_member(args.member).await,
                ServerOperationTargetCommands::Duty(args) => server_remove_duty_from_member(args.duties, args.member).await,
                ServerOperationTargetCommands::Debug => {
                    let mut workspace = Workspace::read().await;
                    if let Some(server) = &mut workspace.server {
                        server.enable_debug_logger = false
                    }
                    Workspace::update(&workspace).await;
                }
            }
        }
        ServerOperationCommands::List(op) => {
            match op {
                ServerListCommands::Member => server_list_members().await
            }
        }
        ServerOperationCommands::Query(op) => {
            match op {
                ServerQueryCommands::Duty(args) => server_query_duties_of_member(args.member).await,
                ServerQueryCommands::Uuid(args) => server_query_uuid_of_member(args.member).await,
                ServerQueryCommands::LoginCode(args) => server_query_login_code(args.member).await,
                ServerQueryCommands::Workspace => server_query_workspace().await,
                ServerQueryCommands::LocalAddress => println!("{}", get_self_address())
            }
        }
        ServerOperationCommands::Set(op) => {
            match op {
                ServerSetCommands::Member(op) => {
                    match op {
                        ServerSetMemberCommands::Duties(args) => server_set_duties_to_member(args.member, args.duties).await,
                        ServerSetMemberCommands::Name(args) => server_set_member_name(args).await
                    }
                }
            }
        }
    }
}

async fn server_run() {

    // 构建数据库
    let database = Arc::new(Mutex::new(Database::read().await));

    // 信号
    let (write_tx, write_rx) : (UnboundedSender<bool>, UnboundedReceiver<bool>) = unbounded_channel();

    join!(jam_server_entry(database.clone(), write_tx.clone()), refresh_monitor(database.clone(), write_rx));
}

/// 添加成员
async fn server_add_member (member_name: String) {
    let member_name = process_id_text(member_name);
    let mut workspace = Workspace::read().await;
    if let Some(server) = &mut workspace.server {
        for (_uuid, member) in server.members.iter() {
            if member.member_name.trim() == member_name.trim() {
                eprintln!("Failed: Contains duplicate member name");
                return;
            }
        }
        let uuid = uuid::Uuid::new_v4().to_string();
        let login_code = generate_login_code();
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
            login_code.clone(),
            uuid
        );
        println!("Member \"{}\" has been added to the workspace, login code: {}", member_name, login_code);
        Workspace::update(&mut workspace).await;
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
async fn server_remove_member(member_name: String) {
    let member_name = process_id_text(member_name);
    let mut workspace = Workspace::read().await;
    if let Some(server) = &mut workspace.server {
        let mut found = false;
        let mut uuid_to_remove = None;
        let mut login_code_to_remove = None;
        for (uuid, member) in &server.members {
            if member.member_name.trim() == member_name {
                uuid_to_remove = Some(uuid.clone());
                for (login_code, mapped_uuid) in &server.login_code_map {
                    if mapped_uuid == uuid {
                        login_code_to_remove = Some(login_code.clone());
                    }
                }
                found = true;
                break;
            }
        }
        // 移除 Login Code 的绑定
        if let Some(login_code) = login_code_to_remove {
            let _ = server.login_code_map.remove(&login_code.clone());
        }
        // 移除用户数据
        if let Some(uuid) = uuid_to_remove {
            let _ = server.member_uuids.remove(&member_name);
            if server.members.remove(&uuid).is_some() {
                println!("Member \"{}\" has been removed from the workspace", member_name);
                Workspace::update(&mut workspace).await;
            } else {
                eprintln!("Failed to remove member \"{}\"", member_name);
            }
        } else if !found {
            eprintln!("Failed: Member \"{}\" does not exist in this workspace.", member_name);
        }
    }
}

/// 添加成员职责
async fn server_add_duty_to_member (duty_name: String, member_name: String) {
    let member_name = process_id_text(member_name);
    let mut workspace = Workspace::read().await;
    if let Some(server) = &mut workspace.server {
        for (_, member) in &mut server.members {
            if member.member_name.trim() == member_name.trim() {
                let duty = search_duty_by_str(duty_name.clone());
                match duty {
                    Ok(duty) => {
                        if ! member.member_duties.contains(&duty) {
                            member.add_duty(duty.clone());
                            println!("Added duty \"{:?}\" for member \"{}\"", duty.clone(), member_name);
                            Workspace::update(&mut workspace).await;
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
async fn server_set_duties_to_member (member_name: String, duties_str: String) {
    let mut workspace = Workspace::read().await;
    if let Some(server) = &mut workspace.server {

        // 清除成员职责
        if let Some(member_uuid) = server.member_uuids.get(member_name.as_str()) {
            if let Some(member) = server.members.get_mut(member_uuid) {
                member.member_duties.clear();
            }
        }
        Workspace::update(&mut workspace).await;
    }

    // 遍历添加
    for duty_str in duties_str.split(",") {
        let duty_str = duty_str.trim();
        server_add_duty_to_member(duty_str.to_string(), member_name.clone()).await;
    }
}

/// 设置成员的名称
async fn server_set_member_name(args: MemberRenameArgs) {
    let old_name = process_id_text(args.old_name);
    let new_name = process_id_text(args.new_name);
    if new_name.is_empty() {
        return;
    }

    let mut workspace = Workspace::read().await;
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
        Workspace::update(&mut workspace).await;
    }
}

/// 移除成员职责
async fn server_remove_duty_from_member (duty_name: String, member_name: String) {
    let member_name = process_id_text(member_name);
    let mut workspace = Workspace::read().await;
    if let Some(server) = &mut workspace.server {
        for (_, member) in &mut server.members {
            if member.member_name.trim() == member_name.trim() {
                let duty = search_duty_by_str(duty_name.clone());
                match duty {
                    Ok(duty) => {
                        if member.member_duties.contains(&duty) {
                            member.remove_duty(duty.clone());
                            println!("Removed duty \"{:?}\" from member \"{}\"", duty.clone(), member_name);
                            Workspace::update(&mut workspace).await;
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
async fn server_list_members () {
    let workspace = Workspace::read().await;
    if let Some(server) = workspace.server {
        let mut result : String = "".to_string();
        for (_uuid, member) in server.members {
            result += format!("{}, ", member.member_name).as_str();
        }
        println!("{}", result.trim().trim_end_matches(","))
    }
}

/// 列出成员的职责
async fn server_query_duties_of_member (member_name: String) {
    let member_name = process_id_text(member_name);
    let workspace = Workspace::read().await;
    if let Some(server) = workspace.server {
        for (_uuid, member) in server.members {
            if member.member_name.trim() == member_name {
                let mut result : String = "".to_string();
                for member_duty in member.member_duties {
                    result += format!("{:?}, ", member_duty).as_str();
                }
                println!("{}", result.trim().trim_end_matches(","));
                break
            }
        }
    }
}

/// 查询成员的 Uuid
async fn server_query_uuid_of_member (member_name: String) {
    let member_name = process_id_text(member_name);
    let workspace = Workspace::read().await;
    if let Some(server) = workspace.server {
        for (uuid, member) in server.members {
            if member.member_name.trim() == member_name {
                println!("{}", uuid);
                break
            }
        }
    }
}

/// 查询成员的 登录代码
async fn server_query_login_code(member_name: String) {
    let member_name = process_id_text(member_name);
    let workspace = Workspace::read().await;
    if let Some(server) = workspace.server {
        for (code, uuid) in server.login_code_map {
            if let Some(member_uuid) = server.member_uuids.get(&member_name) {
                if member_uuid.trim() == uuid.trim() {
                    println!("{}", code);
                }
            }
        }
    }
}

/// 查询工作区名称
async fn server_query_workspace() {
    let workspace = Workspace::read().await;
    if let Some(server) = workspace.server {
        println!("{}", server.workspace_name);
    }
}