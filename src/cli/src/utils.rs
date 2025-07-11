#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use anyhow::{Result, Context};

/// Check if a command exists in PATH
pub fn command_exists(command: &str) -> bool {
    Command::new("which")
        .arg(command)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

/// Get the project root directory by looking for nagari.toml or nagari.json
pub fn find_project_root(start_dir: &Path) -> Option<PathBuf> {
    let mut current = start_dir;

    loop {
        if current.join("nagari.toml").exists() || current.join("nagari.json").exists() {
            return Some(current.to_path_buf());
        }

        match current.parent() {
            Some(parent) => current = parent,
            None => break,
        }
    }

    None
}

/// Ensure a directory exists, creating it if necessary
pub fn ensure_dir(path: &Path) -> Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)
            .with_context(|| format!("Failed to create directory: {}", path.display()))?;
    }
    Ok(())
}

/// Check if a file has a specific extension
pub fn has_extension(path: &Path, ext: &str) -> bool {
    path.extension()
        .and_then(|s| s.to_str())
        .map(|s| s.eq_ignore_ascii_case(ext))
        .unwrap_or(false)
}

/// Get a relative path from one directory to another
pub fn relative_path(from: &Path, to: &Path) -> PathBuf {
    let from = from.canonicalize().unwrap_or_else(|_| from.to_path_buf());
    let to = to.canonicalize().unwrap_or_else(|_| to.to_path_buf());

    pathdiff::diff_paths(&to, &from).unwrap_or(to)
}

/// Recursively find all files with a specific extension
pub fn find_files_with_extension(dir: &Path, ext: &str) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for entry in walkdir::WalkDir::new(dir) {
        let entry = entry?;
        if entry.file_type().is_file() && has_extension(entry.path(), ext) {
            files.push(entry.path().to_path_buf());
        }
    }

    Ok(files)
}

/// Format bytes as human-readable size
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Check if the current directory is a Nagari project
pub fn is_nagari_project(dir: &Path) -> bool {
    dir.join("nagari.toml").exists() || dir.join("nagari.json").exists()
}

/// Get the Nagari cache directory
pub fn get_cache_dir() -> Result<PathBuf> {
    let cache_dir = dirs::cache_dir()
        .context("Failed to get cache directory")?
        .join("nagari");

    ensure_dir(&cache_dir)?;
    Ok(cache_dir)
}

/// Get the Nagari config directory
pub fn get_config_dir() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .context("Failed to get config directory")?
        .join("nagari");

    ensure_dir(&config_dir)?;
    Ok(config_dir)
}

/// Get the current working directory
pub fn current_dir() -> Result<PathBuf> {
    std::env::current_dir()
        .context("Failed to get current directory")
}

/// Check if running on Windows
pub fn is_windows() -> bool {
    cfg!(target_os = "windows")
}

/// Get the executable extension for the current platform
pub fn exe_extension() -> &'static str {
    if is_windows() {
        ".exe"
    } else {
        ""
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_has_extension() {
        assert!(has_extension(Path::new("test.nag"), "nag"));
        assert!(has_extension(Path::new("test.NAG"), "nag"));
        assert!(!has_extension(Path::new("test.js"), "nag"));
        assert!(!has_extension(Path::new("test"), "nag"));
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1536), "1.5 KB");
        assert_eq!(format_bytes(1048576), "1.0 MB");
    }

    #[test]
    fn test_find_project_root() {
        let temp_dir = tempdir().unwrap();
        let project_dir = temp_dir.path().join("project");
        let nested_dir = project_dir.join("src").join("deep");

        std::fs::create_dir_all(&nested_dir).unwrap();
        std::fs::write(project_dir.join("nagari.toml"), "[project]\nname = \"test\"").unwrap();

        let found = find_project_root(&nested_dir);
        assert_eq!(found, Some(project_dir));
    }
}
