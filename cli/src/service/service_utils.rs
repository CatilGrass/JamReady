use std::fmt::Debug;
use std::io::ErrorKind;
use indicatif::{ProgressBar, ProgressStyle};
use log::{error, trace, warn};
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use tokio::io;
use tokio::net::TcpStream;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;

/// Send message using JSON serialization
pub async fn send_msg<Message>(
    stream: &mut TcpStream,
    msg: &Message
) where Message: Serialize + Debug {
    // Serialize message
    match serde_json::to_string(msg) {
        Ok(json_str) => {
            // Convert to bytes and write to stream
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

/// Read message using JSON deserialization
pub async fn read_msg<Message>(
    stream: &mut TcpStream
) -> Message
where
    Message: DeserializeOwned + Default + Debug
{
    let mut buffer = Vec::new();
    let mut temp_buf = [0u8; 128];

    // Read data in chunks
    while let Ok(n) = stream.read(&mut temp_buf).await {
        if n == 0 { break; } // Connection closed
        buffer.extend_from_slice(&temp_buf[..n]);

        // Attempt to deserialize received data
        match serde_json::from_slice::<Message>(&buffer) {
            Ok(decoded) => {
                trace!("Received JSON {:?} from {}", decoded, get_target_address(stream));
                return decoded;
            }
            Err(err) if err.is_eof() || err.is_data() => {
                // Incomplete data, continue reading
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

/// Send large message with progress tracking
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

/// Read large message with progress tracking
pub async fn read_large_msg<Message>(
    stream: &mut TcpStream,
    progress_bar: Option<ProgressBar>
) -> Result<Message, Box<dyn std::error::Error + Send>>
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

/// Send large text with chunked transfer and progress tracking
pub async fn send_large_text(
    stream: &mut TcpStream,
    text: &str,
    progress_bar: Option<ProgressBar>,
) -> io::Result<()> {
    const MAX_TEXT_SIZE: usize = 100 * 1024 * 1024; // 100MB limit
    const CHUNK_SIZE: usize = 16 * 1024; // 16KB chunks

    let text_bytes = text.as_bytes();
    let total_size = text_bytes.len();

    // Validate size limit
    if total_size > MAX_TEXT_SIZE {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("Message too large: {:.1}MB exceeds {:.1}MB limit",
                    total_size as f32 / (1024.0 * 1024.0),
                    MAX_TEXT_SIZE as f32 / (1024.0 * 1024.0))
        ));
    }

    // Setup progress bar if provided
    if let Some(bar) = &progress_bar {
        bar.set_length(total_size as u64);
        bar.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.green/yellow}] {bytes}/{total_bytes} ({percent}%) {eta_precise}")
                .expect("Valid style")
                .progress_chars("■■■")
        );
    }

    // Send size header (8 bytes)
    let size_header = (total_size as u64).to_be_bytes();
    stream.write_all(&size_header).await?;
    if let Some(bar) = &progress_bar {
        bar.inc(8);
    }

    // Send content in chunks
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

    // Finish progress
    if let Some(bar) = &progress_bar {
        bar.finish_with_message("Done");
    }
    Ok(())
}

/// Read large text with chunked transfer and progress tracking
pub async fn read_large_text(
    stream: &mut TcpStream,
    progress_bar: Option<ProgressBar>,
) -> io::Result<String> {
    const MAX_TEXT_SIZE: usize = 100 * 1024 * 1024; // 100MB
    const HEADER_SIZE: usize = 8;
    const CHUNK_SIZE: usize = 16 * 1024;

    // Read size header
    let mut size_buf = [0u8; HEADER_SIZE];
    stream.read_exact(&mut size_buf).await?;
    let total_size = u64::from_be_bytes(size_buf) as usize;

    // Validate size limit
    if total_size > MAX_TEXT_SIZE {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("Message too large: {:.1}MB exceeds {:.1}MB limit",
                    total_size as f32 / (1024.0 * 1024.0),
                    MAX_TEXT_SIZE as f32 / (1024.0 * 1024.0))
        ));
    }

    // Setup progress bar if provided
    if let Some(bar) = &progress_bar {
        bar.set_length(total_size as u64);
        bar.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.blue} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({percent}%) {eta_precise}")
                .expect("Valid style")
                .progress_chars("■■■")
        );
    }

    // Read content in chunks
    let mut buffer = Vec::with_capacity(total_size);
    let mut bytes_read = 0;

    while bytes_read < total_size {
        let chunk_size = CHUNK_SIZE.min(total_size - bytes_read);
        buffer.resize(bytes_read + chunk_size, 0);

        let chunk = &mut buffer[bytes_read..bytes_read + chunk_size];
        stream.read_exact(chunk).await?;

        bytes_read += chunk_size;
        if let Some(bar) = &progress_bar {
            bar.inc(chunk_size as u64);
        }
    }

    // Finish progress
    if let Some(bar) = &progress_bar {
        bar.finish_with_message("Done");
    }

    // Convert to UTF-8
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

/// Get peer address from TCP stream
pub fn get_target_address(stream: &TcpStream) -> String {
    stream.peer_addr()
        .map(|addr| addr.to_string())
        .unwrap_or_else(|_| "Unknown".to_string())
}

/// Get local address with default port
pub fn get_self_address() -> String {
    let port_str = env!("DEFAULT_SERVER_PORT");
    get_self_address_with_port_str(port_str)
}

/// Get local address with specified port
pub fn get_self_address_with_port_str(port: &str) -> String {
    local_ipaddress::get()
        .map(|ip| format!("{}:{}", ip, port))
        .unwrap_or_else(|| format!("127.0.0.1:{}", port))
}