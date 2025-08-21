use std::{
    fs::{self, File},
    io::Write,
    path::Path,
    collections::HashMap,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    generate_help_docs()
}

fn generate_help_docs() -> Result<(), Box<dyn std::error::Error>> {
    let source_dir = Path::new("../docs/cli_help");
    let output_path = Path::new("src/help/help_docs.rs");

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut output = File::create(output_path)?;
    writeln!(output, "// Auto-generated")?;
    writeln!(output, "use jam_ready::utils::text_process::parse_colored_text;")?;

    let mut help_map = HashMap::new();

    process_dir(&source_dir, source_dir, &mut output, &mut help_map)?;

    writeln!(output, "\npub fn get_help_docs(name: &str) -> String {{")?;
    writeln!(output, "    parse_colored_text(match name {{")?;

    for (name, const_name) in &help_map {
        writeln!(output, "        \"{name}\" => {const_name},")?;
    }

    writeln!(output, "        _ => \"\",")?;
    writeln!(output, "    }}.trim())")?;
    writeln!(output, "}}")?;

    Ok(())
}

fn process_dir(
    base_path: &Path,
    current_dir: &Path,
    output: &mut File,
    help_map: &mut HashMap<String, String>,
) -> Result<(), Box<dyn std::error::Error>> {
    for entry in fs::read_dir(current_dir)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = path.metadata()?;

        if metadata.is_dir() {
            process_dir(base_path, &path, output, help_map)?;
        } else if metadata.is_file() {
            process_file(base_path, &path, output, help_map)?;
        }
    }
    Ok(())
}

fn process_file(
    base_path: &Path,
    file_path: &Path,
    output: &mut File,
    help_map: &mut HashMap<String, String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let rel_path = file_path.strip_prefix(base_path.parent().unwrap())?
        .to_string_lossy()
        .replace("\\", "/");

    let const_name = rel_path
        .trim_start_matches('/')
        .trim_end_matches(".txt")
        .replace('/', "_")
        .replace('.', "_")
        .trim_start_matches("cli_help_")
        .to_uppercase();

    let help_name = const_name.to_lowercase();

    let content = fs::read_to_string(file_path)?;

    writeln!(output, "\n/// From ./{}", rel_path)?;
    writeln!(
        output,
        "pub const {}: &'static str = \"\n{}\";",
        const_name, content
    )?;

    help_map.insert(help_name, const_name);

    Ok(())
}