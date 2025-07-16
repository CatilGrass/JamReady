use std::path::Path;
use std::time::Duration;
use indicatif::{ProgressBar, ProgressStyle};
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::TcpStream;

const CHUNK_SIZE: usize = 8 * 1024; // 优化为8KB缓冲区

/// 发送文件
pub async fn send_file(
    stream: &mut TcpStream,
    file: impl AsRef<Path>,
) -> Result<(), Box<dyn std::error::Error>> {
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
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.blue} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({percent}%) {eta_precise}")
            .expect("Valid style")
            .progress_chars("■■■")
    );

    // 发送文件大小
    stream.write_all(&1u64.to_be_bytes()).await?;
    stream.write_all(&file_size.to_be_bytes()).await?;

    // 传输
    let mut reader = BufReader::with_capacity(CHUNK_SIZE, &mut file);
    let mut bytes_sent = 0;

    while bytes_sent < file_size {
        let buffer = reader.fill_buf().await?;
        if buffer.is_empty() { break; }

        let chunk_size = buffer.len().min((file_size - bytes_sent) as usize);
        stream.write_all(&buffer[..chunk_size]).await?;
        reader.consume(chunk_size);

        bytes_sent += chunk_size as u64;
        progress_bar.set_position(bytes_sent);
    }

    // 完整性和刷新检查
    if bytes_sent != file_size {
        return Err(format!(
            "Transfer incomplete: expected {} bytes, sent {} bytes",
            file_size, bytes_sent
        ).into());
    }

    stream.flush().await?;

    // 等待接收方确认
    let mut ack = [0u8; 1];
    tokio::time::timeout(Duration::from_secs(10), stream.read_exact(&mut ack)).await??;

    if ack[0] != 1 {
        return Err("Receiver verification failed".into());
    }

    progress_bar.finish_with_message(format!("Sent {} bytes", bytes_sent));
    Ok(())
}

/// 读取文件
pub async fn read_file(
    stream: &mut TcpStream,
    save_to: impl AsRef<Path>,
) -> Result<(), Box<dyn std::error::Error>> {
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

    // 原子性写入文件
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
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.green/yellow}] {bytes}/{total_bytes} ({percent}%) {eta_precise}")
            .expect("Valid style")
            .progress_chars("■■■")
    );

    // 接收文件内容
    let mut buffer = vec![0u8; CHUNK_SIZE];
    let mut bytes_received = 0;

    while bytes_received < file_size {
        // 计算当前需要读取的大小
        let read_size = std::cmp::min(
            buffer.len(),
            (file_size - bytes_received) as usize
        );

        // 精确读取指定大小的数据
        stream.read_exact(&mut buffer[..read_size]).await?;

        // 写入文件
        writer.write_all(&buffer[..read_size]).await?;
        bytes_received += read_size as u64;
        progress_bar.set_position(bytes_received);

        // 定期刷新缓冲区
        if bytes_received % (CHUNK_SIZE as u64 * 10) == 0 {
            writer.flush().await?;
        }
    }

    // 强制刷新并同步到磁盘
    writer.flush().await?;
    let file = writer.into_inner(); // 取出内部文件
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

    progress_bar.finish_with_message(format!("Received {} bytes", bytes_received));
    Ok(())
}