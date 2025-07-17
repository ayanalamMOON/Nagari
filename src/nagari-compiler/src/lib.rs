//! Nagari Programming Language Compiler
//!
//! This library provides the core compilation functionality for the Nagari programming language,
//! including lexical analysis, parsing, type checking, and transpilation to JavaScript.

pub mod lexer;
pub mod parser;
pub mod ast;
pub mod types;
pub mod error;
pub mod transpiler;

use std::path::Path;
use std::fs;

pub use error::NagariError;
pub use ast::Program;
pub use lexer::Lexer;
pub use parser::Parser as NagParser;

/// Main compiler interface for the Nagari programming language
#[derive(Debug, Clone)]
pub struct Compiler {
    pub config: CompilerConfig,
}

/// Configuration options for the Nagari compiler
#[derive(Debug, Clone)]
pub struct CompilerConfig {
    /// Target JavaScript format (es6, node, esm, cjs)
    pub target: String,
    /// Enable JSX support for React compatibility
    pub jsx: bool,
    /// Generate source maps for debugging
    pub sourcemap: bool,
    /// Enable development mode with debug info
    pub devtools: bool,
    /// Minify output (production mode)
    pub minify: bool,
    /// Generate TypeScript declarations
    pub declarations: bool,
    /// Enable verbose output
    pub verbose: bool,
}

impl Default for CompilerConfig {
    fn default() -> Self {
        Self {
            target: "es6".to_string(),
            jsx: false,
            sourcemap: false,
            devtools: false,
            minify: false,
            declarations: false,
            verbose: false,
        }
    }
}

/// Result of a compilation operation
#[derive(Debug, Clone)]
pub struct CompilationResult {
    /// Generated JavaScript code
    pub js_code: String,
    /// Source map content (if enabled)
    pub source_map: Option<String>,
    /// TypeScript declarations (if enabled)
    pub declarations: Option<String>,
    /// AST of the compiled program
    pub ast: Program,
    /// List of warnings generated during compilation
    pub warnings: Vec<String>,
}

impl Compiler {
    /// Create a new compiler instance with default configuration
    pub fn new() -> Self {
        Self {
            config: CompilerConfig::default(),
        }
    }

    /// Create a new compiler instance with custom configuration
    pub fn with_config(config: CompilerConfig) -> Self {
        Self { config }
    }

    /// Compile a Nagari source string to JavaScript
    pub fn compile_string(&self, source: &str, filename: Option<&str>) -> Result<CompilationResult, NagariError> {
        if self.config.verbose {
            println!("🔄 Compiling Nagari source...");
        }

        // Lexical analysis
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize()
            .map_err(|e| NagariError::LexError(format!("Lexing failed: {}", e)))?;

        if self.config.verbose {
            println!("✅ Lexical analysis completed ({} tokens)", tokens.len());
        }

        // Parsing
        let mut parser = NagParser::new(tokens);
        let ast = parser.parse()
            .map_err(|e| NagariError::ParseError(format!("Parsing failed: {}", e)))?;

        if self.config.verbose {
            println!("✅ Parsing completed");
        }

        // Transpilation
        let js_code = transpiler::transpile(&ast, &self.config.target, self.config.jsx)?;

        if self.config.verbose {
            println!("✅ Transpilation completed");
        }

        // Generate source map if enabled
        let source_map = if self.config.sourcemap {
            Some(self.generate_source_map(filename.unwrap_or("input.nag"), source)?)
        } else {
            None
        };

        // Generate TypeScript declarations if enabled
        let declarations = if self.config.declarations {
            Some(self.generate_declarations(&ast)?)
        } else {
            None
        };

        Ok(CompilationResult {
            js_code,
            source_map,
            declarations,
            ast,
            warnings: Vec::new(),
        })
    }

    /// Compile a Nagari file to JavaScript
    pub fn compile_file<P: AsRef<Path>>(&self, input_path: P) -> Result<CompilationResult, NagariError> {
        let input_path = input_path.as_ref();

        if self.config.verbose {
            println!("📁 Reading file: {}", input_path.display());
        }

        let source = fs::read_to_string(input_path)
            .map_err(|e| NagariError::IoError(format!("Failed to read input file: {}", e)))?;

        let filename = input_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("input.nag");

        self.compile_string(&source, Some(filename))
    }

    /// Transpile a Nagari file directly to JavaScript and write to output file
    pub fn transpile_file<P: AsRef<Path>>(&self, input_path: P) -> Result<String, NagariError> {
        let result = self.compile_file(input_path)?;
        Ok(result.js_code)
    }

    /// Check syntax of a Nagari file without generating output
    pub fn check_syntax<P: AsRef<Path>>(&self, input_path: P) -> Result<Program, NagariError> {
        let input_path = input_path.as_ref();

        if self.config.verbose {
            println!("🔍 Checking syntax: {}", input_path.display());
        }

        let source = fs::read_to_string(input_path)
            .map_err(|e| NagariError::IoError(format!("Failed to read input file: {}", e)))?;

        // Lexical analysis
        let mut lexer = Lexer::new(&source);
        let tokens = lexer.tokenize()
            .map_err(|e| NagariError::LexError(format!("Lexing failed: {}", e)))?;

        // Parsing
        let mut parser = NagParser::new(tokens);
        let ast = parser.parse()
            .map_err(|e| NagariError::ParseError(format!("Parsing failed: {}", e)))?;

        if self.config.verbose {
            println!("✅ Syntax check passed");
        }

        Ok(ast)
    }

    /// Compile and write result to output file
    pub fn compile_to_file<P: AsRef<Path>, Q: AsRef<Path>>(
        &self,
        input_path: P,
        output_path: Q,
    ) -> Result<(), NagariError> {
        let output_path = output_path.as_ref();
        let result = self.compile_file(input_path)?;

        // Create output directory if needed
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| NagariError::IoError(format!("Failed to create output directory: {}", e)))?;
        }

        // Add source map comment if enabled
        let final_code = if self.config.sourcemap && result.source_map.is_some() {
            format!("{}\n//# sourceMappingURL={}.map",
                result.js_code,
                output_path.file_name().unwrap().to_str().unwrap()
            )
        } else {
            result.js_code
        };

        // Write JavaScript output
        fs::write(output_path, final_code)
            .map_err(|e| NagariError::IoError(format!("Failed to write output file: {}", e)))?;

        // Write source map if enabled
        if let Some(source_map) = result.source_map {
            let map_path = output_path.with_extension("js.map");
            fs::write(&map_path, source_map)
                .map_err(|e| NagariError::IoError(format!("Failed to write source map: {}", e)))?;
        }

        // Write TypeScript declarations if enabled
        if let Some(declarations) = result.declarations {
            let dts_path = output_path.with_extension("d.ts");
            fs::write(&dts_path, declarations)
                .map_err(|e| NagariError::IoError(format!("Failed to write declarations: {}", e)))?;
        }

        if self.config.verbose {
            println!("✅ Compiled successfully to: {}", output_path.display());
        }

        Ok(())
    }

    /// Generate a source map for the given source code
    fn generate_source_map(&self, filename: &str, source_content: &str) -> Result<String, NagariError> {
        let sourcemap = serde_json::json!({
            "version": 3,
            "file": filename.replace(".nag", ".js"),
            "sources": [filename],
            "sourcesContent": [source_content],
            "mappings": "AAAA" // Basic mapping - can be enhanced later
        });

        Ok(sourcemap.to_string())
    }

    /// Generate TypeScript declarations for the given AST
    fn generate_declarations(&self, _ast: &Program) -> Result<String, NagariError> {
        // Basic TypeScript declaration generation
        // This can be enhanced to extract actual type information from the AST
        Ok("// Generated TypeScript declarations\nexport {};\n".to_string())
    }

    /// Update compiler configuration
    pub fn set_config(&mut self, config: CompilerConfig) {
        self.config = config;
    }

    /// Get current compiler configuration
    pub fn get_config(&self) -> &CompilerConfig {
        &self.config
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder pattern for creating compiler configurations
pub struct CompilerConfigBuilder {
    config: CompilerConfig,
}

impl CompilerConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: CompilerConfig::default(),
        }
    }

    pub fn target(mut self, target: &str) -> Self {
        self.config.target = target.to_string();
        self
    }

    pub fn jsx(mut self, jsx: bool) -> Self {
        self.config.jsx = jsx;
        self
    }

    pub fn sourcemap(mut self, sourcemap: bool) -> Self {
        self.config.sourcemap = sourcemap;
        self
    }

    pub fn devtools(mut self, devtools: bool) -> Self {
        self.config.devtools = devtools;
        self
    }

    pub fn minify(mut self, minify: bool) -> Self {
        self.config.minify = minify;
        self
    }

    pub fn declarations(mut self, declarations: bool) -> Self {
        self.config.declarations = declarations;
        self
    }

    pub fn verbose(mut self, verbose: bool) -> Self {
        self.config.verbose = verbose;
        self
    }

    pub fn build(self) -> CompilerConfig {
        self.config
    }
}

impl Default for CompilerConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compiler_creation() {
        let compiler = Compiler::new();
        assert_eq!(compiler.config.target, "es6");
        assert!(!compiler.config.jsx);
    }

    #[test]
    fn test_compiler_config_builder() {
        let config = CompilerConfigBuilder::new()
            .target("esm")
            .jsx(true)
            .sourcemap(true)
            .verbose(true)
            .build();

        assert_eq!(config.target, "esm");
        assert!(config.jsx);
        assert!(config.sourcemap);
        assert!(config.verbose);
    }

    #[test]
    fn test_compile_string_basic() {
        let compiler = Compiler::new();
        let source = r#"
def greet(name: str) -> str:
    return f"Hello, {name}!"

print(greet("World"))
"#;

        // This test would require the actual lexer/parser implementation
        // For now, we'll just test that the API exists
        let _result = compiler.compile_string(source, Some("test.nag"));
        // Test should pass once the lexer/parser are fully implemented
    }
}
