use crate::cli_commands::client::client_workspace_main;
use crate::data::workspace::{debug_mode, Workspace};
use crate::linker::linker_config::LinkerConfig;
use jam_ready::utils::local_archive::LocalArchive;
use jam_ready::utils::text_process::split_to_args;
use std::env::current_dir;
use std::io::{Error, ErrorKind};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::signal::ctrl_c;
use tokio::time::sleep;
use tokio::select;

pub async fn jam_linker_entry(linker_config: LinkerConfig) {

    // Workspace type check
    let workspace = Workspace::read().await;
    let Some(workspace) = workspace.client else {
        eprintln!("It's not a client workspace.");
        return;
    };

    let addr = linker_config.get_addr();
    let workspace_directory = current_dir().unwrap().to_str().unwrap().to_string();
    println!("Workspace: \"{}\", Directory: {}, Address: {}", workspace.workspace_name, workspace_directory, addr);

    // Bind listener
    let Ok(listener) = TcpListener::bind(addr).await else {
        eprintln!("Bind listener failed");
        return;
    };

    // Enable debug (linker mode)
    debug_mode(true).await;

    let sleep_duration = (60.0 * linker_config.sleep_minutes).clamp(10.0, 60.0 * 60.0 * 24.0);

    loop {
        select! {

            // Auto shutdown
            _ = sleep(Duration::from_secs_f64(sleep_duration)) => {
                let exit_text = "\"Zzz... Wake me up when you need me.\"";
                println!("{}", exit_text);
                break;
            }

            // Ctrl + C shutdown
            Ok(()) = ctrl_c() => {
                println!("Good bye!");
                break;
            }

            // Process accept
            Ok((stream, _)) = listener.accept() => {

                // Only one connection is allowed.
                // spawn(process_input(stream));
                process_input(stream).await;
            }
        }
    }

    // Disable debug (linker mode)
    debug_mode(false).await;
}

async fn process_input(mut stream: TcpStream) {

    // Buffer
    let mut buffer :[u8; 2048] = [0; 2048];

    let addr_str =
        if let Ok(addr) = stream.peer_addr() { addr.to_string() } else { "Unknown".to_string() };
    println!("[{}] Login.", addr_str);

    loop {

        // Read message
        let Ok(received) = read(&mut stream, &mut buffer).await else { continue };
        let received = received.trim().to_string();
        println!("~# {}", received);

        // Invoke command & Read result
        let result_str = if let Some(result) = client_workspace_main(split_to_args(received)).await {
            result.end_print()
        } else {
            "{}".to_string()
        };

        // Display result
        println!("{}", result_str);

        // Write message
        match write_str(&mut stream, &result_str).await {
            Ok(_) => {}
            Err(_) => {
                break;
            }
        }
    }

    println!("[{}] Logout.", addr_str);
}

async fn read(stream: &mut TcpStream, buffer: &mut [u8; 2048]) -> std::io::Result<String> {
    let read = stream.read(buffer).await?;
    let read_str = String::from_utf8_lossy(&buffer[..read]).to_string();
    if read_str.is_empty() {
        return Err(Error::new(ErrorKind::NotFound, "Input not found."));
    }
    Ok(read_str)
}

async fn write(stream: &mut TcpStream, content: String) -> std::io::Result<()> {
    let _ = stream.write(content.as_bytes()).await;
    stream.flush().await
}

async fn write_str(stream: &mut TcpStream, content: &str) -> std::io::Result<()> {
    write(stream, content.to_string()).await
}