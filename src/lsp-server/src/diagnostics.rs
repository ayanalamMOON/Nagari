use anyhow::Result;
use dashmap::DashMap;
use nagari_compiler::Compiler;
use nagari_parser::{Lexer, Parser};
use std::sync::Arc;
use tower_lsp::lsp_types::*;

pub struct DiagnosticsProvider {
    // Cache for diagnostics per document
    diagnostics_cache: Arc<DashMap<Url, Vec<Diagnostic>>>,
    compiler: Compiler,
}

impl DiagnosticsProvider {
    pub fn new() -> Self {
        Self {
            diagnostics_cache: Arc::new(DashMap::new()),
            compiler: Compiler::new(),
        }
    }

    pub async fn get_diagnostics(&self, uri: &Url, text: &str) -> Result<Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();

        // 1. Lexical analysis - check for tokenization errors
        if let Err(parse_errors) = self.analyze_syntax(text) {
            for error in parse_errors {
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position {
                            line: error.line.saturating_sub(1) as u32,
                            character: error.column.saturating_sub(1) as u32,
                        },
                        end: Position {
                            line: error.line.saturating_sub(1) as u32,
                            character: (error.column + error.length.unwrap_or(1)).saturating_sub(1)
                                as u32,
                        },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: Some(NumberOrString::String(format!("SYNTAX_{}", error.code))),
                    source: Some("nagari".to_string()),
                    message: error.message,
                    related_information: None,
                    tags: None,
                    code_description: None,
                    data: None,
                });
            }
        }

        // 2. Semantic analysis - check for semantic errors
        match self.analyze_semantics(text) {
            Ok(semantic_issues) => {
                for issue in semantic_issues {
                    diagnostics.push(issue);
                }
            }
            Err(_) => {
                // If semantic analysis fails, add a general warning
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position {
                            line: 0,
                            character: 0,
                        },
                        end: Position {
                            line: 0,
                            character: 1,
                        },
                    },
                    severity: Some(DiagnosticSeverity::WARNING),
                    code: Some(NumberOrString::String(
                        "SEMANTIC_ANALYSIS_FAILED".to_string(),
                    )),
                    source: Some("nagari".to_string()),
                    message: "Semantic analysis could not be completed".to_string(),
                    related_information: None,
                    tags: None,
                    code_description: None,
                    data: None,
                });
            }
        }

        // 3. Style and lint checks
        let lint_diagnostics = self.analyze_style(text);
        diagnostics.extend(lint_diagnostics);

        // Cache the diagnostics
        self.diagnostics_cache
            .insert(uri.clone(), diagnostics.clone());

        Ok(diagnostics)
    }

    pub async fn clear_diagnostics(&self, uri: &Url) -> Result<()> {
        self.diagnostics_cache.remove(uri);
        Ok(())
    }

    pub fn get_cached_diagnostics(&self, uri: &Url) -> Option<Vec<Diagnostic>> {
        self.diagnostics_cache.get(uri).map(|diags| diags.clone())
    }

    fn analyze_syntax(&self, text: &str) -> Result<(), Vec<SyntaxError>> {
        let mut lexer = Lexer::new(text);
        let mut errors = Vec::new();

        // Tokenize the entire input and collect lexing errors
        let tokens = match lexer.tokenize() {
            Ok(tokens) => tokens,
            Err(parse_error) => {
                errors.push(SyntaxError {
                    line: 1, // Lexer errors need better position tracking
                    column: 1,
                    length: Some(1),
                    code: "LEXER_ERROR".to_string(),
                    message: format!("Lexical error: {}", parse_error),
                });
                return Err(errors);
            }
        };

        // Parse the tokens and collect parsing errors
        let mut parser = Parser::new(tokens);
        match parser.parse() {
            Ok(_) => {
                if errors.is_empty() {
                    Ok(())
                } else {
                    Err(errors)
                }
            }
            Err(parse_error) => {
                errors.push(SyntaxError {
                    line: 1, // Parser errors need better position tracking
                    column: 1,
                    length: Some(1),
                    code: "PARSER_ERROR".to_string(),
                    message: format!("Parse error: {}", parse_error),
                });
                Err(errors)
            }
        }
    }

    fn analyze_semantics(&self, text: &str) -> Result<Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();

        // Use the compiler to check for semantic issues
        match self.compiler.compile_string(text, None) {
            Ok(_) => {
                // Compilation successful, check for warnings
                diagnostics.extend(self.check_unused_variables(text));
                diagnostics.extend(self.check_undefined_variables(text));
                diagnostics.extend(self.check_type_mismatches(text));
            }
            Err(compiler_error) => {
                // Compilation failed, create diagnostic
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position {
                            line: 0,
                            character: 0,
                        },
                        end: Position {
                            line: 0,
                            character: 1,
                        },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: Some(NumberOrString::String("COMPILER_ERROR".to_string())),
                    source: Some("nagari".to_string()),
                    message: format!("Compilation error: {}", compiler_error),
                    related_information: None,
                    tags: None,
                    code_description: None,
                    data: None,
                });
            }
        }

        Ok(diagnostics)
    }

    fn analyze_style(&self, text: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let lines: Vec<&str> = text.lines().collect();

        for (line_idx, line) in lines.iter().enumerate() {
            // Check line length
            if line.len() > 100 {
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position {
                            line: line_idx as u32,
                            character: 100,
                        },
                        end: Position {
                            line: line_idx as u32,
                            character: line.len() as u32,
                        },
                    },
                    severity: Some(DiagnosticSeverity::WARNING),
                    code: Some(NumberOrString::String("LINE_TOO_LONG".to_string())),
                    source: Some("nagari-lint".to_string()),
                    message: "Line exceeds maximum length of 100 characters".to_string(),
                    related_information: None,
                    tags: None,
                    code_description: None,
                    data: None,
                });
            }

            // Check trailing whitespace
            if line.ends_with(' ') || line.ends_with('\t') {
                let trimmed_len = line.trim_end().len();
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position {
                            line: line_idx as u32,
                            character: trimmed_len as u32,
                        },
                        end: Position {
                            line: line_idx as u32,
                            character: line.len() as u32,
                        },
                    },
                    severity: Some(DiagnosticSeverity::INFORMATION),
                    code: Some(NumberOrString::String("TRAILING_WHITESPACE".to_string())),
                    source: Some("nagari-lint".to_string()),
                    message: "Trailing whitespace".to_string(),
                    related_information: None,
                    tags: Some(vec![DiagnosticTag::UNNECESSARY]),
                    code_description: None,
                    data: None,
                });
            }

            // Check indentation consistency
            if line.starts_with('\t') && line.contains("    ") {
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position {
                            line: line_idx as u32,
                            character: 0,
                        },
                        end: Position {
                            line: line_idx as u32,
                            character: line.len().min(10) as u32,
                        },
                    },
                    severity: Some(DiagnosticSeverity::WARNING),
                    code: Some(NumberOrString::String("MIXED_INDENTATION".to_string())),
                    source: Some("nagari-lint".to_string()),
                    message: "Mixed tabs and spaces for indentation".to_string(),
                    related_information: None,
                    tags: None,
                    code_description: None,
                    data: None,
                });
            }
        }

        diagnostics
    }

    fn check_unused_variables(&self, text: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        // Simple regex-based check for demonstration
        // In a real implementation, this would use AST analysis

        for (line_idx, line) in text.lines().enumerate() {
            if let Some(captures) = regex::Regex::new(r"let\s+(\w+)\s*=")
                .unwrap()
                .captures(line)
            {
                let var_name = &captures[1];
                // Check if variable is used elsewhere in the text
                if !text.contains(&format!("{}", var_name)) || text.matches(var_name).count() <= 1 {
                    let start_pos = line.find(var_name).unwrap_or(0);
                    diagnostics.push(Diagnostic {
                        range: Range {
                            start: Position {
                                line: line_idx as u32,
                                character: start_pos as u32,
                            },
                            end: Position {
                                line: line_idx as u32,
                                character: (start_pos + var_name.len()) as u32,
                            },
                        },
                        severity: Some(DiagnosticSeverity::WARNING),
                        code: Some(NumberOrString::String("UNUSED_VARIABLE".to_string())),
                        source: Some("nagari".to_string()),
                        message: format!("Variable '{}' is declared but never used", var_name),
                        related_information: None,
                        tags: Some(vec![DiagnosticTag::UNNECESSARY]),
                        code_description: None,
                        data: None,
                    });
                }
            }
        }

        diagnostics
    }

    fn check_undefined_variables(&self, text: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        // This would need proper symbol table analysis in a real implementation

        for (line_idx, line) in text.lines().enumerate() {
            // Simple check for common undefined variable patterns
            if line.contains("console.") && !text.contains("import") && !text.contains("console") {
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position {
                            line: line_idx as u32,
                            character: 0,
                        },
                        end: Position {
                            line: line_idx as u32,
                            character: line.len() as u32,
                        },
                    },
                    severity: Some(DiagnosticSeverity::WARNING),
                    code: Some(NumberOrString::String("UNDEFINED_VARIABLE".to_string())),
                    source: Some("nagari".to_string()),
                    message:
                        "'console' is not defined. Consider importing it or using an alternative"
                            .to_string(),
                    related_information: None,
                    tags: None,
                    code_description: None,
                    data: None,
                });
            }
        }

        diagnostics
    }

    fn check_type_mismatches(&self, _text: &str) -> Vec<Diagnostic> {
        let diagnostics = Vec::new();
        // Type checking would require full type inference system
        // This is a placeholder for where type checking diagnostics would go

        diagnostics
    }
}

#[derive(Debug, Clone)]
struct SyntaxError {
    line: usize,
    column: usize,
    length: Option<usize>,
    code: String,
    message: String,
}
