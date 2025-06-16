use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod commands;
mod config;
mod repl;
mod lsp;
mod utils;
mod tools;
mod package;
mod repl_engine;

use commands::*;
use config::NagConfig;

#[derive(Parser)]
#[command(name = "nag")]
#[command(about = "Nagari CLI tool for development and ecosystem management")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Configuration file path
    #[arg(short, long, global = true)]
    pub config: Option<PathBuf>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run a Nagari file directly
    Run {
        /// Input file path
        file: PathBuf,
        /// Arguments to pass to the program
        #[arg(last = true)]
        args: Vec<String>,
        /// Enable watch mode for hot reloading
        #[arg(short, long)]
        watch: bool,
    },

    /// Build/compile Nagari code
    Build {
        /// Input file or directory
        input: PathBuf,
        /// Output directory
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Compilation target (js, bytecode, wasm)
        #[arg(short, long, default_value = "js")]
        target: String,
        /// Enable optimizations
        #[arg(long)]
        release: bool,
        /// Generate source maps
        #[arg(long)]
        sourcemap: bool,
    },

    /// Transpile Nagari to JavaScript
    Transpile {
        /// Input file or directory
        input: PathBuf,
        /// Output directory
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Module format (es6, commonjs, umd)
        #[arg(short, long, default_value = "es6")]
        format: String,
        /// Enable minification
        #[arg(short, long)]
        minify: bool,
        /// Generate TypeScript declarations
        #[arg(long)]
        declarations: bool,
    },

    /// Bundle application for production
    Bundle {
        /// Entry point file
        entry: PathBuf,
        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Bundle format (browser, node, universal)
        #[arg(short, long, default_value = "browser")]
        format: String,
        /// Enable tree shaking
        #[arg(long)]
        treeshake: bool,
        /// External dependencies to exclude
        #[arg(long)]
        external: Vec<String>,
    },

    /// Format Nagari source code
    Format {
        /// Files or directories to format
        paths: Vec<PathBuf>,
        /// Check formatting without making changes
        #[arg(long)]
        check: bool,
        /// Print diff of changes
        #[arg(long)]
        diff: bool,
    },

    /// Lint Nagari source code
    Lint {
        /// Files or directories to lint
        paths: Vec<PathBuf>,
        /// Fix auto-fixable issues
        #[arg(long)]
        fix: bool,
        /// Output format (text, json, checkstyle)
        #[arg(long, default_value = "text")]
        format: String,
    },

    /// Run tests
    Test {
        /// Test files or directories
        paths: Vec<PathBuf>,
        /// Run tests matching pattern
        #[arg(short, long)]
        pattern: Option<String>,
        /// Enable coverage reporting
        #[arg(long)]
        coverage: bool,
        /// Enable watch mode
        #[arg(short, long)]
        watch: bool,
    },

    /// Interactive REPL
    Repl {
        /// Load script before starting REPL
        #[arg(short, long)]
        script: Option<PathBuf>,
        /// Enable experimental features
        #[arg(long)]
        experimental: bool,
    },

    /// Documentation commands
    Doc {
        #[command(subcommand)]
        command: DocCommands,
    },

    /// Package management commands
    Package {
        #[command(subcommand)]
        command: PackageCommands,
    },

    /// Language Server Protocol
    Lsp {
        /// LSP mode (stdio, tcp, websocket)
        #[arg(long, default_value = "stdio")]
        mode: String,
        /// TCP/WebSocket port (for non-stdio modes)
        #[arg(long)]
        port: Option<u16>,
    },

    /// Initialize new Nagari project
    Init {
        /// Project name
        name: Option<String>,
        /// Project template (basic, web, cli, library)
        #[arg(short, long, default_value = "basic")]
        template: String,
        /// Skip interactive prompts
        #[arg(long)]
        yes: bool,
    },

    /// Development server with hot reload
    Serve {
        /// Entry point file
        entry: Option<PathBuf>,
        /// Server port
        #[arg(short, long, default_value = "3000")]
        port: u16,
        /// Enable HTTPS
        #[arg(long)]
        https: bool,
        /// Public directory for static assets
        #[arg(long)]
        public: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
pub enum DocCommands {
    /// Generate documentation
    Generate {
        /// Source directory
        #[arg(short, long, default_value = ".")]
        source: PathBuf,
        /// Output directory
        #[arg(short, long, default_value = "docs")]
        output: PathBuf,
        /// Output format (html, markdown, json)
        #[arg(short, long, default_value = "html")]
        format: String,
        /// Include private items
        #[arg(long)]
        private: bool,
    },

    /// Serve documentation locally
    Serve {
        /// Documentation directory
        #[arg(default_value = "docs")]
        docs_dir: PathBuf,
        /// Server port
        #[arg(short, long, default_value = "8080")]
        port: u16,
    },

    /// Check documentation for broken links
    Check {
        /// Documentation directory
        #[arg(default_value = "docs")]
        docs_dir: PathBuf,
    },
}

#[derive(Subcommand)]
pub enum PackageCommands {
    /// Initialize package.json equivalent
    Init {
        /// Skip interactive prompts
        #[arg(long)]
        yes: bool,
    },

    /// Install packages
    Install {
        /// Package names to install
        packages: Vec<String>,
        /// Install as dev dependency
        #[arg(long)]
        dev: bool,
        /// Install globally
        #[arg(short, long)]
        global: bool,
        /// Exact version matching
        #[arg(long)]
        exact: bool,
    },

    /// Add package dependency
    Add {
        /// Package name
        package: String,
        /// Package version
        #[arg(short, long)]
        version: Option<String>,
        /// Add as dev dependency
        #[arg(long)]
        dev: bool,
    },

    /// Remove package dependency
    Remove {
        /// Package names to remove
        packages: Vec<String>,
    },

    /// Update dependencies
    Update {
        /// Specific packages to update
        packages: Vec<String>,
    },

    /// List installed packages
    List {
        /// Show dependency tree
        #[arg(long)]
        tree: bool,
        /// Show outdated packages
        #[arg(long)]
        outdated: bool,
    },

    /// Publish package
    Publish {
        /// Registry URL
        #[arg(long)]
        registry: Option<String>,
        /// Dry run without actually publishing
        #[arg(long)]
        dry_run: bool,
    },

    /// Pack package for distribution
    Pack {
        /// Output directory
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
      // Load configuration
    let mut config = NagConfig::load(cli.config.as_deref())?;

    // Override config with CLI flags
    if cli.verbose {
        config.verbose = true;
    }

    // Set up logging based on verbosity
    if cli.verbose {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Debug)
            .init();
    }

    // Execute command
    match cli.command {
        Commands::Run { file, args, watch } => {
            run_command(file, args, watch, &config).await
        }
        Commands::Build { input, output, target, release, sourcemap } => {
            build_command(input, output, target, release, sourcemap, &config).await
        }
        Commands::Transpile { input, output, format, minify, declarations } => {
            transpile_command(input, output, format, minify, declarations, &config).await
        }
        Commands::Bundle { entry, output, format, treeshake, external } => {
            bundle_command(entry, output, format, treeshake, external, &config).await
        }
        Commands::Format { paths, check, diff } => {
            format_command(paths, check, diff, &config).await
        }
        Commands::Lint { paths, fix, format } => {
            lint_command(paths, fix, format, &config).await
        }
        Commands::Test { paths, pattern, coverage, watch } => {
            test_command(paths, pattern, coverage, watch, &config).await
        }        Commands::Repl { script, experimental } => {
            handle_repl_command(script, experimental, &config).await
        }
        Commands::Doc { command } => {
            doc_command(command, &config).await
        }
        Commands::Package { command } => {
            handle_package_command(command, &config).await
        }
        Commands::Lsp { mode, port } => {
            lsp_command(mode, port, &config).await
        }
        Commands::Init { name, template, yes } => {
            init_command(name, template, yes, &config).await
        }
        Commands::Serve { entry, port, https, public } => {
            serve_command(entry, port, https, public, &config).await
        }
    }
}
