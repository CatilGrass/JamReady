use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::net::lookup_host;

pub async fn parse_address_v4_str(address: String) -> std::io::Result<SocketAddr> {
    let default_port: u16 = env!("DEFAULT_SERVER_PORT").parse().unwrap();

    // Try to parse as SocketAddr
    if let Ok(socket_addr) = address.parse::<SocketAddr>() {
        return Ok(match socket_addr.ip() {
            IpAddr::V4(_) => socket_addr,
            IpAddr::V6(_) => SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), socket_addr.port()),
        });
    }

    // Parse as pure IPv6 address
    if let Ok(_v6_addr) = address.parse::<std::net::Ipv6Addr>() {
        return Ok(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), default_port));
    }

    // Split address and port parts
    let (host, port_str) = if let Some((host, port)) = address.rsplit_once(':') {
        // Handle IPv6
        (host.trim_matches(|c| c == '[' || c == ']'), Some(port))
    } else {
        (&address[..], None)
    };

    // Parse port
    let port = port_str
        .and_then(|p| p.parse::<u16>().ok())
        .map(|p| p.clamp(0, u16::MAX))
        .unwrap_or(default_port);

    // Async hostname resolution
    let socket_iter = lookup_host((host, 0)).await?;

    // Find first IPv4 address
    if let Some(addr) = socket_iter.filter(|addr| addr.is_ipv4()).next() {
        return Ok(SocketAddr::new(addr.ip(), port));
    }

    // Fallback to local address if resolution fails
    Ok(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port))
}