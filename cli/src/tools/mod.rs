use std::path::PathBuf;
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
