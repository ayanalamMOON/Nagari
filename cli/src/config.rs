use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NagConfig {
    pub project: ProjectConfig,
    pub build: BuildConfig,
    pub lsp: LspConfig,
    pub format: FormatConfig,
    pub lint: LintConfig,
    pub test: TestConfig,
    pub package: PackageConfig,
    pub verbose: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub license: Option<String>,
    pub repository: Option<String>,
    pub main: Option<String>,
    pub source_dir: String,
    pub output_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    pub target: String,
    pub optimization: bool,
    pub sourcemap: bool,
    pub minify: bool,
    pub jsx: bool,
    pub declarations: bool,
    pub treeshake: bool,
    pub external: Vec<String>,
    pub define: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspConfig {
    pub enabled: bool,
    pub diagnostics: bool,
    pub completion: bool,
    pub hover: bool,
    pub goto_definition: bool,
    pub find_references: bool,
    pub rename: bool,
    pub code_actions: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatConfig {
    pub indent_size: u8,
    pub max_line_length: u16,
    pub use_tabs: bool,
    pub trailing_commas: bool,
    pub quote_style: String, // "single", "double", "prefer_single"
    pub space_around_operators: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintConfig {
    pub enabled_rules: Vec<String>,
    pub disabled_rules: Vec<String>,
    pub rule_severity: HashMap<String, String>, // rule_name -> "error" | "warn" | "info"
    pub ignore_patterns: Vec<String>,
    pub max_line_length: u16,
    pub max_complexity: u8,
    pub allow_unused_variables: bool,
    pub allow_unused_imports: bool,
    pub strict_typing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    pub test_pattern: String,
    pub coverage: bool,
    pub timeout: u64,
    pub parallel: bool,
    pub max_workers: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageConfig {
    pub registry: String,
    pub cache_dir: String,
    pub lockfile: String,
    pub auto_install: bool,
}

impl Default for NagConfig {
    fn default() -> Self {
        Self {
            project: ProjectConfig {
                name: "nagari-project".to_string(),
                version: "0.1.0".to_string(),
                description: None,
                author: None,
                license: Some("MIT".to_string()),
                repository: None,
                main: Some("main.nag".to_string()),
                source_dir: "src".to_string(),
                output_dir: "dist".to_string(),
            },            build: BuildConfig {
                target: "js".to_string(),
                optimization: false,
                sourcemap: true,
                minify: false,
                jsx: false,
                declarations: false,
                treeshake: true,
                external: vec![],
                define: HashMap::new(),
            },
            lsp: LspConfig {
                enabled: true,
                diagnostics: true,
                completion: true,
                hover: true,
                goto_definition: true,
                find_references: true,
                rename: true,
                code_actions: true,
            },
            format: FormatConfig {
                indent_size: 4,
                max_line_length: 88,
                use_tabs: false,
                trailing_commas: true,
                quote_style: "double".to_string(),
                space_around_operators: true,
            },
            lint: LintConfig {                enabled_rules: vec![
                    "unused-variables".to_string(),
                    "undefined-variables".to_string(),
                    "unused-imports".to_string(),
                    "shadowing".to_string(),
                    "type-errors".to_string(),
                    "line-length".to_string(),
                    "indentation".to_string(),
                    "trailing-whitespace".to_string(),
                ],
                disabled_rules: vec![],
                rule_severity: HashMap::new(),
                ignore_patterns: vec!["node_modules/**".to_string(), "dist/**".to_string()],
                max_line_length: 88,
                max_complexity: 10,
                allow_unused_variables: false,
                allow_unused_imports: false,
                strict_typing: true,
            },
            test: TestConfig {
                test_pattern: "**/*_test.nag".to_string(),
                coverage: false,
                timeout: 30000,
                parallel: true,
                max_workers: None,
            },            package: PackageConfig {
                registry: "https://registry.nagari-lang.org".to_string(),
                cache_dir: "~/.nag/cache".to_string(),
                lockfile: "nag.lock".to_string(),
                auto_install: true,
            },
            verbose: false,
        }
    }
}

impl NagConfig {
    pub fn load(config_path: Option<&Path>) -> Result<Self> {
        let config_file = match config_path {
            Some(path) => path.to_owned(),
            None => {
                // Look for config files in order of preference
                let candidates = [
                    "nag.toml",
                    "nagari.toml",
                    ".nagari.toml",
                    "nag.json",
                    "nagari.json",
                    ".nagari.json",
                ];

                let mut found = None;
                for candidate in &candidates {
                    let path = PathBuf::from(candidate);
                    if path.exists() {
                        found = Some(path);
                        break;
                    }
                }

                match found {
                    Some(path) => path,
                    None => return Ok(Self::default()),
                }
            }
        };

        let content = std::fs::read_to_string(&config_file)?;

        let config = if config_file.extension().and_then(|s| s.to_str()) == Some("json") {
            serde_json::from_str(&content)?
        } else {
            toml::from_str(&content)?
        };

        Ok(config)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let content = if path.extension().and_then(|s| s.to_str()) == Some("json") {
            serde_json::to_string_pretty(self)?
        } else {
            toml::to_string_pretty(self)?
        };

        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn merge_with_defaults(mut self) -> Self {
        let defaults = Self::default();

        // Apply default values for missing fields
        if self.project.source_dir.is_empty() {
            self.project.source_dir = defaults.project.source_dir;
        }
        if self.project.output_dir.is_empty() {
            self.project.output_dir = defaults.project.output_dir;
        }

        self
    }
}
