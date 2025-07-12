use std::fs::{copy, create_dir_all, remove_dir_all};
use std::path::PathBuf;
use serde::Deserialize;
use std::collections::HashMap;
use std::{env, fs};
use jam_ready::utils::open_in_explorer::open_in_explorer;

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
        root.join(".cargo").join("shared").join("target").join(branch).join("deps");

    // 清理目录
    let _ = remove_dir_all(&export_version_dir);

    for data in config.release.root {
        println!("Releasing \"{}\"", data.0);
        let raw_path = root.join(data.1.raw);
        let target_path = export_version_dir.join(data.1.target);
        copy_files(&raw_path, &target_path, data.1.files);
    }

    for data in config.release.deps {
        println!("Releasing \"{}\"", data.0);
        let raw_path = target_dir.join(data.1.raw);
        let target_path = export_version_dir.join(data.1.target);
        copy_files(&raw_path, &target_path, data.1.files);
    }

    for data in config.release.deploy_root {
        println!("Deploy \"{}\"", data.0);
        let raw_path = root.join(data.1.raw);
        let target_path = root.join(data.1.target);
        copy_files(&raw_path, &target_path, data.1.files);
    }

    for data in config.release.deploy_deps {
        println!("Deploy \"{}\"", data.0);
        let raw_path = target_dir.join(data.1.raw);
        let target_path = root.join(data.1.target);
        copy_files(&raw_path, &target_path, data.1.files);
    }

    if let Ok(()) = open_in_explorer(export_version_dir) {
        println!("DONE");
    }
}

fn copy_files (raw: &PathBuf, target: &PathBuf, file_names: Vec<String>) {
    create_dir_all(target).unwrap();

    for file in file_names.iter() {
        let file_path_raw = raw.join(file);
        let file_path_target = target.join(file);
        if file_path_raw.exists() {
            let _ = copy(&file_path_raw, &file_path_target);
        }
    }
}