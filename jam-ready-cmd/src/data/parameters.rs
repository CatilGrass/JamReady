use std::env::current_dir;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use jam_ready::utils::text_process::process_id_text;

/// 获得参数文件地址
pub fn parameter_file(parameter: String) -> PathBuf {
    let parameter = format!("{}.txt", process_id_text(parameter));
    let file = current_dir().unwrap().join(env!("PATH_PARAMETERS")).join(parameter.clone());
    file
}

/// 写入参数
pub fn write_parameter(parameter: String, content: String) {
    let file = parameter_file(parameter.clone());
    File::create(file).unwrap().write_all(content.as_bytes()).unwrap();
}

/// 擦除参数
pub fn erase_parameter(parameter: String) {
    let file = parameter_file(parameter.clone());
    if file.exists() {
        let _ = fs::remove_file(file);
    }
}

/// 读取参数
pub fn read_parameter(parameter: String) -> Option<String> {
    let file = parameter_file(parameter);
    if file.exists() {
        let read = fs::read_to_string(file);
        match read {
            Ok(content) => Some(content.trim().to_string()),
            Err(_) => None
        }
    } else {
        None
    }
}

/// 列出所有参数
pub fn parameters() -> Vec<String> {
    let mut result = Vec::new();
    let dir = fs::read_dir(current_dir().unwrap().join(env!("PATH_PARAMETERS")));
    if let Ok(dir) = dir {
        for d in dir {
            if let Ok(d) = d {
                if let Some(p) = d.path().to_str() {
                    result.push(p.replace(".txt", ""));
                }
            }
        }
    }
    result
}