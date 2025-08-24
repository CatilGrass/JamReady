use std::env::current_dir;
use jam_ready::utils::local_archive::LocalArchive;
use crate::data::workspace::Workspace;

pub async fn jam_linker_entry() {
    let workspace = Workspace::read().await;
    let Some(workspace) = workspace.client else {
        eprintln!("It's not a client workspace.");
        return;
    };
    println!("Workspace >>> {} ({})", current_dir().unwrap().display(), workspace.workspace_name);
}