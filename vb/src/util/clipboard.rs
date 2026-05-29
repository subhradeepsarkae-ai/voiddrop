use anyhow::{bail, Result};
use colored::Colorize;
use std::path::{Path, PathBuf};

pub struct ClipboardFile {
    pub path: PathBuf,
    pub filename: String,
    pub size: u64,
}

pub fn read_clipboard_file() -> Result<ClipboardFile> {
    let mut cb = arboard::Clipboard::new()
        .map_err(|_| anyhow::anyhow!("Could not access clipboard"))?;

    if let Ok(files) = cb.get().file_list() {
        for path in files {
            if path.is_file() {
                return file_from_path(&path);
            }
        }
    }

    if let Ok(text) = cb.get_text() {
        let text = text.trim().to_string();
        if !text.is_empty() {
            let path = PathBuf::from(&text);
            if path.is_file() {
                return file_from_path(&path);
            }
        }
    }

    bail!("No valid file found in clipboard")
}

fn file_from_path(path: &Path) -> Result<ClipboardFile> {
    let metadata = std::fs::metadata(path)?;
    let filename = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    Ok(ClipboardFile {
        path: path.to_path_buf(),
        filename,
        size: metadata.len(),
    })
}

pub fn print_clipboard_detected(file: &ClipboardFile) {
    println!(
        "  {} {}",
        "📋 Clipboard detected:".cyan().bold(),
        file.filename.yellow()
    );
    println!(
        "  {}  {}",
        " ".repeat(2),
        crate::util::helpers::format_size(file.size)
    );
    println!();
    println!("  {} {}", "⚡".to_string(), "Starting transfer...".green().bold());
    println!();
}
