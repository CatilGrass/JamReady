use std::env::args;
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
use crate::help::help_docs::get_help_docs;

/// Server command line interface
#[derive(Parser, Debug)]
#[command(
    disable_help_flag = true,
    disable_version_flag = true,
    disable_help_subcommand = true,
    help_template = "{all-args}"
)]
pub struct ServerWorkspaceEntry {
    #[command(subcommand)]
    command: ServerOperationCommands,
}

/// Server operation commands
#[derive(Subcommand, Debug)]
enum ServerOperationCommands {

    #[command(
        hide = true,
        short_flag = 'h',
        long_flag = "help")]
    Help,

    /// Start server and listen for client messages
    Run,

    /// Add
    #[command(subcommand)]
    Add(ServerOperationTargetCommands),

    /// Remove
    #[command(subcommand)]
    Remove(ServerOperationTargetCommands),

    /// List
    #[command(subcommand)]
    List(ServerListCommands),

    /// Query
    #[command(subcommand)]
    Query(ServerQueryCommands),

    /// Set
    #[command(subcommand)]
    Set(ServerSetCommands),
}

/// Server operation targets
#[derive(Subcommand, Debug)]
enum ServerOperationTargetCommands {

    /// Operate on members
    Member(MemberArgs),

    /// Operate on duties
    Duty(DutyOperationArgs),

    /// Debug level logger
    Debug
}

/// Server list commands
#[derive(Subcommand, Debug)]
enum ServerListCommands {

    /// List members
    Member
}

/// Server query commands
#[derive(Subcommand, Debug)]
enum ServerQueryCommands {

    /// Query member duties
    Duty(MemberArgs),

    /// Query member Uuid
    Uuid(MemberArgs),

    /// Query member login code
    LoginCode(MemberArgs),

    /// Query workspace name
    Workspace,

    /// Query local address
    LocalAddress
}

/// Server set commands
#[derive(Subcommand, Debug)]
enum ServerSetCommands {

    /// Set member properties
    #[command(subcommand)]
    Member(ServerSetMemberCommands),
}

/// Server set member commands
#[derive(Subcommand, Debug)]
enum ServerSetMemberCommands {

    /// Set member duties
    Duties(DutiesSetArgs),

    /// Set member name
    Name(MemberRenameArgs),
}

/// Member operation arguments
#[derive(Args, Debug)]
struct MemberArgs {

    /// Member name
    member: String
}

/// Member rename arguments
#[derive(Args, Debug)]
struct MemberRenameArgs {

    /// Current name
    old_name: String,

    /// New name
    new_name: String
}

/// Duty operation arguments
#[derive(Args, Debug)]
struct DutyOperationArgs {

    /// Duty name
    duties: String,

    /// Member name
    member: String
}

/// Duties set arguments
#[derive(Args, Debug)]
struct DutiesSetArgs {

    /// Member name
    member: String,

    /// Duties
    duties: String
}

pub async fn server_workspace_main() {

    if args().count() <= 1 {
        server_print_help();
        return;
    }

    let cmd = ServerWorkspaceEntry::parse();

    match cmd.command {

        ServerOperationCommands::Help => server_print_help(),

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

fn server_print_help() {
    println!("{}", get_help_docs("server_help"));
}

async fn server_run() {

    // Build database
    let database = Arc::new(Mutex::new(Database::read().await));

    // Signals
    let (write_tx, write_rx) : (UnboundedSender<bool>, UnboundedReceiver<bool>) = unbounded_channel();

    join!(jam_server_entry(database.clone(), write_tx.clone()), refresh_monitor(database.clone(), write_rx));
}

/// Add member
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

/// Generate login code
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

/// Remove member
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
        // Remove login code binding
        if let Some(login_code) = login_code_to_remove {
            let _ = server.login_code_map.remove(&login_code.clone());
        }
        // Remove member data
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

/// Add duty to member
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

/// Set member duties
async fn server_set_duties_to_member (member_name: String, duties_str: String) {
    let mut workspace = Workspace::read().await;
    if let Some(server) = &mut workspace.server {

        // Clear member duties
        if let Some(member_uuid) = server.member_uuids.get(member_name.as_str()) {
            if let Some(member) = server.members.get_mut(member_uuid) {
                member.member_duties.clear();
            }
        }
        Workspace::update(&mut workspace).await;
    }

    // Add duties one by one
    for duty_str in duties_str.split(",") {
        let duty_str = duty_str.trim();
        server_add_duty_to_member(duty_str.to_string(), member_name.clone()).await;
    }
}

/// Set member name
async fn server_set_member_name(args: MemberRenameArgs) {
    let old_name = process_id_text(args.old_name);
    let new_name = process_id_text(args.new_name);
    if new_name.is_empty() {
        return;
    }

    let mut workspace = Workspace::read().await;
    let mut found_uuid = None;
    if let Some(server) = &mut workspace.server {

        // New name doesn't exist
        if let None = server.member_uuids.get(new_name.trim()) {

            // Remove old uuid and get original member
            if let Some(uuid) = server.member_uuids.remove(old_name.trim()) {
                if let Some(member) = server.members.get_mut(uuid.as_str()) {

                    // Set new name
                    member.member_name = new_name.clone();

                    // Record old uuid
                    found_uuid = Some(uuid);
                }
            }
        }
    }

    // If old uuid was found, rebuild mapping and save workspace
    if let Some(server) = &mut workspace.server {
        if let Some(uuid) = found_uuid {
            server.member_uuids.insert(new_name, uuid);
        }
        Workspace::update(&mut workspace).await;
    }
}

/// Remove duty from member
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

/// Search duty (with possible suggestions)
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

/// List members
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

/// Query member duties
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

/// Query member Uuid
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

/// Query member login code
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

/// Query workspace name
async fn server_query_workspace() {
    let workspace = Workspace::read().await;
    if let Some(server) = workspace.server {
        println!("{}", server.workspace_name);
    }
}