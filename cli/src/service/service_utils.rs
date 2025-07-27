use std::fmt::Debug;
use std::io::ErrorKind;
use indicatif::{ProgressBar, ProgressStyle};
use log::{error, info, trace, warn};
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use tokio::io;
use tokio::net::TcpStream;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;

/// 发送消息 - 使用 JSON 序列化
pub async fn send_msg<Message>(
    stream: &mut TcpStream,
    msg: &Message
) where Message: Serialize + Debug {
    // 序列化
    match serde_json::to_string(msg) {
        Ok(json_str) => {

            // 转换为字节
            let bytes = json_str.as_bytes();
            match stream.write_all(bytes).await {
                Ok(_) => {
                    trace!("Sent JSON {:?} to {}", msg, get_target_address(stream));
                }
                Err(err) => {
                    warn!("Failed to send JSON message: {}", err);
                }
            }
        }
        Err(err) => {
            error!("Failed to serialize message to JSON: {}", err);
        }
    }
}

/// 读取消息 - 使用 JSON 反序列化
pub async fn read_msg<Message>(
    stream: &mut TcpStream
) -> Message
where
    Message: DeserializeOwned + Default + Debug
{
    let mut buffer = Vec::new();
    let mut temp_buf = [0u8; 128];

    // 读取
    while let Ok(n) = stream.read(&mut temp_buf).await {
        if n == 0 { break; } // 连接关闭
        buffer.extend_from_slice(&temp_buf[..n]);

        // 反序列化接收到的数据
        match serde_json::from_slice::<Message>(&buffer) {
            Ok(decoded) => {
                trace!("Received JSON {:?} from {}", decoded, get_target_address(stream));
                return decoded;
            }
            Err(err) if err.is_eof() || err.is_data() => {
                // 数据不完整则继续
                continue;
            }
            Err(err) => {
                error!("Failed to deserialize JSON message: {}", err);
                return Message::default();
            }
        }
    }

    warn!("Connection closed before complete JSON message received");
    Message::default()
}

/// 发送大消息
pub async fn send_large_msg<Message>(
    stream: &mut TcpStream,
    msg: &Message,
    progress_bar: Option<ProgressBar>
) -> Result<(), Box<dyn std::error::Error>>
where for<'a> Message: Serialize + Deserialize<'a> + Default + Debug
{
    let content = serde_yaml::to_string(msg);
    match content {
        Ok(content) => {
            match send_large_text(stream, content.as_str(), progress_bar).await {
                Ok(_) => {
                    Ok(())
                }
                Err(error) => {
                    Err(Box::new(error))
                }
            }
        }
        Err(error) => {
            Err(Box::new(error))
        }
    }
}

/// 读取大消息
pub async fn read_large_msg<Message>(
    stream: &mut TcpStream,
    progress_bar: Option<ProgressBar>
) -> Result<Message, Box<dyn std::error::Error>>
where
        for<'a> Message: Serialize + Deserialize<'a> + Default + Debug
{
    let read = read_large_text(stream, progress_bar).await;
    match read {
        Ok(content) => {
            match serde_yaml::from_str::<Message>(&content) {
                Ok(msg) => {
                    Ok(msg)
                }
                Err(error) => {
                    Err(Box::new(error))
                }
            }
        }
        Err(error) => {
            Err(Box::new(error))
        }
    }
}

/// 发送大文本
pub async fn send_large_text(
    stream: &mut TcpStream,
    text: &str,
    progress_bar: Option<ProgressBar>,
) -> io::Result<()> {

    // 大小限制
    const MAX_TEXT_SIZE: usize = 100 * 1024 * 1024;

    let text_bytes = text.as_bytes();
    let total_size = text_bytes.len();

    if total_size > MAX_TEXT_SIZE {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("Failed to send: Message too large, {:.1}Mb exceeds the {:.1}Mb limit",
                    total_size as f32 / (1024.0 * 1024.0),
                    MAX_TEXT_SIZE as f32 / (1024.0 * 1024.0))
        ));
    }

    // 设置进度条
    if let Some(bar) = &progress_bar {
        bar.set_length(total_size as u64);
        bar.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.green/yellow}] {bytes}/{total_bytes} ({percent}%) {eta_precise}")
                .expect("Valid style")
                .progress_chars("■■■")
        );
    }

    // 发送文本长度
    let size_header = (total_size as u64).to_be_bytes();
    stream.write_all(&size_header).await?;
    if let Some(bar) = &progress_bar {
        bar.inc(8);
    }

    // 发送文本内容
    const CHUNK_SIZE: usize = 16 * 1024;
    let mut bytes_sent = 0;

    while bytes_sent < total_size {
        let end = (bytes_sent + CHUNK_SIZE).min(total_size);
        let chunk = &text_bytes[bytes_sent..end];

        stream.write_all(chunk).await?;

        let chunk_len = chunk.len();
        bytes_sent += chunk_len;
        if let Some(bar) = &progress_bar {
            bar.inc(chunk_len as u64);
        }
    }
    stream.flush().await?;

    // 完成
    if let Some(bar) = &progress_bar {
        bar.finish_with_message("Done");
    }
    Ok(())
}

/// 读取大文本
pub async fn read_large_text(
    stream: &mut TcpStream,
    progress_bar: Option<ProgressBar>,
) -> io::Result<String> {

    // 大小限制
    const MAX_TEXT_SIZE: usize = 100 * 1024 * 1024;
    const HEADER_SIZE: usize = 8;

    // 文本长度
    let mut size_buf = [0u8; HEADER_SIZE];
    stream.read_exact(&mut size_buf).await?;
    let total_size = u64::from_be_bytes(size_buf) as usize;

    if total_size > MAX_TEXT_SIZE {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("Failed to send: Message too large, {:.1}Mb exceeds the {:.1}Mb limit",
                    total_size as f32 / (1024.0 * 1024.0),
                    MAX_TEXT_SIZE as f32 / (1024.0 * 1024.0))
        ));
    }

    // 进度条
    if let Some(bar) = &progress_bar {
        bar.set_length(total_size as u64);
        bar.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.blue} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({percent}%) {eta_precise}")
                .expect("Valid style")
                .progress_chars("■■■")
        );
    }

    // 读取内容
    let mut buffer = Vec::with_capacity(total_size);
    let mut bytes_read = 0;
    const CHUNK_SIZE: usize = 16 * 1024;

    while bytes_read < total_size {
        // 当前块大小
        let chunk_size = CHUNK_SIZE.min(total_size - bytes_read);

        // 调整缓冲区大小
        buffer.resize(bytes_read + chunk_size, 0);

        // 读取数据块
        let chunk = &mut buffer[bytes_read..bytes_read + chunk_size];
        stream.read_exact(chunk).await?;

        // 更新状态
        bytes_read += chunk_size;
        if let Some(bar) = &progress_bar {
            bar.inc(chunk_size as u64);
        }
    }

    if let Some(bar) = &progress_bar {
        bar.finish_with_message("Done");
    }

    // 转换为 UTF-8
    String::from_utf8(buffer).map_err(|e| {
        if let Some(bar) = &progress_bar {
            bar.abandon_with_message("");
        }
        io::Error::new(
            ErrorKind::InvalidData,
            format!("Invalid UTF-8 data: {}", e)
        )
    })
}

/// 获得目标地址
pub fn get_target_address(stream: &TcpStream) -> String {
    let peer = stream.peer_addr();
    if peer.is_ok() {
        peer.unwrap().to_string()
    } else {
        "Unknown".to_string()
    }
}

/// 获得本机地址
pub fn get_self_address() -> String {
    let port_str = env!("DEFAULT_SERVER_PORT");
    get_self_address_with_port_str(port_str)
}

/// 获得本机地址
pub fn get_self_address_with_port_str(port: &str) -> String {
    let mut address: String = format!("127.0.0.1:{}", &port);
    if let Some(ip) = local_ipaddress::get() {
        address = format!("{}:{}", ip, &port);
        info!("Bind address: {}", &address);
    } else {
        info!("Bind address: 127.0.0.1:{}", &port);
    }
    address
}