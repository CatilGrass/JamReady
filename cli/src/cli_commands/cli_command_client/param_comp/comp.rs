use crate::cli_commands::cli_command_client::param_comp::data::{CompConfig, CompContext, CompError};
use crate::data::database::{Database, VirtualFile};
use crate::data::parameters::read_parameter;
use jam_ready::utils::text_process::process_path_text;
use regex::Regex;

/// 编译 FROM 参数
pub fn comp_param_from(config: &CompConfig, mut context: CompContext) -> Result<CompContext, CompError> {
    context = comp_alias_param_tag(context)?;
    context = comp_short_path_tag(&config, context)?;
    context = comp_multi_file_regex_tag(&config, context)?;
    // context = comp_normalize(context)?;

    Ok(comp_final(context))
}

/// 编译 TO 参数
pub fn comp_param_to(config: &CompConfig, mut context: CompContext) -> Result<CompContext, CompError> {
    context = comp_context_path_tag(context)?;
    context = comp_extract_multi_results(config, context)?;
    // context = comp_normalize(context)?;
    Ok(comp_final(context))
}

/// 空结果时
pub fn comp_final(context: CompContext) -> CompContext {
    if context.final_paths.len() <= 0 {
        CompContext {
            ctx: get_path(&context.input.clone()),
            final_paths: vec![context.input.clone()],
            ..context
        }
    } else {
        context
    }
}

/// 标准化输入
#[allow(dead_code)]
pub fn comp_normalize (mut context: CompContext) -> Result<CompContext, CompError> {

    // 修改输入内容
    context.input = normalize_path(&context.input.clone())?;

    // 修改输出内容
    let mut output = Vec::new();
    for final_path in context.final_paths {
        output.push(normalize_path(&final_path)?)
    }
    context.final_paths = output;

    Ok(context)
}

/// 编译 上下文目录 标识
pub fn comp_context_path_tag(mut context: CompContext) -> Result<CompContext, CompError> {
    let raw = context.clone();
    if context.input.starts_with("./") {
        let path = context.input.clone().trim_start_matches("./").to_string();
        let full = format!("{}{}", context.ctx, path);

        // 修改输入
        context.input = full.clone();

        // 修改上下文
        context.ctx = get_path(&context.input.clone());

        return Ok(context)
    }
    Ok(raw)
}

pub fn comp_extract_multi_results(config: &CompConfig, mut context: CompContext) -> Result<CompContext, CompError> {
    let raw = context.clone();

    // 判断是否为目录
    if !context.input.ends_with("/") {

        // 清空结果
        context.final_paths.clear();
        return Ok(context)
    }

    // 判断是否支持多结果
    if !config.allow_multi_path { return Ok(raw) }

    let mut output = Vec::new();
    for final_path in context.final_paths.clone() {

        // 从上下文中剥离出原始地址的相对地址
        let relative_path = final_path.strip_prefix(get_path(&context.ctx).as_str());
        if let Some(relative_path) = relative_path {

            // 将相对地址附加到当前地址
            let current = format!("{}{}", context.input, relative_path);
            output.push(current);
        }
    }
    context.final_paths = output;

    // 修改上下文
    context.ctx = get_path(&context.input);

    Ok(context)
}

/// 编译 别称参数 标识
pub fn comp_alias_param_tag(mut context: CompContext) -> Result<CompContext, CompError> {
    let raw = context.clone();
    if context.input.ends_with('?') {
        let param = context.input.clone().trim_end_matches("?").to_string();
        if let Some(content) = read_parameter(param.clone()) {

            // 修改输入
            context.input = content;
            return Ok(context)
        }
        return Err(CompError::err(format!("Parameter \"{}\" not found", param).as_str()));
    }
    Ok(raw)
}

/// 编译 简短路径 标识
pub fn comp_short_path_tag(config: &CompConfig, mut context: CompContext) -> Result<CompContext, CompError> {
    let raw = context.clone();
    if context.input.starts_with(':') {
        let full = config.local_folder_map.short_file_map.get(&context.input.trim_start_matches(":").to_string());
        if let Some(full) = full {

            // 设置输出目录
            let mut output: Vec<String> = Vec::new();
            output.push(full.clone());
            context.final_paths = output;

            // 修改输入内容
            context.input = full.clone();

            // 修改上下文
            context.ctx = get_path(&full.clone());

            // 导出结果
            return Ok(context)
        }
        return Err(CompError::err("Incorrect short path."));
    }
    Ok(raw)
}

/// 编译 多文件标识符
pub fn comp_multi_file_regex_tag(config: &CompConfig, mut context: CompContext) -> Result<CompContext, CompError> {

    let raw = context.clone();
    let split = context.input.split('/');
    let Some(regex_str) = split.clone().last() else {
        return Ok(raw)
    };

    // 若存在 * 则视为正则表达式，否则跳过
    if !regex_str.contains("*") { return Ok(raw); }
    let Ok(regex) = Regex::new(regex_str) else {
        return Err(CompError::err(format!("Failed to parse the regular expression \"{}\".", regex_str).as_str()));
    };

    // 设置上下文
    context.ctx = get_path(&context.input.clone());

    // 若未启用多目录则结束编译
    if ! config.allow_multi_path { Err(CompError::err("Multiple paths not allowed."))?; }

    // 清空结果
    context.final_paths = Vec::new();

    // 将上下文作为搜索内容，搜索相似的文件
    for virtual_file in get_files_in_dir(&context.ctx, &config.database) {
        let path = virtual_file.path().clone();
        let name = path.split("/").last();
        if let Some(name) = name {
            if regex.is_match(name) {
                context.final_paths.push(path);
            }
        }
    }

    // 导出结果
    Ok(context)
}

fn get_path(path: &str) -> String {
    if let Some(last_slash) = path.rfind('/') {
        path[..=last_slash].to_string()
    } else {
        "".to_string()
    }
}

pub fn normalize_path(path: &str) -> Result<String, CompError> {
    if path.is_empty() {
        return Err(CompError::err("input path must not be empty"));
    }

    let mut stack = Vec::new();
    let parts: Vec<&str> = path.split('/').collect();

    for part in parts {
        match part.trim() {
            ".." => {
                if stack.pop().is_none() {}
            }
            "." | "" => continue,
            _ => {
                stack.push(part);
            }
        }
    }

    let normalized = stack.join("/");

    if normalized.is_empty() {
        Err(CompError::err("resulting path is empty after normalization"))
    } else {
        Ok(normalized)
    }
}

// 从目录获得文件列表
pub fn get_files_in_dir<'a>(dir: &'a str, database: &'a Database) -> Vec<&'a VirtualFile> {
    let dir = process_path_text(dir.to_string());
    if dir.trim().is_empty() {
        return database.files();
    }
    let mut result = Vec::new();
    for file in database.files() {
        if file.path().starts_with(format!("{}/", dir).as_str()) {
            result.push(file);
        }
    }
    result
}