#![allow(dead_code)]

use std::path::{Path, PathBuf};
use anyhow::Result;
use serde::{Deserialize, Serialize};

pub mod formatter;
pub mod linter;
pub mod doc_generator;
pub mod package_manager;

pub use formatter::NagFormatter;
pub use linter::NagLinter;
pub use doc_generator::DocGenerator;

#[derive(Debug, Clone)]
pub struct FileChange {
    pub path: PathBuf,
    pub changed: bool,
    pub diff: Option<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintIssue {
    pub file: PathBuf,
    pub line: u32,
    pub column: u32,
    pub severity: Severity,
    pub rule: String,
    pub message: String,
    pub fixable: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

impl LintIssue {
    pub fn format_text(&self) -> String {
        let severity_icon = match self.severity {
            Severity::Error => "❌",
            Severity::Warning => "⚠️",
            Severity::Info => "ℹ️",
        };

        format!(
            "{} {}:{}:{} [{}] {}",
            severity_icon,
            self.file.display(),
            self.line,
            self.column,
            self.rule,
            self.message
        )
    }
}

/// Tools utilities that use the imported components
pub struct ToolsManager {
    formatter: NagFormatter,
    linter: NagLinter,
    doc_generator: DocGenerator,
}

impl ToolsManager {    pub fn new() -> Result<Self> {
        // Create default configs
        let nag_config = crate::config::NagConfig::default();

        Ok(Self {
            formatter: NagFormatter::new(&nag_config.format),
            linter: NagLinter::new(&nag_config.lint),
            doc_generator: DocGenerator::new(&nag_config),
        })
    }    pub fn format_files(&self, files: &[PathBuf]) -> Result<Vec<FileChange>> {
        let mut changes = Vec::new();
        for file in files {
            match self.formatter.format_file(file, false, false) {
                Ok(change) => changes.push(change),
                Err(e) => {
                    changes.push(FileChange {
                        path: file.clone(),
                        changed: false,
                        diff: None,
                        errors: vec![e.to_string()],
                    });
                }
            }
        }
        Ok(changes)
    }
      pub fn lint_files(&self, files: &[PathBuf]) -> Result<Vec<FileChange>> {
        let mut changes = Vec::new();
        for file in files {
            match self.linter.lint_file(file, false) {
                Ok(issues) => {
                    let error_messages: Vec<String> = issues.into_iter()
                        .map(|issue| format!("Line {}: {}", issue.line, issue.message))
                        .collect();
                    changes.push(FileChange {
                        path: file.clone(),
                        changed: false,
                        diff: None,
                        errors: error_messages,
                    });
                }
                Err(e) => {
                    changes.push(FileChange {
                        path: file.clone(),
                        changed: false,
                        diff: None,
                        errors: vec![e.to_string()],
                    });
                }
            }
        }
        Ok(changes)
    }

    pub fn generate_docs(&self, source_dir: &Path, output_dir: &Path) -> Result<Vec<FileChange>> {
        match self.doc_generator.generate(source_dir, output_dir, "html", false) {
            Ok(_) => Ok(vec![FileChange {
                path: output_dir.to_path_buf(),
                changed: true,
                diff: None,
                errors: Vec::new(),            }]),
            Err(e) => Ok(vec![FileChange {
                path: output_dir.to_path_buf(),
                changed: false,
                diff: None,
                errors: vec![e.to_string()],
            }])
        }
    }
}
