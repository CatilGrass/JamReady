use colored::Colorize;
use jam_ready::utils::open_in_explorer::open_in_explorer;
use jam_ready::utils::text_process::process_path_text;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::{copy, create_dir_all, read_dir, remove_dir_all};
use std::path::{Path, PathBuf};
use std::{env, fs, io};
use tokio::time::Instant;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub release: Release,
}

#[derive(Debug, Deserialize)]
pub struct Release {
    pub root: HashMap<String, ReleaseItem>,
    pub deps: HashMap<String, ReleaseItem>,
    pub deploy_root: HashMap<String, ReleaseItem>,
    pub deploy_deps: HashMap<String, ReleaseItem>
}

#[derive(Debug, Deserialize)]
pub struct ReleaseItem {
    pub raw: String,
    pub target: String,
    pub files: Vec<String>,
}

pub fn main() {

    let mut count = 0;
    let start_time = Instant::now();

    let args: Vec<String> = env::args().collect();
    let version = if let Some(v) = args.get(1) { v.to_string() } else { env!("PROJECT_VERSION").to_string() };
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let toml_config = root.join("./.cargo/export.toml");
    let config: Config = toml::from_str(fs::read_to_string(toml_config).unwrap().as_str())
        .expect("Failed to parse TOML");

    let export_root =
        root.join(".cargo").join("shared").join("export");

    let export_version_dir =
        export_root.join(&version);

    let branch = if version.eq("dev") { "debug" } else { "release" };

    let target_dir =
        root.join(".cargo").join("shared").join("target").join(branch);

    let _ = remove_dir_all(&export_version_dir);

    for data in config.release.root {
        println!("{} `{}`", "   Releasing".green().bold(), data.0);
        let raw_path = root.join(data.1.raw);
        let target_path = export_version_dir.join(data.1.target);
        copy_files(&raw_path, &target_path, data.1.files);
        count += 1;
    }

    for data in config.release.deps {
        println!("{} `{}`", "   Releasing".green().bold(), data.0);
        let raw_path = target_dir.join(data.1.raw);
        let target_path = export_version_dir.join(data.1.target);
        copy_files(&raw_path, &target_path, data.1.files);
        count += 1;
    }

    for data in config.release.deploy_root {
        println!("{} `{}`", "      Deploy".green().bold(), data.0);
        let raw_path = root.join(data.1.raw);
        let target_path = root.join(data.1.target);
        copy_files(&raw_path, &target_path, data.1.files);
        count += 1;
    }

    for data in config.release.deploy_deps {
        println!("{} `{}`", "      Deploy".green().bold(), data.0);
        let raw_path = target_dir.join(data.1.raw);
        let target_path = root.join(data.1.target);
        copy_files(&raw_path, &target_path, data.1.files);
        count += 1;
    }

    let elapsed = start_time.elapsed();

    if let Ok(()) = open_in_explorer(export_version_dir.clone()) {
        println!("{} {}", "    Finished".green().bold(),
                 format!("released {} profile(s) in {:.2}s", count, elapsed.as_secs_f64()));
        if let Some(path_text) = export_version_dir.to_str() {
            println!("{} {:?}", "      Output".green().bold(),
                     process_path_text(path_text.to_string()).trim_matches('"'));
        }
    }
}

fn copy_files(raw: &PathBuf, target: &PathBuf, file_names: Vec<String>) {
    create_dir_all(target).unwrap();

    for file in file_names {
        let source_path = raw.join(&file);

        if source_path.is_file() {
            // Copy file
            let destination_path = target.join(&file);
            if let Err(e) = copy_with_parents(&source_path, &destination_path) {
                eprintln!("Error copying file: {:?} -> {:?}: {}", source_path, destination_path, e);
            }
        } else if source_path.is_dir() {
            // Copy folder
            if let Err(e) = copy_dir_contents(&source_path, target) {
                eprintln!("Error copying directory contents: {:?} -> {:?}: {}", source_path, target, e);
            }
        } else {
            // eprintln!("Warning: Path is neither file nor directory: {:?}", source_path);
        }
    }
}

fn copy_with_parents(src: &PathBuf, dst: &PathBuf) -> io::Result<()> {
    if let Some(parent) = dst.parent() {
        create_dir_all(parent)?;
    }
    copy(src, dst)?;
    Ok(())
}

fn copy_dir_contents(src_dir: &Path, dst_dir: &Path) -> io::Result<()> {
    create_dir_all(dst_dir)?;

    for entry in read_dir(src_dir)? {
        let entry = entry?;
        let source_path = entry.path();
        let entry_name = entry.file_name();
        let destination_path = dst_dir.join(&entry_name);

        if source_path.is_dir() {
            copy_dir_recursive(&source_path, &destination_path)?;
        } else {
            copy(&source_path, &destination_path)?;
        }
    }
    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> io::Result<()> {
    create_dir_all(dst)?;

    for entry in read_dir(src)? {
        let entry = entry?;
        let entry_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if entry_path.is_dir() {
            copy_dir_recursive(&entry_path, &dst_path)?;
        } else {
            copy(&entry_path, &dst_path)?;
        }
    }
    Ok(())
}