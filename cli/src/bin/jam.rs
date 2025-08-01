use jam_ready_cli::cli_commands::setup::cli_entry;

#[tokio::main]
async fn main() {
    cli_entry().await;
}