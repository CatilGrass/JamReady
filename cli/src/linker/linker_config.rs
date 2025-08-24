use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use jam_ready::utils::local_archive::LocalArchive;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LinkerConfig {
    pub port: u16,
    pub sleep_minutes: f64
}

impl Default for LinkerConfig {
    fn default() -> Self {
        Self {
            port: u16::from_str(env!("DEFAULT_LINKER_PORT")).unwrap(),
            sleep_minutes: 15.0,
        }
    }
}

impl LocalArchive for LinkerConfig {
    type DataType = LinkerConfig;

    fn relative_path() -> String {
        env!("FILE_LINKER_CONFIG").to_string()
    }
}

impl LinkerConfig {

    pub fn get_addr(&self) -> SocketAddr {
        SocketAddr::from_str(format!("127.0.0.1:{}", self.port).as_str())
            .unwrap_or(
                SocketAddr::new(
                    IpAddr::V4(
                        Ipv4Addr::new(127, 0, 0, 1)
                    ), self.port
                )
            )
    }
}