use std::path::Path;
use std::time::Duration;
use indicatif::{ProgressBar, ProgressStyle};
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::TcpStream;
use tokio::time::Instant;
use crate::data::client_result::ClientResult;

const CHUNK_SIZE: usize = 8 * 1024;

/// 发送文件
pub async fn send_file(
    stream: &mut TcpStream,
    file: impl AsRef<Path>,
) -> Result<(), Box<dyn std::error::Error>> {

    let debug = ClientResult::debug_mode().await;

    let file_path = file.as_ref();

    // 文件检查
    if !file_path.exists() {
        return Err(format!("File not found \"{}\"", file_path.display()).into());
    }
    if file_path.is_dir() {
        return Err(format!("Path is directory \"{}\"", file_path.display()).into());
    }

    // 打开文件
    let (file_size, mut file) = {
        let file = File::open(file_path).await?;
        let metadata = file.metadata().await?;
        if metadata.len() == 0 {
            return Err("Empty file".into());
        }
        (metadata.len(), file)
    };

    // 进度条初始化
    let progress_bar = ProgressBar::new(file_size);
    if !debug {
        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.blue} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({percent}%) {eta_precise}")
                .expect("Valid style")
                .progress_chars("■■■")
        );
    }

    // 发送文件大小
    stream.write_all(&1u64.to_be_bytes()).await?;
    stream.write_all(&file_size.to_be_bytes()).await?;

    // 传输
    let mut reader = BufReader::with_capacity(CHUNK_SIZE, &mut file);
    let mut bytes_sent = 0;

    let mut last_update = Instant::now();
    let mut last_bytes = 0;

    while bytes_sent < file_size {
        let buffer = reader.fill_buf().await?;
        if buffer.is_empty() { break; }

        let chunk_size = buffer.len().min((file_size - bytes_sent) as usize);
        stream.write_all(&buffer[..chunk_size]).await?;
        reader.consume(chunk_size);

        bytes_sent += chunk_size as u64;

        if bytes_sent - last_bytes >= 256 * 1024 ||
            last_update.elapsed() >= Duration::from_millis(350)
        {
            if ! debug {
                progress_bar.set_position(bytes_sent);
            }
            last_bytes = bytes_sent;
            last_update = Instant::now();
        }
    }

    // 完整性检查
    if bytes_sent != file_size {
        return Err(format!(
            "Transfer incomplete: expected {} bytes, sent {} bytes",
            file_size, bytes_sent
        ).into());
    }

    stream.flush().await?;

    // 等待确认
    let mut ack = [0u8; 1];
    tokio::time::timeout(Duration::from_secs(10), stream.read_exact(&mut ack)).await??;

    if ack[0] != 1 {
        return Err("Receiver verification failed".into());
    }

    if ! debug {
        progress_bar.finish_with_message(format!("Sent {} bytes", bytes_sent));
    }

    Ok(())
}

/// 读取文件
pub async fn read_file(
    stream: &mut TcpStream,
    save_to: impl AsRef<Path>,
) -> Result<(), Box<dyn std::error::Error>> {

    let debug = ClientResult::debug_mode().await;

    let save_path = save_to.as_ref();

    // 检查目录存在
    if let Some(parent) = save_path.parent() {
        if !parent.exists() {
            tokio::fs::create_dir_all(parent).await?;
        }
    } else {
        return Err("Invalid save path".into());
    }

    // 接收文件元数据
    let mut version_buffer = [0u8; 8];
    stream.read_exact(&mut version_buffer).await?;
    let version = u64::from_be_bytes(version_buffer);

    if version != 1 {
        return Err("Unsupported transfer version".into());
    }

    let mut size_buffer = [0u8; 8];
    stream.read_exact(&mut size_buffer).await?;
    let file_size = u64::from_be_bytes(size_buffer);

    if file_size == 0 {
        return Err("Zero-length file transfer".into());
    }

    // 写入文件
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(save_path)
        .await
        .map_err(|e| format!("Failed to create file \"{}\"", e))?;
    let mut writer = BufWriter::with_capacity(CHUNK_SIZE, file);

    // 进度条初始化
    let progress_bar = ProgressBar::new(file_size);
    if ! debug {
        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.green/yellow}] {bytes}/{total_bytes} ({percent}%) {eta_precise}")
                .expect("Valid style")
                .progress_chars("■■■")
        );
    }

    // 接收文件内容
    let mut buffer = vec![0u8; CHUNK_SIZE];
    let mut bytes_received = 0;
    let mut buffered_bytes = 0;
    let mut last_update = Instant::now();
    let mut last_bytes = 0;

    while bytes_received < file_size {
        let read_size = std::cmp::min(
            buffer.len(),
            (file_size - bytes_received) as usize
        );
        stream.read_exact(&mut buffer[..read_size]).await?;

        writer.write_all(&buffer[..read_size]).await?;
        bytes_received += read_size as u64;
        buffered_bytes += read_size;

        if buffered_bytes >= 256 * 1024 {
            writer.flush().await?;
            buffered_bytes = 0;
        }

        if bytes_received - last_bytes >= 256 * 1024 ||
            last_update.elapsed() >= Duration::from_millis(500)
        {
            if ! debug {
                progress_bar.set_position(bytes_received);
            }
            last_bytes = bytes_received;
            last_update = Instant::now();
        }
    }

    writer.flush().await?;
    let file = writer.into_inner();
    file.sync_all().await?;

    // 完整性检查
    if bytes_received != file_size {
        let _ = tokio::fs::remove_file(save_path).await; // 清理不完整文件
        return Err(format!(
            "File incomplete: expected {} bytes, received {} bytes",
            file_size, bytes_received
        ).into());
    }

    // 发送确认信号
    stream.write_all(&[1]).await?;
    stream.flush().await?;

    if ! debug {
        progress_bar.finish_with_message(format!("Received {} bytes", bytes_received));
    }
    Ok(())
}