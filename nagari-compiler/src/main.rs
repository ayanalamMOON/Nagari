#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use clap::Parser;
use std::fs;
use std::path::Path;
use std::process::Command;

mod lexer;
mod parser;
mod ast;
mod types;
mod error;
mod transpiler;

use lexer::Lexer;
use parser::Parser as NagParser;
use error::NagariError;

#[derive(Parser)]
#[command(name = "nagc")]
#[command(about = "Nagari compiler - transpiles .nag files to JavaScript")]
#[command(version = "0.1.0")]
struct Cli {
    /// Input file (.nag)
    input: String,

    /// Output file (.js) - optional
    #[arg(short, long)]
    output: Option<String>,

    /// Target JavaScript format
    #[arg(long, default_value = "es6", value_parser = ["es6", "node", "esm", "cjs"])]
    target: String,

    /// Enable JSX support for React compatibility
    #[arg(long)]
    jsx: bool,

    /// Bundle output with dependencies
    #[arg(long)]
    bundle: bool,

    /// Generate source maps for debugging
    #[arg(long)]
    sourcemap: bool,

    /// Enable development mode with debug info
    #[arg(long)]
    devtools: bool,

    /// Minify output (production mode)
    #[arg(long)]
    minify: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Watch mode for development
    #[arg(short, long)]
    watch: bool,

    /// Check syntax only (no output)
    #[arg(long)]
    check: bool,

    /// Output directory for multiple files
    #[arg(long)]
    outdir: Option<String>,

    /// Generate TypeScript declarations
    #[arg(long)]
    declarations: bool,
}

fn main() {
    let cli = Cli::parse();

    if cli.verbose {
        println!("🚀 Nagari Compiler v0.1.0");
        println!("📁 Input: {}", cli.input);
        println!("🎯 Target: {}", cli.target);
        if cli.jsx { println!("⚛️  JSX: enabled"); }
        if cli.bundle { println!("📦 Bundle: enabled"); }
        if cli.devtools { println!("🔧 DevTools: enabled"); }
    }

    if cli.watch {
        println!("🔍 Starting watch mode...");
        watch_mode(&cli);
        return;
    }

    if cli.check {
        if cli.verbose { println!("🔍 Checking syntax..."); }
        match check_syntax(&cli.input) {
            Ok(_) => {
                println!("✅ Syntax check passed");
                std::process::exit(0);
            }
            Err(e) => {
                eprintln!("❌ Syntax error: {}", e);
                std::process::exit(1);
            }
        }
    }

    match compile_file(&cli) {
        Ok(output_path) => {
            if cli.verbose {
                println!("✅ Compiled successfully to: {}", output_path);
            }

            // Post-processing steps
            if cli.bundle {
                if let Err(e) = bundle_output(&output_path, &cli) {
                    eprintln!("⚠️  Bundle failed: {}", e);
                }
            }

            if cli.minify {
                if let Err(e) = minify_output(&output_path) {
                    eprintln!("⚠️  Minification failed: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Compilation failed: {}", e);
            std::process::exit(1);
        }
    }
}

fn compile_file(cli: &Cli) -> Result<String, NagariError> {
    // Read input file
    let input_content = fs::read_to_string(&cli.input)
        .map_err(|e| NagariError::IoError(format!("Failed to read input file: {}", e)))?;

    // Parse and transpile
    let mut lexer = Lexer::new(&input_content);
    let tokens = lexer.tokenize()
        .map_err(|e| NagariError::LexError(format!("Lexing failed: {}", e)))?;

    let mut parser = NagParser::new(tokens);
    let ast = parser.parse()
        .map_err(|e| NagariError::ParseError(format!("Parsing failed: {}", e)))?;

    // Configure transpiler based on target
    let mut target = cli.target.clone();
    if cli.bundle && target == "es6" {
        target = "esm".to_string(); // Use ES modules for bundling
    }

    let js_code = transpiler::transpile(&ast, &target, cli.jsx)?;

    // Determine output path
    let output_path = if let Some(output) = &cli.output {
        output.clone()
    } else if let Some(outdir) = &cli.outdir {
        let input_path = Path::new(&cli.input);
        let filename = input_path.file_stem().unwrap().to_str().unwrap();
        format!("{}/{}.js", outdir, filename)
    } else {        let input_path = Path::new(&cli.input);
        let output_path = input_path.with_extension("js");
        output_path.to_string_lossy().to_string()
    };

    // Create output directory if needed
    if let Some(parent) = Path::new(&output_path).parent() {
        fs::create_dir_all(parent)
            .map_err(|e| NagariError::IoError(format!("Failed to create output directory: {}", e)))?;
    }

    // Add source map comment if enabled
    let final_code = if cli.sourcemap {
        format!("{}\n//# sourceMappingURL={}.map", js_code,
               Path::new(&output_path).file_name().unwrap().to_str().unwrap())
    } else {
        js_code
    };

    // Write output
    fs::write(&output_path, final_code)
        .map_err(|e| NagariError::IoError(format!("Failed to write output file: {}", e)))?;

    // Generate source map if enabled
    if cli.sourcemap {
        generate_sourcemap(&cli.input, &output_path, &input_content)?;
    }

    // Generate TypeScript declarations if enabled
    if cli.declarations {
        generate_declarations(&output_path, &ast)?;
    }

    Ok(output_path)
}

fn check_syntax(input_path: &str) -> Result<(), NagariError> {
    let input_content = fs::read_to_string(input_path)
        .map_err(|e| NagariError::IoError(format!("Failed to read input file: {}", e)))?;

    let mut lexer = Lexer::new(&input_content);
    let tokens = lexer.tokenize()
        .map_err(|e| NagariError::LexError(format!("Lexing failed: {}", e)))?;

    let mut parser = NagParser::new(tokens);
    parser.parse()
        .map_err(|e| NagariError::ParseError(format!("Parsing failed: {}", e)))?;

    Ok(())
}

fn watch_mode(cli: &Cli) {
    use std::time::Duration;
    use std::thread;

    println!("👀 Watching {} for changes...", cli.input);

    let mut last_modified = get_file_modified_time(&cli.input).unwrap_or(0);

    loop {
        thread::sleep(Duration::from_millis(500));

        if let Ok(current_modified) = get_file_modified_time(&cli.input) {
            if current_modified > last_modified {
                last_modified = current_modified;
                println!("🔄 File changed, recompiling...");

                match compile_file(cli) {
                    Ok(output_path) => {
                        println!("✅ Recompiled successfully: {}", output_path);
                    }
                    Err(e) => {
                        eprintln!("❌ Compilation error: {}", e);
                    }
                }
            }
        }
    }
}

fn get_file_modified_time(path: &str) -> Result<u64, std::io::Error> {
    let metadata = fs::metadata(path)?;
    Ok(metadata.modified()?.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs())
}

fn bundle_output(output_path: &str, cli: &Cli) -> Result<(), String> {
    if cli.verbose {
        println!("📦 Bundling with rollup...");
    }

    // Use rollup for bundling
    let mut cmd = Command::new("npx");
    cmd.args(&["rollup", output_path, "-f", "iife", "-o"]);

    let bundled_path = output_path.replace(".js", ".bundle.js");
    cmd.arg(&bundled_path);

    let output = cmd.output()
        .map_err(|e| format!("Failed to run rollup: {}", e))?;

    if !output.status.success() {
        return Err(format!("Rollup failed: {}", String::from_utf8_lossy(&output.stderr)));
    }

    if cli.verbose {
        println!("📦 Bundle created: {}", bundled_path);
    }

    Ok(())
}

fn minify_output(output_path: &str) -> Result<(), String> {
    // Use terser for minification
    let mut cmd = Command::new("npx");
    cmd.args(&["terser", output_path, "-o"]);

    let minified_path = output_path.replace(".js", ".min.js");
    cmd.arg(&minified_path);
    cmd.args(&["-c", "-m"]);

    let output = cmd.output()
        .map_err(|e| format!("Failed to run terser: {}", e))?;

    if !output.status.success() {
        return Err(format!("Terser failed: {}", String::from_utf8_lossy(&output.stderr)));
    }

    Ok(())
}

fn generate_sourcemap(input_path: &str, output_path: &str, source_content: &str) -> Result<(), NagariError> {
    // Simple source map generation
    let sourcemap = format!(r#"{{
  "version": 3,
  "file": "{}",
  "sources": ["{}"],
  "sourcesContent": [{}],
  "mappings": "AAAA"
}}"#,
        Path::new(output_path).file_name().unwrap().to_str().unwrap(),
        input_path,
        serde_json::to_string(source_content).unwrap()
    );

    let map_path = format!("{}.map", output_path);
    fs::write(&map_path, sourcemap)
        .map_err(|e| NagariError::IoError(format!("Failed to write source map: {}", e)))?;

    Ok(())
}

fn generate_declarations(output_path: &str, _ast: &ast::Program) -> Result<(), NagariError> {
    // Basic TypeScript declaration generation
    let declarations = "// Generated TypeScript declarations\nexport {};\n";

    let dts_path = output_path.replace(".js", ".d.ts");
    fs::write(&dts_path, declarations)
        .map_err(|e| NagariError::IoError(format!("Failed to write declarations: {}", e)))?;

    Ok(())
}
