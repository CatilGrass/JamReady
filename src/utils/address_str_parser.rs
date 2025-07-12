use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::net::lookup_host;

pub async fn parse_address_v4_str(address: String) -> std::io::Result<SocketAddr> {
    let default_port : u16 = env!("DEFAULT_SERVER_PORT").parse().unwrap();

    // 尝试解析为 SocketAddr
    if let Ok(socket_addr) = address.parse::<SocketAddr>() {
        return Ok(match socket_addr.ip() {
            IpAddr::V4(_) => socket_addr,
            IpAddr::V6(_) => SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), socket_addr.port()),
        });
    }

    // 解析为纯 IPv6 地址
    if let Ok(_v6_addr) = address.parse::<std::net::Ipv6Addr>() {
        return Ok(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), default_port));
    }

    // 拆分地址和端口部分
    let (host, port_str) = if let Some((host, port)) = address.rsplit_once(':') {
        // 处理 IPv6
        (host.trim_matches(|c| c == '[' || c == ']'), Some(port))
    } else {
        (&address[..], None)
    };

    // 解析端口
    let port = port_str
        .and_then(|p| p.parse::<u16>().ok())
        .map(|p| p.clamp(0, u16::MAX))
        .unwrap_or(default_port);

    // 异步解析主机名
    let socket_iter = lookup_host((host, 0)).await?;

    // 寻找第一个 IPv4 地址
    if let Some(addr) = socket_iter.filter(|addr| addr.is_ipv4()).next() {
        return Ok(SocketAddr::new(addr.ip(), port));
    }

    // 解析失败时使用本地地址
    Ok(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port))
}