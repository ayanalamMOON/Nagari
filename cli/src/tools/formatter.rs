use crate::config::FormatConfig;
use crate::tools::{FileChange, Severity};
use anyhow::Result;
use std::path::Path;

pub struct NagFormatter {
    config: FormatConfig,
}

impl NagFormatter {
    pub fn new(config: &FormatConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }

    pub fn format_file(
        &self,
        file_path: &Path,
        check_only: bool,
        show_diff: bool,
    ) -> Result<FileChange> {
        let content = std::fs::read_to_string(file_path)?;
        let formatted = self.format_string(&content)?;

        let changed = content != formatted;
        let diff = if show_diff && changed {
            Some(self.generate_diff(&content, &formatted))
        } else {
            None
        };

        if !check_only && changed {
            std::fs::write(file_path, &formatted)?;
        }

        Ok(FileChange {
            path: file_path.to_path_buf(),
            changed,
            diff,
            errors: vec![],
        })
    }

    pub fn format_string(&self, content: &str) -> Result<String> {
        let mut formatted = String::new();
        let mut indent_level = 0;
        let mut in_string = false;
        let mut string_char = '\0';
        let mut escape_next = false;

        for line in content.lines() {
            let trimmed = line.trim();

            // Skip empty lines
            if trimmed.is_empty() {
                formatted.push('\n');
                continue;
            }

            // Handle indentation
            let mut new_line = String::new();

            // Check if this line should decrease indentation
            if trimmed.starts_with("except") ||
               trimmed.starts_with("elif") ||
               trimmed.starts_with("else") ||
               trimmed.starts_with("finally") {
                indent_level = indent_level.saturating_sub(1);
            }

            // Add indentation
            let indent = if self.config.use_tabs {
                "\t".repeat(indent_level)
            } else {
                " ".repeat(indent_level * self.config.indent_size as usize)
            };

            new_line.push_str(&indent);

            // Format the line content
            let formatted_content = self.format_line_content(trimmed)?;
            new_line.push_str(&formatted_content);

            // Check if this line should increase indentation for next line
            if trimmed.ends_with(':') &&
               !trimmed.starts_with('#') &&
               (trimmed.starts_with("def ") ||
                trimmed.starts_with("class ") ||
                trimmed.starts_with("if ") ||
                trimmed.starts_with("elif ") ||
                trimmed.starts_with("else") ||
                trimmed.starts_with("for ") ||
                trimmed.starts_with("while ") ||
                trimmed.starts_with("try") ||
                trimmed.starts_with("except") ||
                trimmed.starts_with("finally") ||
                trimmed.starts_with("with ") ||
                trimmed.starts_with("async ")) {
                indent_level += 1;
            }

            // Enforce line length
            if new_line.len() > self.config.max_line_length as usize {
                // TODO: Implement line wrapping
            }

            formatted.push_str(&new_line);
            formatted.push('\n');
        }

        // Remove trailing whitespace and normalize line endings
        let lines: Vec<&str> = formatted.lines().collect();
        let mut result = String::new();

        for (i, line) in lines.iter().enumerate() {
            result.push_str(line.trim_end());
            if i < lines.len() - 1 {
                result.push('\n');
            }
        }

        Ok(result)
    }

    fn format_line_content(&self, line: &str) -> Result<String> {
        let mut result = String::new();
        let mut chars = line.chars().peekable();
        let mut in_string = false;
        let mut string_delimiter = '\0';

        while let Some(ch) = chars.next() {
            match ch {
                '"' | '\'' if !in_string => {
                    in_string = true;
                    string_delimiter = ch;
                    result.push(ch);
                }
                ch if ch == string_delimiter && in_string => {
                    in_string = false;
                    result.push(ch);
                }
                ' ' if !in_string => {
                    // Handle spacing around operators
                    if self.config.space_around_operators {
                        if let Some(&next_ch) = chars.peek() {
                            match next_ch {
                                '=' | '+' | '-' | '*' | '/' | '%' | '<' | '>' | '!' => {
                                    if !result.ends_with(' ') {
                                        result.push(' ');
                                    }
                                }
                                _ => result.push(' '),
                            }
                        } else {
                            result.push(' ');
                        }
                    } else {
                        result.push(' ');
                    }
                }
                '=' | '+' | '-' | '*' | '/' | '%' | '<' | '>' | '!' if !in_string => {
                    if self.config.space_around_operators {
                        if !result.ends_with(' ') {
                            result.push(' ');
                        }
                        result.push(ch);
                        if chars.peek().is_some() && chars.peek() != Some(&' ') {
                            result.push(' ');
                        }
                    } else {
                        result.push(ch);
                    }
                }
                ',' if !in_string => {
                    result.push(ch);
                    if self.config.trailing_commas {
                        if chars.peek().is_some() && chars.peek() != Some(&' ') {
                            result.push(' ');
                        }
                    }
                }
                _ => result.push(ch),
            }
        }

        Ok(result)
    }

    fn generate_diff(&self, original: &str, formatted: &str) -> String {
        let mut diff = String::new();
        let original_lines: Vec<&str> = original.lines().collect();
        let formatted_lines: Vec<&str> = formatted.lines().collect();

        diff.push_str("--- original\n");
        diff.push_str("+++ formatted\n");

        let max_lines = std::cmp::max(original_lines.len(), formatted_lines.len());

        for i in 0..max_lines {
            let orig_line = original_lines.get(i).unwrap_or(&"");
            let fmt_line = formatted_lines.get(i).unwrap_or(&"");

            if orig_line != fmt_line {
                if !orig_line.is_empty() {
                    diff.push_str(&format!("-{}\n", orig_line));
                }
                if !fmt_line.is_empty() {
                    diff.push_str(&format!("+{}\n", fmt_line));
                }
            }
        }

        diff
    }
}
