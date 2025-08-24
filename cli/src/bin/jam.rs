use jam_ready_cli::cli_commands::setup::cli_entry;
use jam_ready_cli::try_correct_current_dir;

#[tokio::main]
async fn main() {
    let _ = try_correct_current_dir();
    cli_entry().await;
}