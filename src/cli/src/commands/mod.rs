use crate::config::NagConfig;
use crate::package::PackageManager;
use crate::repl_engine::ReplEngine;
use crate::{DocCommands, PackageCommands};
use anyhow::{Context, Result};
use colored::*;
use notify::{recommended_watcher, RecursiveMode, Watcher};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::time::Duration;
use tokio::process::Command;

pub async fn run_command(
    file: PathBuf,
    args: Vec<String>,
    watch: bool,
    config: &NagConfig,
) -> Result<()> {
    println!("{} Running {}", "✓".green().bold(), file.display());

    if watch {
        println!(
            "{} Watch mode enabled - file changes will trigger restart",
            "👀".yellow()
        );
        let (tx, rx) = channel();
        let mut watcher = recommended_watcher(tx).context("Failed to create file watcher")?;

        watcher
            .watch(&file, RecursiveMode::NonRecursive)
            .context("Failed to watch file")?;

        loop {
            // Initial run
            println!("{} Running {}", "▶️".blue().bold(), file.display());

            match run_file_once(&file, &args, config).await {
                Ok(_) => println!("{} Execution completed", "✓".green()),
                Err(e) => println!("{} Execution failed: {}", "❌".red(), e),
            }

            println!("{} Waiting for file changes...", "👀".yellow());

            // Wait for file changes
            match rx.recv() {
                Ok(_) => {
                    println!("{} File changed, restarting...", "🔄".cyan());
                    // Small delay to avoid rapid restarts
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
                Err(e) => {
                    println!("{} Watch error: {}", "❌".red(), e);
                    break;
                }
            }
        }

        return Ok(());
    }

    // Single run
    run_file_once(&file, &args, config).await
}

async fn run_file_once(file: &PathBuf, args: &[String], config: &NagConfig) -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let output_file = temp_dir.path().join("output.js");

    // Setup runtime in temp directory
    setup_runtime_in_temp_dir(temp_dir.path())?;

    // Create compiler with configuration
    let compiler_config = nagari_compiler::CompilerConfigBuilder::new()
        .target(&config.build.target)
        .jsx(config.build.jsx)
        .sourcemap(config.build.sourcemap)
        .verbose(config.verbose)
        .build();

    let compiler = nagari_compiler::Compiler::with_config(compiler_config);

    // Compile the file
    match compiler.compile_to_file(file, &output_file) {
        Ok(_) => {
            // Run with Node.js
            let mut cmd = Command::new("node");
            cmd.arg(&output_file);
            cmd.args(args);

            let status = cmd.status().await?;

            if !status.success() {
                anyhow::bail!("Program exited with code: {}", status.code().unwrap_or(1));
            }
        }
        Err(e) => {
            anyhow::bail!("Compilation failed: {}", e);
        }
    }

    Ok(())
}

/// Setup the Nagari runtime in a temporary directory for execution
fn setup_runtime_in_temp_dir(temp_dir: &Path) -> Result<()> {
    // Find the nagari-runtime directory relative to the CLI
    let runtime_path = find_nagari_runtime_path()?;

    // Create node_modules/nagari-runtime in temp directory
    let node_modules_dir = temp_dir.join("node_modules");
    let runtime_dest = node_modules_dir.join("nagari-runtime");

    fs::create_dir_all(&runtime_dest)
        .context("Failed to create node_modules directory in temp dir")?;

    // Copy runtime files
    copy_dir_recursive(&runtime_path, &runtime_dest)
        .context("Failed to copy nagari-runtime to temp directory")?;

    // Create package.json in temp dir to enable ES6 modules
    let package_json = r#"{
  "type": "module"
}"#;

    fs::write(temp_dir.join("package.json"), package_json)
        .context("Failed to write package.json in temp directory")?;

    Ok(())
}

/// Find the nagari-runtime directory path
fn find_nagari_runtime_path() -> Result<PathBuf> {
    // Try to find nagari-runtime relative to current executable or working directory
    let current_exe = std::env::current_exe().context("Failed to get current executable path")?;

    // Look for runtime relative to executable (in case CLI is installed)
    let exe_parent = current_exe
        .parent()
        .context("Failed to get executable parent directory")?;

    // Try multiple possible locations
    let possible_paths = [
        // Relative to executable (development)
        exe_parent.join("../nagari-runtime"),
        // Relative to working directory (development)
        PathBuf::from("src/nagari-runtime"),
        // Legacy location (for backwards compatibility)
        PathBuf::from("nagari-runtime"),
        // Relative to executable parent (installed)
        exe_parent.join("../share/nagari/runtime"),
        // System-wide installation
        PathBuf::from("/usr/share/nagari/runtime"),
        // Windows system-wide
        PathBuf::from("C:\\Program Files\\Nagari\\runtime"),
    ];

    for path in &possible_paths {
        let full_path = path.join("dist");
        if full_path.exists() && full_path.join("index.js").exists() {
            return Ok(path.clone());
        }
    }

    anyhow::bail!(
        "Could not find nagari-runtime. Please ensure it's built and available. \
         Tried paths: {:?}",
        possible_paths
    );
}

/// Recursively copy a directory
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    if !src.is_dir() {
        anyhow::bail!("Source path is not a directory: {}", src.display());
    }

    fs::create_dir_all(dst)
        .with_context(|| format!("Failed to create destination directory: {}", dst.display()))?;

    for entry in fs::read_dir(src)
        .with_context(|| format!("Failed to read source directory: {}", src.display()))?
    {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();
        let name = entry.file_name();
        let dest_path = dst.join(&name);

        if path.is_dir() {
            copy_dir_recursive(&path, &dest_path)?;
        } else {
            fs::copy(&path, &dest_path).with_context(|| {
                format!(
                    "Failed to copy file from {} to {}",
                    path.display(),
                    dest_path.display()
                )
            })?;
        }
    }

    Ok(())
}

pub async fn build_command(
    input: PathBuf,
    output: Option<PathBuf>,
    target: String,
    release: bool,
    sourcemap: bool,
    config: &NagConfig,
) -> Result<()> {
    println!(
        "{} Building {} (target: {})",
        "🔨".yellow(),
        input.display(),
        target
    );
    let output_dir = output.unwrap_or_else(|| PathBuf::from(&config.project.output_dir));
    std::fs::create_dir_all(&output_dir)?;

    // Create compiler with configuration
    let compiler_config = nagari_compiler::CompilerConfigBuilder::new()
        .target(&target)
        .sourcemap(sourcemap)
        .verbose(config.verbose)
        .minify(release)
        .build();

    let compiler = nagari_compiler::Compiler::with_config(compiler_config);

    match target.as_str() {
        "js" => {
            if input.is_file() {
                let output_file = output_dir
                    .join(input.file_stem().unwrap())
                    .with_extension("js");
                compiler.compile_to_file(&input, &output_file)?;
                println!("{} Generated {}", "✓".green(), output_file.display());
            } else {
                // Process directory recursively
                for entry in walkdir::WalkDir::new(&input) {
                    let entry = entry?;
                    if entry.file_type().is_file()
                        && entry.path().extension().and_then(|s| s.to_str()) == Some("nag")
                    {
                        let relative_path = entry.path().strip_prefix(&input)?;
                        let output_file = output_dir.join(relative_path).with_extension("js");

                        if let Some(parent) = output_file.parent() {
                            std::fs::create_dir_all(parent)?;
                        }

                        compiler.compile_to_file(entry.path(), &output_file)?;
                        println!("{} Generated {}", "✓".green(), output_file.display());
                    }
                }
            }
        }
        "bytecode" => {
            println!("{} Bytecode target not yet implemented", "⚠️".yellow());
        }
        "wasm" => {
            println!("{} WASM target not yet implemented", "⚠️".yellow());
        }
        _ => {
            anyhow::bail!("Unknown target: {}", target);
        }
    }

    println!("{} Build completed!", "🎉".green().bold());
    Ok(())
}

pub async fn transpile_command(
    input: PathBuf,
    output: Option<PathBuf>,
    format: String,
    _minify: bool,
    declarations: bool,
    config: &NagConfig,
) -> Result<()> {
    println!(
        "{} Transpiling {} (format: {})",
        "🔄".cyan(),
        input.display(),
        format
    );

    let output_dir = output.unwrap_or_else(|| PathBuf::from(&config.project.output_dir));
    build_command(
        input,
        Some(output_dir),
        "js".to_string(),
        false,
        true,
        config,
    )
    .await?;

    if declarations {
        println!(
            "{} TypeScript declarations not yet implemented",
            "⚠️".yellow()
        );
    }

    Ok(())
}

pub async fn bundle_command(
    entry: PathBuf,
    output: Option<PathBuf>,
    format: String,
    _treeshake: bool,
    _external: Vec<String>,
    config: &NagConfig,
) -> Result<()> {
    println!(
        "{} Bundling {} (format: {})",
        "📦".cyan(),
        entry.display(),
        format
    );

    // For now, just transpile the entry point
    let output_file = output.unwrap_or_else(|| PathBuf::from("bundle.js"));
    transpile_command(
        entry,
        Some(output_file.parent().unwrap().to_path_buf()),
        format,
        false,
        false,
        config,
    )
    .await?;

    println!("{} Bundle created: {}", "✓".green(), output_file.display());
    Ok(())
}

pub async fn format_command(
    paths: Vec<PathBuf>,
    check: bool,
    diff: bool,
    config: &NagConfig,
) -> Result<()> {
    if check {
        println!("{} Checking formatting...", "🔍".cyan());
    } else {
        println!("{} Formatting files...", "✨".cyan());
    }

    let formatter = crate::tools::formatter::NagFormatter::new(&config.format);
    let mut total_files = 0;
    let mut changed_files = 0;

    for path in paths {
        if path.is_file() {
            if path.extension().and_then(|s| s.to_str()) == Some("nag") {
                let result = formatter.format_file(&path, check, diff)?;
                total_files += 1;
                if result.changed {
                    changed_files += 1;
                }

                if result.changed && diff {
                    println!("{}", result.diff.unwrap_or_default());
                }
            }
        } else {
            for entry in walkdir::WalkDir::new(&path) {
                let entry = entry?;
                if entry.file_type().is_file()
                    && entry.path().extension().and_then(|s| s.to_str()) == Some("nag")
                {
                    let result = formatter.format_file(entry.path(), check, diff)?;
                    total_files += 1;
                    if result.changed {
                        changed_files += 1;
                    }

                    if result.changed && diff {
                        println!("{}", result.diff.unwrap_or_default());
                    }
                }
            }
        }
    }

    if check {
        if changed_files > 0 {
            println!("{} {} files need formatting", "❌".red(), changed_files);
            std::process::exit(1);
        } else {
            println!("{} All files are properly formatted", "✓".green());
        }
    } else {
        println!(
            "{} Formatted {} files ({} changed)",
            "✓".green(),
            total_files,
            changed_files
        );
    }

    Ok(())
}

pub async fn lint_command(
    paths: Vec<PathBuf>,
    fix: bool,
    format: String,
    config: &NagConfig,
) -> Result<()> {
    println!("{} Linting files...", "🔍".cyan());

    let linter = crate::tools::linter::NagLinter::new(&config.lint);
    let mut all_issues = Vec::new();

    for path in paths {
        let issues = linter.lint_path(&path, fix)?;
        all_issues.extend(issues);
    }

    let stats = linter.get_statistics(&all_issues);

    if !all_issues.is_empty() {
        let formatted_output = linter.format_issues(&all_issues, &format)?;
        if !formatted_output.is_empty() {
            println!("{}", formatted_output);
        }
    }

    // Print summary
    if config.verbose {
        println!("{}", stats.summary());
    }

    if stats.total > 0 {
        if stats.has_errors() {
            println!(
                "{} Found {} issues ({} errors)",
                "❌".red(),
                stats.total,
                stats.errors
            );
            if !fix {
                println!("Run with --fix to automatically fix issues where possible");
            }
            // Exit with error code if there are errors
            std::process::exit(1);
        } else {
            println!("{} Found {} issues", "⚠️".yellow(), stats.total);
            if !fix && stats.fixable > 0 {
                println!(
                    "Run with --fix to automatically fix {} issues",
                    stats.fixable
                );
            }
        }
    } else {
        println!("{} No issues found", "✓".green());
    }

    Ok(())
}

pub async fn test_command(
    _paths: Vec<PathBuf>,
    _pattern: Option<String>,
    coverage: bool,
    watch: bool,
    _config: &NagConfig,
) -> Result<()> {
    println!("{} Running tests...", "🧪".cyan());

    if watch {
        println!("{} Watch mode enabled", "👀".yellow());
    }

    if coverage {
        println!("{} Coverage reporting enabled", "📊".cyan());
    }

    // TODO: Implement test runner
    println!("{} Test runner not yet implemented", "⚠️".yellow());

    Ok(())
}

#[allow(dead_code)]
pub async fn repl_command(
    script: Option<PathBuf>,
    experimental: bool,
    config: &NagConfig,
) -> Result<()> {
    println!("{} Starting Nagari REPL...", "🔄".cyan());

    if experimental {
        println!("{} Experimental features enabled", "🧪".yellow());
    }

    let repl = crate::repl::NagRepl::new(config.clone());

    if let Some(script_path) = script {
        repl.load_script(&script_path).await?;
    }

    repl.run().await?;

    Ok(())
}

pub async fn doc_command(command: DocCommands, config: &NagConfig) -> Result<()> {
    match command {
        DocCommands::Generate {
            source,
            output,
            format,
            private,
        } => {
            println!("{} Generating documentation...", "📚".cyan());

            let doc_gen = crate::tools::doc_generator::DocGenerator::new(config);
            doc_gen.generate(&source, &output, &format, private)?;

            println!(
                "{} Documentation generated in {}",
                "✓".green(),
                output.display()
            );
        }
        DocCommands::Serve { docs_dir: _, port } => {
            println!(
                "{} Serving documentation on http://localhost:{}",
                "🌐".cyan(),
                port
            );
            // TODO: Implement doc server
        }
        DocCommands::Check { docs_dir: _ } => {
            println!("{} Checking documentation...", "🔍".cyan());
            // TODO: Implement doc checker
        }
    }

    Ok(())
}

#[allow(dead_code)]
pub async fn package_command(command: PackageCommands, config: &NagConfig) -> Result<()> {
    match command {
        PackageCommands::Init { yes } => {
            println!("{} Initializing package...", "📦".cyan());
            crate::tools::package_manager::init_package(yes, config).await?;
        }
        PackageCommands::Install {
            packages,
            dev,
            global,
            exact,
        } => {
            println!("{} Installing packages...", "📦".cyan());
            crate::tools::package_manager::install_packages(packages, dev, global, exact, config)
                .await?;
        }
        PackageCommands::Add {
            package,
            version,
            dev,
        } => {
            println!("{} Adding package: {}", "📦".cyan(), package);
            crate::tools::package_manager::add_package(package, version, dev, config).await?;
        }
        PackageCommands::Remove { packages } => {
            println!("{} Removing packages...", "📦".cyan());
            crate::tools::package_manager::remove_packages(packages, config).await?;
        }
        PackageCommands::Update { packages } => {
            println!("{} Updating packages...", "📦".cyan());
            crate::tools::package_manager::update_packages(packages, config).await?;
        }
        PackageCommands::List { tree, outdated } => {
            crate::tools::package_manager::list_packages(tree, outdated, config).await?;
        }
        PackageCommands::Publish { registry, dry_run } => {
            println!("{} Publishing package...", "📦".cyan());
            crate::tools::package_manager::publish_package(registry, dry_run, config).await?;
        }
        PackageCommands::Pack { output } => {
            println!("{} Packing package...", "📦".cyan());
            crate::tools::package_manager::pack_package(output, config).await?;
        }
        PackageCommands::Uninstall { packages } => {
            println!("{} Uninstalling packages...", "📦".cyan());
            crate::tools::package_manager::remove_packages(packages, config).await?;
        }
        PackageCommands::Search { query } => {
            println!("{} Searching packages: {}", "📦".cyan(), query);
            // TODO: Implement package search
        }
        PackageCommands::Info { package } => {
            println!("{} Package info: {}", "📦".cyan(), package);
            // TODO: Implement package info
        }
        PackageCommands::Unpublish {
            package: _,
            version: _,
            force: _,
        } => {
            println!("{} Unpublishing package...", "📦".cyan());
            // TODO: Implement package unpublish
        }
        PackageCommands::Login { registry: _ } => {
            println!("{} Logging in to registry...", "📦".cyan());
            // TODO: Implement registry login
        }
        PackageCommands::Logout => {
            println!("{} Logging out from registry...", "📦".cyan());
            // TODO: Implement registry logout
        }
        PackageCommands::Cache { command } => {
            match command {
                crate::CacheCommands::Clear => {
                    println!("{} Clearing package cache...", "📦".cyan());
                    // TODO: Implement cache clear
                }
                crate::CacheCommands::Info => {
                    println!("{} Package cache info...", "📦".cyan());
                    // TODO: Implement cache info
                }
            }
        }
    }

    Ok(())
}

// Package management commands
pub async fn handle_package_command(
    package_command: PackageCommands,
    config: &NagConfig,
) -> Result<()> {
    let mut package_manager = PackageManager::new(config.clone())?;

    match package_command {
        PackageCommands::Init { yes } => {
            package_manager.init_package(None, yes).await?;
        }
        PackageCommands::Install {
            packages,
            dev,
            global: _,
            exact: _,
        } => {
            if packages.is_empty() {
                // Install from manifest
                package_manager.install(vec![], false).await?;
            } else {
                package_manager.install(packages, dev).await?;
            }
        }
        PackageCommands::Uninstall { packages } => {
            package_manager.uninstall(packages).await?;
        }
        PackageCommands::Update { packages } => {
            package_manager.update(Some(packages)).await?;
        }
        PackageCommands::List {
            tree: _,
            outdated: _,
        } => {
            package_manager.list().await?;
        }
        PackageCommands::Search { query } => {
            package_manager.search(query).await?;
        }
        PackageCommands::Info { package } => {
            package_manager.info(package).await?;
        }
        PackageCommands::Publish { .. } => {
            println!("{} Package publishing not yet implemented", "⚠️".yellow());
        }
        PackageCommands::Unpublish { .. } => {
            println!("{} Package unpublishing not yet implemented", "⚠️".yellow());
        }
        PackageCommands::Login { registry } => {
            println!(
                "{} Registry login not yet implemented (registry: {:?})",
                "⚠️".yellow(),
                registry
            );
        }
        PackageCommands::Logout => {
            println!("{} Registry logout not yet implemented", "⚠️".yellow());
        }
        PackageCommands::Cache { command } => match command {
            crate::CacheCommands::Info => {
                package_manager.cache_info().await?;
            }
            crate::CacheCommands::Clear => {
                package_manager.cache_clean().await?;
            }
        },
        PackageCommands::Add {
            package,
            version,
            dev,
        } => {
            let pkg_with_version = if let Some(v) = version {
                format!("{}@{}", package, v)
            } else {
                package
            };
            package_manager.install(vec![pkg_with_version], dev).await?;
        }
        PackageCommands::Remove { packages } => {
            package_manager.uninstall(packages).await?;
        }
        PackageCommands::Pack { output: _ } => {
            println!("{} Package packing not yet implemented", "⚠️".yellow());
        }
    }

    Ok(())
}

// Enhanced REPL command
pub async fn handle_repl_command(
    script: Option<PathBuf>,
    load: Option<PathBuf>,
    save: Option<PathBuf>,
    session: Option<PathBuf>,
    config: &NagConfig,
) -> Result<()> {
    let mut repl = ReplEngine::new(config.clone())?;

    // Load script if provided
    if let Some(script_path) = script {
        repl.load_script(&script_path).await?;
    }

    // Load session if provided
    if let Some(session_path) = session {
        repl.load_session(&session_path)?;
    }

    // Load additional script if provided
    if let Some(load_path) = load {
        repl.load_script(&load_path).await?;
    }

    // Run the REPL
    repl.run().await?;

    // Save session if requested
    if let Some(save_path) = save {
        repl.save_session(&save_path)?;
    }

    Ok(())
}

pub async fn lsp_command(mode: String, port: Option<u16>, config: &NagConfig) -> Result<()> {
    println!(
        "{} Starting Nagari Language Server (mode: {})",
        "🔧".cyan(),
        mode
    );

    let lsp_server = crate::lsp::NagLspServer::new(config.clone());

    match mode.as_str() {
        "stdio" => {
            lsp_server.run_stdio().await?;
        }
        "tcp" => {
            let port = port.unwrap_or(9257);
            lsp_server.run_tcp(port).await?;
        }
        "websocket" => {
            let port = port.unwrap_or(9258);
            lsp_server.run_websocket(port).await?;
        }
        _ => {
            anyhow::bail!("Unknown LSP mode: {}", mode);
        }
    }

    Ok(())
}

pub async fn init_command(
    name: Option<String>,
    template: String,
    _yes: bool,
    _config: &NagConfig,
) -> Result<()> {
    let project_name = name.unwrap_or_else(|| "nagari-project".to_string());

    println!(
        "{} Initializing new Nagari project: {}",
        "🚀".cyan(),
        project_name
    );
    println!("Template: {}", template);

    let project_dir = PathBuf::from(&project_name);
    std::fs::create_dir_all(&project_dir)?;

    // Create project structure based on template
    match template.as_str() {
        "basic" => create_basic_template(&project_dir, &project_name)?,
        "web" => create_web_template(&project_dir, &project_name)?,
        "cli" => create_cli_template(&project_dir, &project_name)?,
        "library" => create_library_template(&project_dir, &project_name)?,
        _ => anyhow::bail!("Unknown template: {}", template),
    }

    println!("{} Project initialized successfully!", "✓".green().bold());
    println!("Next steps:");
    println!("  cd {}", project_name);
    println!("  nag run main.nag");

    Ok(())
}

pub async fn serve_command(
    entry: Option<PathBuf>,
    port: u16,
    https: bool,
    public: Option<PathBuf>,
    _config: &NagConfig,
) -> Result<()> {
    let entry_file = entry.unwrap_or_else(|| PathBuf::from("main.nag"));

    println!("{} Starting development server...", "🌐".cyan());
    println!("Entry: {}", entry_file.display());
    println!("Port: {}", port);
    println!("HTTPS: {}", https);

    if let Some(public_dir) = &public {
        println!("Public: {}", public_dir.display());
    }

    // TODO: Implement dev server with hot reload
    println!("{} Development server not yet implemented", "⚠️".yellow());

    Ok(())
}

// Template creation functions
fn create_basic_template(dir: &Path, name: &str) -> Result<()> {
    // Create basic project structure
    std::fs::create_dir_all(dir.join("src"))?;

    // Create main.nag
    let main_content = r#"def greet(name: str = "World") -> str:
    return f"Hello, {name}!"

def main():
    message = greet("Nagari")
    print(message)

if __name__ == "__main__":
    main()
"#;

    std::fs::write(dir.join("main.nag"), main_content)?;

    // Create nagari.toml
    let config_content = format!(
        r#"[project]
name = "{}"
version = "0.1.0"
description = "A Nagari project"
main = "main.nag"

[build]
target = "js"
optimization = false
sourcemap = true
"#,
        name
    );

    std::fs::write(dir.join("nagari.toml"), config_content)?;

    // Create .gitignore
    let gitignore_content = r#"# Nagari build outputs
dist/
*.js.map

# Dependencies
node_modules/
nag_modules/

# IDE
.vscode/
.idea/

# OS
.DS_Store
Thumbs.db
"#;

    std::fs::write(dir.join(".gitignore"), gitignore_content)?;

    Ok(())
}

fn create_web_template(dir: &Path, name: &str) -> Result<()> {
    create_basic_template(dir, name)?;

    // Add web-specific files
    let index_html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Nagari Web App</title>
</head>
<body>
    <div id="app"></div>
    <script type="module" src="dist/main.js"></script>
</body>
</html>
"#;

    std::fs::write(dir.join("index.html"), index_html)?;

    Ok(())
}

fn create_cli_template(dir: &Path, name: &str) -> Result<()> {
    create_basic_template(dir, name)?;

    // Modify main.nag for CLI
    let cli_main = r#"import sys
import os

def main():
    args = sys.argv[1:]

    if len(args) == 0:
        print("Usage: nag run main.nag <command> [args...]")
        return

    command = args[0]

    if command == "hello":
        name = args[1] if len(args) > 1 else "World"
        print(f"Hello, {name}!")
    elif command == "version":
        print("1.0.0")
    else:
        print(f"Unknown command: {command}")

if __name__ == "__main__":
    main()
"#;

    std::fs::write(dir.join("main.nag"), cli_main)?;

    Ok(())
}

fn create_library_template(dir: &Path, name: &str) -> Result<()> {
    create_basic_template(dir, name)?;

    // Create lib structure
    std::fs::create_dir_all(dir.join("src"))?;

    let lib_content = r#""""
A sample Nagari library
"""

def add(a: int, b: int) -> int:
    """Add two numbers together."""
    return a + b

def multiply(a: int, b: int) -> int:
    """Multiply two numbers."""
    return a * b

class Calculator:
    """A simple calculator class."""

    def __init__(self):
        self.history = []

    def add(self, a: int, b: int) -> int:
        result = add(a, b)
        self.history.append(f"{a} + {b} = {result}")
        return result

    def multiply(self, a: int, b: int) -> int:
        result = multiply(a, b)
        self.history.append(f"{a} * {b} = {result}")
        return result

    def get_history(self) -> list:
        return self.history.copy()
"#;

    std::fs::write(dir.join("src").join("lib.nag"), lib_content)?;

    // Create test file
    let test_content = r#"from src.lib import add, multiply, Calculator

def test_add():
    assert add(2, 3) == 5
    assert add(-1, 1) == 0

def test_multiply():
    assert multiply(2, 3) == 6
    assert multiply(-1, 5) == -5

def test_calculator():
    calc = Calculator()

    result = calc.add(2, 3)
    assert result == 5

    result = calc.multiply(4, 5)
    assert result == 20

    history = calc.get_history()
    assert len(history) == 2
    assert "2 + 3 = 5" in history
    assert "4 * 5 = 20" in history

if __name__ == "__main__":
    test_add()
    test_multiply()
    test_calculator()
    print("All tests passed!")
"#;

    std::fs::write(dir.join("test_lib.nag"), test_content)?;

    Ok(())
}
