use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;
use tokio::net::{TcpStream, UdpSocket};
use jam_ready::connect_once;
use jam_ready::utils::local_archive::LocalArchive;
use crate::data::client_result::ClientResult;
use crate::data::local_folder_map::LocalFolderMap;
use crate::data::parameters::{read_parameter};
use crate::data::workspace::{ClientWorkspace, Workspace};
use crate::service::commands::registry;
use crate::service::jam_command::execute_local_command;
use crate::service::messages::ServerMessage;
use crate::service::messages::ClientMessage::{Command, Verify};
use crate::service::messages::ServerMessage::Uuid;
use crate::service::service_utils::{read_msg, send_msg};

/// 执行命令
pub async fn execute(command_input: Vec<String>) -> Option<ClientResult> {

    let mut workspace = Workspace::read().await;
    let mut result = None;

    if let Some(client) = &mut workspace.client {

        // 尝试使用目标地址连接
        let addr = client.target_addr;

        // 连接、验证并取得流
        let stream = try_verify_connection(addr, client).await;

        let mut args_input = Vec::new();
        for arg in command_input.iter() {
            if arg.ends_with("?") {
                // 若命令文本中存在 ? 说明有参数
                let param = arg.to_lowercase().replace("?", "");
                if let Some(content) = read_parameter(param) {
                    args_input.push(content.trim().to_string());
                }
            } else if arg.starts_with(":") {
                // 若命令文本开头为 : 说明为简写
                let folder_map = LocalFolderMap::read().await;
                let full = folder_map.short_file_map.get(&arg.replace(":", "").to_string());
                if let Some(full) = full {
                    args_input.push(full.to_string());
                }
            } else {
                // 正常加入
                args_input.push(arg.clone());
            }
        }
        let args = args_input.iter().map(String::as_str).collect::<Vec<&str>>();

        // 若成功取得流，进入正式操作
        if let Some(mut stream) = stream {

            // 发送命令
            send_msg(&mut stream, &Command(args_input.clone())).await;

            // 进入命令
            result = execute_local_command(&registry(), &mut stream, args).await;
        }
    }

    Workspace::update(&workspace).await;
    result
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
                client.uuid = uuid;
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

    let target_addr = format!("{}:{}", get_broadcast_address(), DISCOVERY_PORT);
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

fn get_broadcast_address() -> Ipv4Addr {
    let local_address =
        Ipv4Addr::from_str(
            local_ipaddress::get().unwrap_or("127.0.0.1".to_string()).as_str()
        ).unwrap_or(Ipv4Addr::new(127, 0, 0, 1));
    let mask_address = Ipv4Addr::new(255, 255, 255, 0);

    let ip_int = u32::from(local_address);
    let mask_int = u32::from(mask_address);
    let network_address = ip_int & mask_int;
    let broadcast_address = network_address | !mask_int;
    Ipv4Addr::from(broadcast_address)
}