use std::path::Path;
use std::time::Duration;
use indicatif::{ProgressBar, ProgressStyle};
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::TcpStream;
use tokio::time::Instant;
use crate::data::client_result::ClientResult;

const CHUNK_SIZE: usize = 8 * 1024;
const PROGRESS_UPDATE_THRESHOLD: u64 = 256 * 1024;
const PROGRESS_UPDATE_INTERVAL: Duration = Duration::from_millis(350);

/// Sends a file over TCP with progress tracking
pub async fn send_file(
    stream: &mut TcpStream,
    file_path: impl AsRef<Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    let debug = ClientResult::debug_mode().await;
    let path = file_path.as_ref();

    // Validate file
    if !path.exists() {
        return Err(format!("File not found: {}", path.display()).into());
    }
    if path.is_dir() {
        return Err(format!("Path is directory: {}", path.display()).into());
    }

    // Open file and get metadata
    let mut file = File::open(path).await?;
    let file_size = file.metadata().await?.len();
    if file_size == 0 {
        return Err("Cannot send empty file".into());
    }

    // Initialize progress bar
    let progress_bar = if !debug {
        let pb = ProgressBar::new(file_size);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.blue} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({percent}%) {eta_precise}")
                .unwrap()
                .progress_chars("■■■")
        );
        pb
    } else {
        ProgressBar::hidden()
    };

    // Send file header (version + size)
    stream.write_all(&1u64.to_be_bytes()).await?;
    stream.write_all(&file_size.to_be_bytes()).await?;

    // Transfer file content
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

        // Update progress periodically
        if bytes_sent - last_bytes >= PROGRESS_UPDATE_THRESHOLD ||
            last_update.elapsed() >= PROGRESS_UPDATE_INTERVAL
        {
            progress_bar.set_position(bytes_sent);
            last_bytes = bytes_sent;
            last_update = Instant::now();
        }
    }

    // Verify transfer completion
    if bytes_sent != file_size {
        return Err(format!(
            "Transfer incomplete: expected {} bytes, sent {} bytes",
            file_size, bytes_sent
        ).into());
    }

    stream.flush().await?;

    // Wait for receiver confirmation
    let mut ack = [0u8; 1];
    tokio::time::timeout(Duration::from_secs(10), stream.read_exact(&mut ack)).await??;

    if ack[0] != 1 {
        return Err("Receiver verification failed".into());
    }

    progress_bar.finish_with_message(format!("Sent {} bytes", bytes_sent));
    Ok(())
}

/// Receives a file over TCP with progress tracking
pub async fn read_file(
    stream: &mut TcpStream,
    save_path: impl AsRef<Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    let debug = ClientResult::debug_mode().await;
    let path = save_path.as_ref();

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            tokio::fs::create_dir_all(parent).await?;
        }
    }

    // Read file header (version + size)
    let version = stream.read_u64().await?;
    if version != 1 {
        return Err("Unsupported transfer version".into());
    }

    let file_size = stream.read_u64().await?;
    if file_size == 0 {
        return Err("Cannot receive zero-length file".into());
    }

    // Prepare output file
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .await?;
    let mut writer = BufWriter::with_capacity(CHUNK_SIZE, file);

    // Initialize progress bar
    let progress_bar = if !debug {
        let pb = ProgressBar::new(file_size);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.green/yellow}] {bytes}/{total_bytes} ({percent}%) {eta_precise}")
                .unwrap()
                .progress_chars("■■■")
        );
        pb
    } else {
        ProgressBar::hidden()
    };

    // Receive file content
    let mut buffer = vec![0u8; CHUNK_SIZE];
    let mut bytes_received = 0;
    let mut last_update = Instant::now();
    let mut last_bytes = 0;

    while bytes_received < file_size {
        let read_size = buffer.len().min((file_size - bytes_received) as usize);
        stream.read_exact(&mut buffer[..read_size]).await?;

        writer.write_all(&buffer[..read_size]).await?;
        bytes_received += read_size as u64;

        // Flush periodically and update progress
        if bytes_received - last_bytes >= PROGRESS_UPDATE_THRESHOLD ||
            last_update.elapsed() >= PROGRESS_UPDATE_INTERVAL
        {
            writer.flush().await?;
            progress_bar.set_position(bytes_received);
            last_bytes = bytes_received;
            last_update = Instant::now();
        }
    }

    // Final flush and sync
    writer.flush().await?;
    writer.into_inner().sync_all().await?;

    // Verify completion
    if bytes_received != file_size {
        let _ = tokio::fs::remove_file(path).await;
        return Err(format!(
            "Transfer incomplete: expected {} bytes, received {} bytes",
            file_size, bytes_received
        ).into());
    }

    // Send confirmation
    stream.write_all(&[1]).await?;
    stream.flush().await?;

    progress_bar.finish_with_message(format!("Received {} bytes", bytes_received));
    Ok(())
}