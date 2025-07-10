#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use clap::Parser;
use tower_lsp::{LspService, Server};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod backend;
mod capabilities;
mod completion;
mod diagnostics;
mod document;
mod goto;
mod hover;
mod references;
mod rename;
mod symbols;
mod workspace;
mod formatting;
mod semantic_tokens;
mod inlay_hints;
mod code_actions;

use backend::NagariLanguageServer;

#[derive(Parser)]
#[command(name = "nagari-lsp")]
#[command(about = "Nagari Language Server Protocol implementation")]
pub struct Args {
    /// Enable debug logging
    #[arg(long)]
    debug: bool,

    /// Use stdio for communication (default)
    #[arg(long)]
    stdio: bool,

    /// Use TCP socket for communication
    #[arg(long)]
    tcp: Option<u16>,

    /// Use WebSocket for communication
    #[arg(long)]
    websocket: Option<u16>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Initialize tracing
    let filter = if args.debug {
        "nagari_lsp=debug,tower_lsp=debug"
    } else {
        "nagari_lsp=info"
    };

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(filter))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Nagari Language Server");

    // Create the language server
    let (service, socket) = LspService::new(|client| NagariLanguageServer::new(client));

    // Start the server based on the communication method
    if let Some(port) = args.tcp {
        // TCP mode
        tracing::info!("Starting LSP server on TCP port {}", port);
        let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
        let (stream, _) = listener.accept().await?;
        let (read, write) = tokio::io::split(stream);
        Server::new(read, write, socket).serve(service).await;
    } else if let Some(port) = args.websocket {
        // WebSocket mode
        tracing::info!("Starting LSP server on WebSocket port {}", port);
        // TODO: Implement WebSocket support
        eprintln!("WebSocket mode not yet implemented");
        std::process::exit(1);
    } else {
        // Default to stdio
        tracing::info!("Starting LSP server on stdio");
        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();
        Server::new(stdin, stdout, socket).serve(service).await;
    }

    Ok(())
}
