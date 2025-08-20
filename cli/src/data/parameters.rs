use std::env::current_dir;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use jam_ready::utils::text_process::process_id_text;

/// Get parameter file path
pub fn parameter_file(parameter: String) -> PathBuf {
    let parameter = format!("{}.txt", process_id_text(parameter));
    current_dir()
        .unwrap()
        .join(env!("PATH_PARAMETERS"))
        .join(parameter)
}

/// Write parameter to file
pub fn write_parameter(parameter: String, content: String) {
    let file = parameter_file(parameter);
    File::create(file)
        .unwrap()
        .write_all(content.as_bytes())
        .unwrap();
}

/// Delete parameter file
pub fn erase_parameter(parameter: String) {
    let file = parameter_file(parameter);
    if file.exists() {
        let _ = fs::remove_file(file);
    }
}

/// Read parameter from file
pub fn read_parameter(parameter: String) -> Option<String> {
    let file = parameter_file(parameter);
    if file.exists() {
        fs::read_to_string(file)
            .map(|content| content.trim().to_string())
            .ok()
    } else {
        None
    }
}

/// List all available parameters
pub fn parameters() -> Vec<String> {
    let mut result = Vec::new();
    let dir_path = current_dir().unwrap().join(env!("PATH_PARAMETERS"));

    if let Ok(dir) = fs::read_dir(dir_path) {
        for entry in dir.flatten() {
            if let Some(file_name) = entry.file_name().to_str() {
                if let Some(stripped) = file_name.strip_suffix(".txt") {
                    result.push(stripped.to_string());
                }
            }
        }
    }
    result
}