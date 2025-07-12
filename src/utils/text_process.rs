use std::vec;

/// 处理 ID 文本
pub fn process_id_text(input: String) -> String {
    let s = input.trim().to_lowercase();
    let mut result = String::new();
    for c in s.chars() {
        match c {
            '\n' | '_' => continue,
            '-' | '.' | ',' | ' ' => result.push('_'),
            _ => result.push(c),
        }
    }
    result.chars()
        .filter(|&c| c.is_ascii_alphanumeric() || c == '_')
        .collect()
}

pub fn process_id_text_not_to_lower(input: String) -> String {
    let s = input.trim();
    let mut result = String::new();
    for c in s.chars() {
        match c {
            '\n' | '_' => continue,
            '-' | '.' | ',' | ' ' => result.push('_'),
            _ => result.push(c),
        }
    }
    result.chars()
        .filter(|&c| c.is_ascii_alphanumeric() || c == '_')
        .collect()
}

/// 处理目录文本
pub fn process_path_text(path: String) -> String {
    let mut result = String::with_capacity(path.len());
    let mut chars = path.chars();

    // 处理可能的盘符
    if let (Some(first), Some(second)) = (chars.next(), chars.next()) {
        if first.is_ascii_alphabetic() && second == ':' {
            result.push(first);
            result.push(second);
        } else {
            // 创建 Boxed 迭代器组合
            return process_iterator(
                vec![first, second].into_iter().chain(chars),
                result
            );
        }
    }

    process_iterator(chars, result)
}

fn process_iterator<I: Iterator<Item = char>>(chars: I, mut result: String) -> String {
    for c in chars {
        match c {
            '\\' | '|' => result.push('/'),
            '/' => result.push('/'),
            '<' | '>' | '"' | '?' | '*' => {},
            ':' => {},
            _ if c.is_ascii_control() => result.push(' '),
            _ => result.push(c),
        }
    }

    // 清理结果
    let mut cleaned = result.trim().to_string();
    while cleaned.ends_with('/') {
        cleaned.pop();
    }
    cleaned
}