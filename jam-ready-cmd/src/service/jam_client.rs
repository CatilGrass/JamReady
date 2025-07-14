use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::net::{TcpStream, UdpSocket};
use jam_ready::connect_once;
use jam_ready::utils::local_archive::LocalArchive;
use crate::data::workspace::{ClientWorkspace, Workspace};
use crate::service::commands::registry;
use crate::service::jam_command::execute_local_command;
use crate::service::messages::ServerMessage;
use crate::service::messages::ClientMessage::{Command, Verify};
use crate::service::messages::ServerMessage::Uuid;
use crate::service::service_utils::{read_msg, send_msg};

/// 执行命令
pub async fn execute(command_input: Vec<String>) {
    let mut workspace = Workspace::read();

    if let Some(client) = &mut workspace.client {

        // 尝试使用目标地址连接
        let addr = client.target_addr;

        // 连接、验证并取得流
        let stream = try_verify_connection(addr, client).await;

        let mut args_input = Vec::new();
        for arg in command_input.iter() {
            args_input.push(arg.as_str());
        }

        // 若成功取得流，进入正式操作
        if let Some(mut stream) = stream {

            // 发送命令
            send_msg(&mut stream, &Command(command_input.clone())).await;

            // 进入命令
            execute_local_command(&registry(), &mut stream, args_input).await;
        }
    }

    Workspace::update(&workspace);
}

async fn try_verify_connection(addr: SocketAddr, client: &mut ClientWorkspace) -> Option<TcpStream> {
    connect_once!(addr, |stream| {

        // 发送 登录代码 尝试验证
        send_msg(&mut stream, &Verify(client.login_code.clone())).await;

        // 接收消息
        let message: ServerMessage = read_msg(&mut stream).await;

        // 判断接收的消息
        match message {

            // 收到 Uuid 表示验证成功
            Uuid(uuid) => {

                // 若 Uuid 字符串字数不够
                if client.uuid.trim().len() < 32 {
                    client.uuid = uuid;
                }

                Some(stream)
            }
            ServerMessage::Deny(error) => {
                eprintln!("Server denied your connection: {}", error);
                None
            }
            _ => None
        }
    })
}

// --------------------------------------------------------------------------- //

const DISCOVERY_PORT: u16 = 54000;
const MAX_BUFFER_SIZE: usize = 1024;

// 网络发现一次
pub async fn search_workspace_lan(workspace_name: String) -> Result<SocketAddr, Box<dyn std::error::Error>> {
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    socket.set_broadcast(true)?;

    let target_addr = format!("255.255.255.255:{}", DISCOVERY_PORT);
    socket.send_to(workspace_name.trim().as_bytes(), &target_addr).await?;

    let mut buf = [0u8; MAX_BUFFER_SIZE];
    let (len, _) = socket.recv_from(&mut buf).await?;
    let response = std::str::from_utf8(&buf[..len])?;

    parse_socket_addr(response)
}

fn parse_socket_addr(addr_str: &str) -> Result<SocketAddr, Box<dyn std::error::Error>> {
    let parts: Vec<&str> = addr_str.split(':').collect();
    if parts.len() != 2 {
        return Err(format!("Invalid address format: {}", addr_str).into());
    }

    let ip = parts[0].parse::<Ipv4Addr>()
        .map_err(|_| format!("Invalid IPv4 address: {}", parts[0]))?;

    let port = parts[1].parse::<u16>()
        .map_err(|_| format!("Invalid port number: {}", parts[1]))?;

    Ok(SocketAddr::new(IpAddr::V4(ip), port))
}