use std::str::from_utf8;
use std::sync::Arc;
use log::{error, info};
use log::LevelFilter::{Info, Trace};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::{select, spawn};
use tokio::signal::ctrl_c;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::Mutex;
use jam_ready::{entry_mutex_async};
use jam_ready::utils::local_archive::LocalArchive;
use jam_ready::utils::logger_build::logger_build;
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

/// 服务器入口
pub async fn jam_server_entry(
    full_logger: bool
) {

    // 构建日志，尝试获得工作区名称
    let workspace = Workspace::read();
    let mut workspace_name = "Workspace";

    if let Some(server) = &workspace.server {

        // 工作区
        workspace_name = server.workspace_name.as_str();

        // 设置 Logger
        if server.enable_debug_logger {
            logger_build(Trace, !full_logger);
        } else {
            logger_build(Info, !full_logger);
        }
    } else {
        logger_build(Info, !full_logger);
    }

    info!("/// Jam Ready! ///");
    info!("/// Workspace: \"{}\"", workspace_name);

    // 构建命令
    let commands = registry();
    info!("/// Registered {} command(s).", commands.len());
    let commands = Arc::new(commands);

    // 构建数据库
    let database = Arc::new(Mutex::new(Database::read()));
    info!("Database loaded!");

    // 构建信号

    // 保存信号
    let (write_tx, mut write_rx) : (UnboundedSender<bool>, UnboundedReceiver<bool>) = unbounded_channel();

    // 获得本机 ip
    let address_tcp = get_self_address_with_port_str(env!("DEFAULT_SERVER_PORT"));

    // 绑定 TCP 监听器
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

    // 网络发现初始化
    info!("Network discovery is enabled, listening on port: {}", DISCOVERY_PORT);
    let socket = UdpSocket::bind(format!("0.0.0.0:{}", DISCOVERY_PORT)).await.unwrap();
    let mut buf = [0u8; MAX_BUFFER_SIZE];

    // Arc 数据
    let database_write = Arc::clone(&database);

    // 进入循环
    loop {
        select! {

            // Ctrl + C 关闭
            Ok(()) = ctrl_c() => {
                info!("Shutting down");
                break;
            }

            // 更新数据库
            Some(result) = write_rx.recv() => {
                if result {
                    entry_mutex_async!(database_write, |guard| {
                        Database::update(guard);
                    });
                }
            }

            // 接收请求
            Ok((stream, _)) = listener.accept() => {
                spawn(process_connection(stream, Arc::clone(&database), Arc::clone(&commands), write_tx.clone()));
            }

            // 网络发现
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

/// 初次验证该请求
async fn process_connection (
    mut stream: TcpStream,
    database_arc: Arc<Mutex<Database>>,
    command_registry: Arc<CommandRegistry>,
    write_tx: UnboundedSender<bool>) {

    // 从客户端读取消息
    let message: ClientMessage = read_msg(&mut stream).await;

    // 若接收到验证请求则继续
    if let Verify(login_code) = message {

        // 尝试拿到工作区数据
        let workspace = Workspace::read();
        if let Some(server) = workspace.server {

            // 通过 登录代码 拿到 Uuid
            if let Some(uuid) = server.login_code_map.get(&login_code) {

                // 存在该成员则继续
                if server.members.contains_key(uuid) {

                    // 发送 Uuid 代表该用户通过
                    send_msg(&mut stream, &Uuid(uuid.clone())).await;

                    let member = server.members.get(uuid);
                    if let Some(member) = member {

                        // 继续处理
                        process_member_command(
                            &mut stream,
                            database_arc.clone(),
                            command_registry.clone(),
                            write_tx.clone(),
                            (uuid.clone(), member)
                        ).await;
                    }
                } else {

                    // 发送失败信息
                    send_msg(&mut stream, &Deny("Who are you?".to_string())).await;
                    return;
                }
            }
        } else {

            // 发送失败信息
            send_msg(&mut stream, &Deny("No ServerWorkspace setup!".to_string())).await;
            return;
        }
    } else {

        // 发送失败信息
        send_msg(&mut stream, &Deny("Please verify first.".to_string())).await;
        return;
    }
}

/// 处理成员命令
async fn process_member_command (
    stream: &mut TcpStream,
    database: Arc<Mutex<Database>>,
    command_registry: Arc<CommandRegistry>,
    sender: UnboundedSender<bool>,
    (uuid, member): (String, &Member)
) {
    // 接收命令
    let command: ClientMessage = read_msg(stream).await;

    if let ClientMessage::Command(args_input) = command {
        let args: Vec<&str> = args_input.iter().map(String::as_str).collect();

        // 进入命令
        entry_mutex_async!(database, |database_guard| {
            let changed = execute_remote_command(command_registry.as_ref(), stream, args, (uuid, member), database_guard).await;
            if changed {
                let _ = sender.send(true);
            }
        })
    }
}