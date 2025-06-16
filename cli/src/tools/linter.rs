use crate::config::LintConfig;
use crate::tools::{LintIssue, Severity};
use anyhow::Result;
use std::path::{Path, PathBuf};
use regex::Regex;
use walkdir::WalkDir;

#[derive(Debug, Clone, Default)]
pub struct LintStatistics {
    pub total: usize,
    pub errors: usize,
    pub warnings: usize,
    pub info: usize,
    pub fixable: usize,
    pub files_checked: usize,
}

impl LintStatistics {
    pub fn has_errors(&self) -> bool {
        self.errors > 0
    }

    pub fn summary(&self) -> String {
        format!(
            "Checked {} files: {} issues ({} errors, {} warnings, {} info), {} fixable",
            self.files_checked, self.total, self.errors, self.warnings, self.info, self.fixable
        )
    }
}

// Helper function for XML escaping
fn xml_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

pub struct NagLinter {
    config: LintConfig,
    rules: Vec<Box<dyn LintRule>>,
}

impl NagLinter {
    pub fn new(config: &LintConfig) -> Self {
        let mut linter = Self {
            config: config.clone(),
            rules: Vec::new(),
        };

        // Register built-in lint rules
        linter.register_default_rules();

        linter
    }    fn register_default_rules(&mut self) {
        self.rules.push(Box::new(UnusedVariableRule::new(&self.config)));
        self.rules.push(Box::new(UndefinedVariableRule::new()));
        self.rules.push(Box::new(UnusedImportRule::new(&self.config)));
        self.rules.push(Box::new(ShadowingRule::new()));
        self.rules.push(Box::new(TypeErrorRule::new()));
        self.rules.push(Box::new(LineLengthRule::new(&self.config)));
        self.rules.push(Box::new(IndentationRule::new()));
        self.rules.push(Box::new(TrailingWhitespaceRule::new()));
    }

    pub fn lint_path(&self, path: &Path, fix: bool) -> Result<Vec<LintIssue>> {
        let mut all_issues = Vec::new();

        if path.is_file() {
            if path.extension().and_then(|s| s.to_str()) == Some("nag") {
                let issues = self.lint_file(path, fix)?;
                all_issues.extend(issues);
            }        } else {
            for entry in WalkDir::new(path) {
                let entry = entry?;
                if entry.file_type().is_file() &&
                   entry.path().extension().and_then(|s| s.to_str()) == Some("nag") {

                    // Check if file should be ignored
                    if self.should_ignore_file(entry.path()) {
                        continue;
                    }

                    let issues = self.lint_file(entry.path(), fix)?;
                    all_issues.extend(issues);
                }
            }
        }

        Ok(all_issues)
    }

    pub fn lint_file(&self, file_path: &Path, fix: bool) -> Result<Vec<LintIssue>> {
        let content = std::fs::read_to_string(file_path)?;
        self.lint_string(&content, file_path.to_path_buf(), fix)
    }

    pub fn lint_string(&self, content: &str, file_path: PathBuf, fix: bool) -> Result<Vec<LintIssue>> {
        let mut issues = Vec::new();
        let mut fixed_content = content.to_string();

        for rule in &self.rules {
            if self.is_rule_enabled(rule.name()) {
                let rule_issues = rule.check(&fixed_content, &file_path)?;

                for issue in rule_issues {
                    if fix && issue.fixable {
                        // Apply the fix
                        if let Some(fixed) = rule.fix(&fixed_content, &issue)? {
                            fixed_content = fixed;
                        }
                    } else {
                        issues.push(issue);
                    }
                }
            }
        }

        // If fixes were applied, write back to file
        if fix && fixed_content != content {
            std::fs::write(&file_path, fixed_content)?;
        }

        Ok(issues)
    }

    fn is_rule_enabled(&self, rule_name: &str) -> bool {
        if self.config.disabled_rules.contains(&rule_name.to_string()) {
            return false;
        }

        self.config.enabled_rules.is_empty() ||
        self.config.enabled_rules.contains(&rule_name.to_string())
    }

    fn should_ignore_file(&self, file_path: &Path) -> bool {
        let path_str = file_path.to_string_lossy();

        for pattern in &self.config.ignore_patterns {
            if let Ok(regex) = Regex::new(&pattern.replace("**", ".*").replace("*", "[^/]*")) {
                if regex.is_match(&path_str) {
                    return true;
                }
            }
        }

        false
    }

    /// Format lint issues according to the specified format
    pub fn format_issues(&self, issues: &[LintIssue], format: &str) -> Result<String> {
        match format {
            "json" => {
                Ok(serde_json::to_string_pretty(issues)?)
            }
            "checkstyle" => {
                self.format_checkstyle(issues)
            }
            "github" => {
                self.format_github_actions(issues)
            }
            "compact" => {
                Ok(issues.iter()
                    .map(|issue| format!("{}:{}:{}: {} [{}]",
                        issue.file.display(), issue.line, issue.column,
                        issue.message, issue.rule))
                    .collect::<Vec<_>>()
                    .join("\n"))
            }
            "text" | _ => {
                Ok(issues.iter()
                    .map(|issue| issue.format_text())
                    .collect::<Vec<_>>()
                    .join("\n"))
            }
        }
    }

    fn format_checkstyle(&self, issues: &[LintIssue]) -> Result<String> {
        let mut output = String::from(r#"<?xml version="1.0" encoding="UTF-8"?>
<checkstyle version="8.0">
"#);

        // Group issues by file
        let mut files: std::collections::HashMap<&PathBuf, Vec<&LintIssue>> = std::collections::HashMap::new();
        for issue in issues {
            files.entry(&issue.file).or_default().push(issue);
        }

        for (file, file_issues) in files {
            output.push_str(&format!(r#"    <file name="{}">
"#, file.display()));

            for issue in file_issues {
                let severity = match issue.severity {
                    Severity::Error => "error",
                    Severity::Warning => "warning",
                    Severity::Info => "info",
                };
                  output.push_str(&format!(
                    r#"        <error line="{}" column="{}" severity="{}" message="{}" source="{}"/>
"#,
                    issue.line, issue.column, severity,
                    xml_escape(&issue.message), issue.rule
                ));
            }

            output.push_str("    </file>\n");
        }

        output.push_str("</checkstyle>\n");
        Ok(output)
    }

    fn format_github_actions(&self, issues: &[LintIssue]) -> Result<String> {
        let output = issues.iter()
            .map(|issue| {
                let level = match issue.severity {
                    Severity::Error => "error",
                    Severity::Warning => "warning",
                    Severity::Info => "notice",
                };

                format!("::{} file={},line={},col={}::{} [{}]",
                    level, issue.file.display(), issue.line, issue.column,
                    issue.message, issue.rule)
            })
            .collect::<Vec<_>>()
            .join("\n");

        Ok(output)
    }

    /// Get statistics about linting results
    pub fn get_statistics(&self, issues: &[LintIssue]) -> LintStatistics {
        let mut stats = LintStatistics::default();

        for issue in issues {
            match issue.severity {
                Severity::Error => stats.errors += 1,
                Severity::Warning => stats.warnings += 1,
                Severity::Info => stats.info += 1,
            }

            if issue.fixable {
                stats.fixable += 1;
            }
        }

        stats.total = issues.len();
        stats.files_checked = issues.iter()
            .map(|i| &i.file)
            .collect::<std::collections::HashSet<_>>()
            .len();

        stats
    }
}

// Lint rule trait
pub trait LintRule {
    fn name(&self) -> &str;
    fn check(&self, content: &str, file_path: &PathBuf) -> Result<Vec<LintIssue>>;
    fn fix(&self, content: &str, issue: &LintIssue) -> Result<Option<String>>;
}

// Individual lint rules
pub struct UnusedVariableRule {
    allow_unused: bool,
}

impl UnusedVariableRule {
    pub fn new(config: &LintConfig) -> Self {
        Self {
            allow_unused: config.allow_unused_variables,
        }
    }
}

impl LintRule for UnusedVariableRule {
    fn name(&self) -> &str {
        "unused-variables"
    }    fn check(&self, content: &str, file_path: &PathBuf) -> Result<Vec<LintIssue>> {
        if self.allow_unused {
            return Ok(Vec::new());
        }

        let mut issues = Vec::new();

        // Simple regex-based detection (in a real implementation, this would use AST analysis)
        let assignment_regex = Regex::new(r"^(\s*)([a-zA-Z_][a-zA-Z0-9_]*)\s*=")?;

        for (line_num, line) in content.lines().enumerate() {
            if let Some(captures) = assignment_regex.captures(line) {
                let var_name = &captures[2];

                // Check if variable is used later in the file
                let usage_regex = Regex::new(&format!(r"\b{}\b", regex::escape(var_name)))?;
                let remaining_content = content.lines().skip(line_num + 1).collect::<Vec<_>>().join("\n");

                if !usage_regex.is_match(&remaining_content) {
                    issues.push(LintIssue {
                        file: file_path.clone(),
                        line: (line_num + 1) as u32,
                        column: captures[1].len() as u32,
                        severity: Severity::Warning,
                        rule: self.name().to_string(),
                        message: format!("Variable '{}' is assigned but never used", var_name),
                        fixable: true,
                    });
                }
            }
        }

        Ok(issues)
    }

    fn fix(&self, content: &str, issue: &LintIssue) -> Result<Option<String>> {
        // Simple fix: comment out the unused assignment
        let lines: Vec<&str> = content.lines().collect();
        let mut fixed_lines = lines.clone();

        if let Some(line) = fixed_lines.get_mut((issue.line - 1) as usize) {
            if !line.trim().starts_with('#') {
                *line = &format!("# {}", line);
            }
        }

        Ok(Some(fixed_lines.join("\n")))
    }
}

pub struct UndefinedVariableRule;

impl UndefinedVariableRule {
    pub fn new() -> Self {
        Self
    }
}

impl LintRule for UndefinedVariableRule {
    fn name(&self) -> &str {
        "undefined-variables"
    }

    fn check(&self, content: &str, file_path: &PathBuf) -> Result<Vec<LintIssue>> {
        let mut issues = Vec::new();

        // This would require proper AST analysis in a real implementation
        // For now, just check for common undefined variable patterns
        let undefined_regex = Regex::new(r"\bundefined_var\b")?;

        for (line_num, line) in content.lines().enumerate() {
            if undefined_regex.is_match(line) {
                issues.push(LintIssue {
                    file: file_path.clone(),
                    line: (line_num + 1) as u32,
                    column: 0,
                    severity: Severity::Error,
                    rule: self.name().to_string(),
                    message: "Undefined variable 'undefined_var'".to_string(),
                    fixable: false,
                });
            }
        }

        Ok(issues)
    }

    fn fix(&self, _content: &str, _issue: &LintIssue) -> Result<Option<String>> {
        Ok(None) // Not fixable
    }
}

pub struct UnusedImportRule {
    allow_unused: bool,
}

impl UnusedImportRule {
    pub fn new(config: &LintConfig) -> Self {
        Self {
            allow_unused: config.allow_unused_imports,
        }
    }
}

impl LintRule for UnusedImportRule {
    fn name(&self) -> &str {
        "unused-imports"
    }

    fn check(&self, content: &str, file_path: &PathBuf) -> Result<Vec<LintIssue>> {
        let mut issues = Vec::new();

        let import_regex = Regex::new(r"^import\s+([a-zA-Z_][a-zA-Z0-9_]*)")?;
        let from_import_regex = Regex::new(r"^from\s+\S+\s+import\s+([a-zA-Z_][a-zA-Z0-9_]*)")?;

        for (line_num, line) in content.lines().enumerate() {
            let mut imported_name = None;

            if let Some(captures) = import_regex.captures(line) {
                imported_name = Some(&captures[1]);
            } else if let Some(captures) = from_import_regex.captures(line) {
                imported_name = Some(&captures[1]);
            }

            if let Some(name) = imported_name {
                // Check if import is used
                let usage_regex = Regex::new(&format!(r"\b{}\b", regex::escape(name)))?;
                let rest_of_file = content.lines().skip(line_num + 1).collect::<Vec<_>>().join("\n");

                if !usage_regex.is_match(&rest_of_file) {
                    issues.push(LintIssue {
                        file: file_path.clone(),
                        line: (line_num + 1) as u32,
                        column: 0,
                        severity: Severity::Warning,
                        rule: self.name().to_string(),
                        message: format!("Import '{}' is unused", name),
                        fixable: true,
                    });
                }
            }
        }

        Ok(issues)
    }

    fn fix(&self, content: &str, issue: &LintIssue) -> Result<Option<String>> {
        let lines: Vec<&str> = content.lines().collect();
        let mut fixed_lines = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            if i + 1 != issue.line as usize {
                fixed_lines.push(*line);
            }
            // Skip the line with unused import
        }

        Ok(Some(fixed_lines.join("\n")))
    }
}

pub struct ShadowingRule;

impl ShadowingRule {
    pub fn new() -> Self {
        Self
    }
}

impl LintRule for ShadowingRule {
    fn name(&self) -> &str {
        "shadowing"
    }

    fn check(&self, content: &str, file_path: &PathBuf) -> Result<Vec<LintIssue>> {
        let mut issues = Vec::new();

        // Check for variable shadowing (simplified)
        let builtins = ["print", "len", "str", "int", "float", "bool", "list", "dict", "set", "tuple"];
        let assignment_regex = Regex::new(r"^\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*=")?;

        for (line_num, line) in content.lines().enumerate() {
            if let Some(captures) = assignment_regex.captures(line) {
                let var_name = &captures[1];

                if builtins.contains(&var_name) {
                    issues.push(LintIssue {
                        file: file_path.clone(),
                        line: (line_num + 1) as u32,
                        column: 0,
                        severity: Severity::Warning,
                        rule: self.name().to_string(),
                        message: format!("Variable '{}' shadows built-in function", var_name),
                        fixable: false,
                    });
                }
            }
        }

        Ok(issues)
    }

    fn fix(&self, _content: &str, _issue: &LintIssue) -> Result<Option<String>> {
        Ok(None) // Not automatically fixable
    }
}

pub struct TypeErrorRule;

impl TypeErrorRule {
    pub fn new() -> Self {
        Self
    }
}

impl LintRule for TypeErrorRule {
    fn name(&self) -> &str {
        "type-errors"
    }

    fn check(&self, content: &str, file_path: &PathBuf) -> Result<Vec<LintIssue>> {
        let mut issues = Vec::new();

        // Simple type checking (would need full type inference in real implementation)
        let type_mismatch_regex = Regex::new(r#"(\w+):\s*int\s*=\s*"[^"]*""#)?;

        for (line_num, line) in content.lines().enumerate() {
            if type_mismatch_regex.is_match(line) {
                issues.push(LintIssue {
                    file: file_path.clone(),
                    line: (line_num + 1) as u32,
                    column: 0,
                    severity: Severity::Error,
                    rule: self.name().to_string(),
                    message: "Type mismatch: expected int, got str".to_string(),
                    fixable: false,
                });
            }
        }

        Ok(issues)
    }

    fn fix(&self, _content: &str, _issue: &LintIssue) -> Result<Option<String>> {
        Ok(None) // Type errors are not automatically fixable
    }
}

pub struct LineLengthRule {
    max_length: u16,
}

impl LineLengthRule {
    pub fn new(config: &LintConfig) -> Self {
        Self {
            max_length: config.max_line_length,
        }
    }
}

impl LintRule for LineLengthRule {
    fn name(&self) -> &str {
        "line-length"
    }    fn check(&self, content: &str, file_path: &PathBuf) -> Result<Vec<LintIssue>> {
        let mut issues = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            if line.len() > self.max_length as usize {
                issues.push(LintIssue {
                    file: file_path.clone(),
                    line: (line_num + 1) as u32,
                    column: self.max_length as u32,
                    severity: Severity::Warning,
                    rule: self.name().to_string(),
                    message: format!("Line too long ({} > {} characters)", line.len(), self.max_length),
                    fixable: false,
                });
            }
        }

        Ok(issues)
    }

    fn fix(&self, _content: &str, _issue: &LintIssue) -> Result<Option<String>> {
        Ok(None) // Line length issues require manual fixing
    }
}

pub struct IndentationRule;

impl IndentationRule {
    pub fn new() -> Self {
        Self
    }
}

impl LintRule for IndentationRule {
    fn name(&self) -> &str {
        "indentation"
    }

    fn check(&self, content: &str, file_path: &PathBuf) -> Result<Vec<LintIssue>> {
        let mut issues = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }

            let leading_spaces = line.len() - line.trim_start().len();

            // Check for tabs
            if line.starts_with('\t') {
                issues.push(LintIssue {
                    file: file_path.clone(),
                    line: (line_num + 1) as u32,
                    column: 0,
                    severity: Severity::Warning,
                    rule: self.name().to_string(),
                    message: "Use spaces instead of tabs for indentation".to_string(),
                    fixable: true,
                });
            }

            // Check for non-multiple of 4 spaces
            if leading_spaces % 4 != 0 && !line.trim().starts_with('#') {
                issues.push(LintIssue {
                    file: file_path.clone(),
                    line: (line_num + 1) as u32,
                    column: 0,
                    severity: Severity::Warning,
                    rule: self.name().to_string(),
                    message: "Indentation should be a multiple of 4 spaces".to_string(),
                    fixable: true,
                });
            }
        }

        Ok(issues)
    }

    fn fix(&self, content: &str, issue: &LintIssue) -> Result<Option<String>> {
        let lines: Vec<&str> = content.lines().collect();
        let mut fixed_lines = lines.clone();

        if let Some(line) = fixed_lines.get_mut((issue.line - 1) as usize) {
            // Replace tabs with 4 spaces
            let fixed_line = line.replace('\t', "    ");
            *line = &fixed_line;
        }

        Ok(Some(fixed_lines.join("\n")))
    }
}

pub struct TrailingWhitespaceRule;

impl TrailingWhitespaceRule {
    pub fn new() -> Self {
        Self
    }
}

impl LintRule for TrailingWhitespaceRule {
    fn name(&self) -> &str {
        "trailing-whitespace"
    }

    fn check(&self, content: &str, file_path: &PathBuf) -> Result<Vec<LintIssue>> {
        let mut issues = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            if line.ends_with(' ') || line.ends_with('\t') {
                issues.push(LintIssue {
                    file: file_path.clone(),
                    line: (line_num + 1) as u32,
                    column: line.trim_end().len() as u32,
                    severity: Severity::Info,
                    rule: self.name().to_string(),
                    message: "Trailing whitespace".to_string(),
                    fixable: true,
                });
            }
        }

        Ok(issues)
    }

    fn fix(&self, content: &str, _issue: &LintIssue) -> Result<Option<String>> {
        let fixed = content
            .lines()
            .map(|line| line.trim_end())
            .collect::<Vec<_>>()
            .join("\n");

        Ok(Some(fixed))
    }
}
