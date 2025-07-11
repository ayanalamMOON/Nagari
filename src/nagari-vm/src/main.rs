use clap::Parser;
use std::fs;
use std::path::Path;

mod vm;
mod value;
mod bytecode;
mod builtins;
mod env;

use vm::VM;

#[derive(Parser)]
#[command(name = "nagrun")]
#[command(about = "Nagari virtual machine - runs .nac bytecode files")]
struct Cli {
    /// Input bytecode file (.nac)
    input: String,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Debug mode
    #[arg(short, long)]
    debug: bool,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match run_bytecode_file(&cli.input, cli.verbose, cli.debug).await {
        Ok(_) => {
            if cli.verbose {
                println!("✅ Execution completed successfully");
            }
        }
        Err(e) => {
            eprintln!("❌ Runtime error: {}", e);
            std::process::exit(1);
        }
    }
}

async fn run_bytecode_file(
    input_path: &str,
    verbose: bool,
    debug: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Check if file exists and has correct extension
    if !Path::new(input_path).exists() {
        return Err(format!("File not found: {}", input_path).into());
    }

    if !input_path.ends_with(".nac") {
        return Err("Input file must have .nac extension".into());
    }

    if verbose {
        println!("📖 Loading bytecode: {}", input_path);
    }

    // Read bytecode file
    let bytecode = fs::read(input_path)?;

    if verbose {
        println!("📦 Loaded {} bytes of bytecode", bytecode.len());
    }

    // Create and run VM
    let mut vm = VM::new(debug);
    vm.load_bytecode(&bytecode)?;

    if verbose {
        println!("🚀 Starting execution...");
    }

    vm.run().await?;

    Ok(())
}
