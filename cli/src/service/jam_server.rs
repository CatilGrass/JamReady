use std::env::current_dir;
use std::io::Write;
use std::path::PathBuf;
use std::str::from_utf8;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use clearscreen::clear;
use colored::Colorize;
use log::{error, info};
use log::LevelFilter::{Info};
use sysinfo::{get_current_pid, System};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::{select, spawn};
use tokio::signal::ctrl_c;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::Mutex;
use tokio::time::sleep;
use walkdir::WalkDir;
use jam_ready::entry_mutex_async;
use jam_ready::utils::local_archive::LocalArchive;
use jam_ready::utils::logger_build::logger_build;
use jam_ready::utils::text_process::show_tree;
use crate::data::database::Database;
use crate::data::member::Member;
use crate::data::workspace::Workspace;
use crate::service::commands::registry;
use crate::service::jam_command::{execute_remote_command, CommandRegistry};
use crate::service::messages::ClientMessage;
use crate::service::messages::ClientMessage::Verify;
use crate::service::messages::ServerMessage::{Deny, Uuid};
use crate::service::service_utils::{get_self_address_with_port_str, read_msg, send_msg};

const DISCOVERY_PORT: u16 = 54000;
const MAX_BUFFER_SIZE: usize = 1024;

/// Server entry point
pub async fn jam_server_entry(
    database: Arc<Mutex<Database>>,
    sender: UnboundedSender<bool>
) {
    // Build logger and try to get workspace name
    let workspace = Workspace::read().await;
    let mut workspace_name = "Workspace";

    if let Some(server) = &workspace.server {
        workspace_name = server.workspace_name.as_str();

        // Setup logger
        if server.enable_debug_logger {
            logger_build(Info);
        }
    }

    // Build command registry
    let commands = Arc::new(registry());

    // Get local IP address
    let address_tcp = get_self_address_with_port_str(env!("DEFAULT_SERVER_PORT"));

    // Bind TCP listener
    let listener_bind = TcpListener::bind(&address_tcp).await;
    if listener_bind.is_err() {
        error!("Failed to bind to {}", address_tcp.to_string());
        return;
    }

    let listener : TcpListener;
    if let Ok(result) = listener_bind {
        listener = result;
    } else {
        error!("Failed to bind to {}", address_tcp.to_string());
        return;
    }

    // Initialize network discovery
    let socket = UdpSocket::bind(format!("0.0.0.0:{}", DISCOVERY_PORT)).await.unwrap();
    let mut buf = [0u8; MAX_BUFFER_SIZE];

    // Print server info
    info!("Workspace: \"{}\", Address: {}, DiscoveryServicePort: {}, Commands: {}",
        workspace_name,
        &address_tcp,
        DISCOVERY_PORT,
        commands.len()
    );

    // Main event loop
    loop {
        select! {
            // Ctrl+C shutdown
            Ok(()) = ctrl_c() => {
                info!("Shutting down");
                break;
            }

            // Handle incoming connections
            Ok((stream, _)) = listener.accept() => {
                spawn(process_connection(stream, Arc::clone(&database), Arc::clone(&commands), sender.clone()));
            }

            // Network discovery
            Ok((len, addr)) = socket.recv_from(&mut buf) => {
                if let Ok(received) = from_utf8(&buf[..len]) {
                    if received == workspace_name {
                        let _ = socket.send_to(address_tcp.as_bytes(), addr).await;
                    }
                }
            }
        }
    }

    info!("Main thread exiting");
}

/// Initial connection verification
async fn process_connection (
    mut stream: TcpStream,
    database_arc: Arc<Mutex<Database>>,
    command_registry: Arc<CommandRegistry>,
    sender: UnboundedSender<bool>) {

    // Read message from client
    let message: ClientMessage = read_msg(&mut stream).await;

    // Handle verification request
    if let Verify(login_code) = message {
        let workspace = Workspace::read().await;
        if let Some(server) = workspace.server {
            if let Some(uuid) = server.login_code_map.get(&login_code) {
                if server.members.contains_key(uuid) {
                    // Send UUID to indicate successful verification
                    send_msg(&mut stream, &Uuid(uuid.clone())).await;

                    let member = server.members.get(uuid);
                    if let Some(member) = member {
                        // Process member commands
                        process_member_command(
                            &mut stream,
                            database_arc.clone(),
                            command_registry.clone(),
                            (uuid.clone(), member)
                        ).await;

                        // Send update notification
                        let _ = sender.send(true);
                    }
                } else {
                    send_msg(&mut stream, &Deny("Who are you?".to_string())).await;
                    return;
                }
            }
        } else {
            send_msg(&mut stream, &Deny("No ServerWorkspace setup!".to_string())).await;
            return;
        }
    } else {
        send_msg(&mut stream, &Deny("Please verify first.".to_string())).await;
        return;
    }
}

/// Process member commands
async fn process_member_command (
    stream: &mut TcpStream,
    database: Arc<Mutex<Database>>,
    command_registry: Arc<CommandRegistry>,
    (uuid, member): (String, &Member)
) {
    let command: ClientMessage = read_msg(stream).await;

    if let ClientMessage::Command(args_input) = command {
        let args: Vec<&str> = args_input.iter().map(String::as_str).collect();
        execute_remote_command(command_registry.as_ref(), stream, args, (uuid, member), database.clone()).await;
    }
}

pub async fn refresh_monitor(database: Arc<Mutex<Database>>, mut write_rx: UnboundedReceiver<bool>) {
    let Some(workspace) = Workspace::read().await.server else {
        return;
    };

    if workspace.enable_debug_logger {
        return;
    }

    let _ = clear();
    render_monitor(&database).await;

    loop {
        select! {
            _ = sleep(Duration::from_secs(5)) => {
                render_monitor(&database).await;
            }

            Some(_result) = write_rx.recv() => {
                render_monitor(&database).await;
            }
        }
    }
}

/// Render the monitoring display
async fn render_monitor(database: &Arc<Mutex<Database>>) {
    static LAST_LINES: AtomicUsize = AtomicUsize::new(0);

    let Some(workspace) = Workspace::read().await.server else {
        return;
    };

    let Ok(current) = current_dir() else { return; };
    let current = current.join(env!("PATH_DATABASE"));

    let mut output_buffer = String::new();

    // Storage size
    let storage_size_str = if let Ok(size) = get_folder_size(current) {
        format!("STORAGE: {:.2} MB", size as f64 / (1024.0 * 1024.0))
    } else {
        "0".to_string()
    };

    // Virtual file count
    let virtual_count_str;
    entry_mutex_async!(database, |guard| {
        virtual_count_str = format!("VIRTUAL_FILES: {}", guard.files().len());
    });

    // Memory and CPU usage
    let mut mem_str = "UNKNOWN".to_string();
    let mut cpu_str = "UNKNOWN".to_string();
    let mut sys = System::new_all();
    if let Ok(current_pid) = get_current_pid() {
        sys.refresh_all();
        if let Some(process) = sys.process(current_pid) {
            let memory_usage_mb = process.memory() as f64 / 1024.0 / 1024.0;
            let cpu_usage_percent = process.cpu_usage();
            mem_str = format!("MEM: {:.2} MB", memory_usage_mb);
            cpu_str = format!("CPU: {:.2}%", cpu_usage_percent)
        }
    }

    // Info table
    let table_info_str =
        format!("| {} | {} | {} | {} |", storage_size_str, virtual_count_str, mem_str, cpu_str);
    let table_width = table_info_str.len();

    // Table borders
    let table_top_str = "_".repeat(table_width);
    let table_bottom_str = "▔".repeat(table_width);

    // Workspace info
    let workspace_info = format!("Workspace: {}, Member: ({})", workspace.workspace_name, {
        let mut result = "".to_string();
        for (_uuid, member) in workspace.members {
            result = format!("{}, {}", result, member.member_name);
        }
        result.trim_start_matches(',').trim().to_string()
    });

    // File tree
    let mut virtual_file_path_list = Vec::new();
    entry_mutex_async!(database, |guard| {
        for file in guard.files() {
            let mut path = file.path();

            // Version
            path = format!("{} {}", path, format!("[v{}]", file.version()).green());

            // Lock status
            if let Some((_, member)) = file.get_locker_owner().await {
                let lock_status = if file.is_longer_lock_unchecked() {
                    format!("[HELD: {}]", member.member_name)
                } else {
                    format!("[held: {}]", member.member_name)
                };
                path = format!("{} {}", path, lock_status.yellow());
            }

            virtual_file_path_list.push(path);
        }
    });

    // Build complete output
    output_buffer.push('\n');
    output_buffer.push_str(&workspace_info);
    output_buffer.push('\n');
    output_buffer.push_str(&table_top_str);
    output_buffer.push('\n');
    output_buffer.push_str(&table_info_str);
    output_buffer.push('\n');
    output_buffer.push_str(&table_bottom_str);
    output_buffer.push('\n');
    output_buffer.push_str(&show_tree(virtual_file_path_list));
    output_buffer.push('\n');
    output_buffer.push('\n');

    // Calculate line count
    let current_lines = output_buffer.matches('\n').count() + 1;
    let last_lines = LAST_LINES.swap(current_lines, Ordering::SeqCst);

    // Cursor positioning and rendering
    if last_lines > 0 {
        print!("\x1B[{}A", last_lines);
    }

    // Clear from cursor to end of screen
    print!("\x1B[J");

    // Output new content
    print!("{}", output_buffer);

    let _ = std::io::stdout().flush();
}

fn get_folder_size(path: PathBuf) -> std::io::Result<u64> {
    let mut total_size: u64 = 0;
    for entry in WalkDir::new(path).into_iter() {
        let entry = entry?;
        let metadata = std::fs::metadata(entry.path())?;
        if metadata.is_file() {
            total_size += metadata.len();
        }
    }
    Ok(total_size)
}