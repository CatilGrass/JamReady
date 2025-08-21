use std::collections::BTreeMap;
use std::vec;
use colored::Colorize;
use regex::Regex;

/// Process text
pub fn process_text(input: String) -> String {
    let s = input.trim();
    let mut result = String::new();
    for c in s.chars() {
        match c {
            '\n' => result.push_str("\\n"),
            '\t' => result.push_str("\\t"),
            '\r' => result.push_str("\\r"),
            '\"' => result.push_str("\\\""),
            _ => result.push(c),
        }
    }
    result.chars().collect()
}

/// Process ID text
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

/// Process path text
pub fn process_path_text(path: String) -> String {
    let mut result = String::with_capacity(path.len());
    let mut chars = path.chars();

    if let (Some(first), Some(second)) = (chars.next(), chars.next()) {
        if first.is_ascii_alphabetic() && second == ':' {
            result.push(first);
            result.push(second);
        } else {
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

    let mut cleaned = result.trim().to_string();
    while cleaned.ends_with('/') {
        cleaned.pop();
    }

    cleaned = cleaned.trim_start_matches("./").to_string();

    cleaned
}

/// Split path text
pub fn split_path_text(path: &str) -> (String, String) {
    if path.is_empty() {
        return ("".to_string(), "".to_string());
    }

    let normalized_path = path.trim_start_matches('/');

    if let Some(last_idx) = normalized_path.rfind('/') {
        let dir_part = &normalized_path[..=last_idx];
        let file_part = &normalized_path[last_idx + 1..];
        (dir_part.to_string(), file_part.to_string())
    } else {
        ("".to_string(), normalized_path.to_string())
    }
}

/// Parse colored text
pub fn parse_colored_text(text: &str) -> String {
    let re = Regex::new(r"\[(red|green|blue|yellow|magenta|cyan|white|gray)](.*?)\[/]").unwrap();

    let mut result = String::new();
    let mut last_pos = 0;

    for cap in re.captures_iter(text) {
        result.push_str(&text[last_pos..cap.get(0).unwrap().start()]);
        last_pos = cap.get(0).unwrap().end();

        let color = cap.get(1).unwrap().as_str();
        let content = cap.get(2).unwrap().as_str();

        let colored_content = match color {
            "red" => content.red(),
            "green" => content.green(),
            "blue" => content.blue(),
            "yellow" => content.yellow(),
            "magenta" => content.magenta(),
            "cyan" => content.cyan(),
            "white" => content.white(),
            "gray" => content.truecolor(128, 128, 128),
            _ => content.normal(),
        };

        result.push_str(&colored_content.to_string());
    }
    result.push_str(&text[last_pos..]);
    result
}

/// Display file tree
pub fn show_tree(paths: Vec<String>) -> String {
    #[derive(Default)]
    struct Node {
        is_file: bool,
        children: BTreeMap<String, Node>,
    }

    let mut root = Node::default();

    for path in paths {
        let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        let mut current = &mut root;

        for (i, part) in parts.iter().enumerate() {
            let is_file = i == parts.len() - 1;
            let child = current.children.entry((*part).to_string())
                .or_insert_with(Node::default);

            if is_file {
                child.is_file = true;
            }
            current = child;
        }
    }

    // Generate tree structure text
    fn generate_tree_lines(children: &BTreeMap<String, Node>, prefix: &str) -> Vec<String> {
        // Group children: directories first, then files, each sorted by name
        let mut dirs = Vec::new();
        let mut files = Vec::new();

        for (name, node) in children {
            if node.children.is_empty() {
                files.push((name, node));
            } else {
                dirs.push((name, node));
            }
        }

        // Sort by name
        dirs.sort_by_key(|(name, _)| *name);
        files.sort_by_key(|(name, _)| *name);
        let child_nodes = dirs.into_iter().chain(files.into_iter()).collect::<Vec<_>>();

        let mut lines = Vec::new();
        let last_index = child_nodes.len().saturating_sub(1);
        let child_prefix = format!("{}│   ", prefix);

        for (index, (name, node)) in child_nodes.into_iter().enumerate() {
            let is_last = index == last_index;
            let connector = if is_last { "└── " } else { "├── " };

            let name = name.replace("\\s", "/");

            if !node.children.is_empty() {
                lines.push(format!("{}{}{}/", prefix, connector, name));
                let child_lines = generate_tree_lines(&node.children, &child_prefix);
                lines.extend(child_lines);
            } else {
                lines.push(format!("{}{}{}", prefix, connector, name));
            }
        }

        lines
    }

    generate_tree_lines(&root.children, "").join("\n")
}