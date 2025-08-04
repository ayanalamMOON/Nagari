#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use clap::Parser;
use futures_util::{SinkExt, StreamExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use tower_lsp::{LspService, Server};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod backend;
mod capabilities;
mod code_actions;
mod completion;
mod diagnostics;
mod document;
mod formatting;
mod goto;
mod hover;
mod inlay_hints;
mod references;
mod rename;
mod semantic_tokens;
mod symbols;
mod workspace;

use backend::NagariLanguageServer;

// Helper function to extract LSP messages from accumulated data
fn extract_lsp_message(data: &str) -> Option<(String, String)> {
    // Look for Content-Length header
    if let Some(header_end) = data.find("\r\n\r\n") {
        let headers = &data[..header_end];
        let body_start = header_end + 4;

        // Parse Content-Length
        if let Some(content_length_line) = headers
            .lines()
            .find(|line| line.starts_with("Content-Length:"))
        {
            if let Some(length_str) = content_length_line.split(':').nth(1) {
                if let Ok(content_length) = length_str.trim().parse::<usize>() {
                    let body_end = body_start + content_length;
                    if data.len() >= body_end {
                        let json_content = data[body_start..body_end].to_string();
                        let remaining = data[body_end..].to_string();
                        return Some((json_content, remaining));
                    }
                }
            }
        }
    }
    None
}

// WebSocket message handler
async fn handle_websocket_connection(
    ws_stream: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
) -> anyhow::Result<()> {
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    // Create pipes for LSP communication
    let (client_reader, mut client_writer) = tokio::io::duplex(8192);
    let (mut server_reader, server_writer) = tokio::io::duplex(8192);

    // Create the language server with the pipe I/O
    let (service, socket) = LspService::new(NagariLanguageServer::new);

    // Task to forward messages from WebSocket to LSP server
    let ws_to_server_task = tokio::spawn(async move {
        while let Some(msg) = ws_receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    tracing::debug!("Received from WebSocket: {}", text);
                    // Format message with LSP headers
                    let content_length = text.len();
                    let lsp_message = format!("Content-Length: {}\r\n\r\n{}", content_length, text);
                    if let Err(e) = client_writer.write_all(lsp_message.as_bytes()).await {
                        tracing::error!("Failed to write to LSP server: {}", e);
                        break;
                    }
                }
                Ok(Message::Binary(data)) => {
                    tracing::debug!("Received binary data from WebSocket");
                    if let Err(e) = client_writer.write_all(&data).await {
                        tracing::error!("Failed to write binary to LSP server: {}", e);
                        break;
                    }
                }
                Ok(Message::Close(_)) => {
                    tracing::info!("WebSocket connection closed");
                    break;
                }
                Ok(Message::Ping(_)) => {
                    tracing::debug!("Received WebSocket ping");
                    // Pings are automatically handled by tungstenite
                }
                Ok(Message::Pong(_)) => {
                    tracing::debug!("Received WebSocket pong");
                    // Pongs are automatically handled by tungstenite
                }
                Ok(Message::Frame(_)) => {
                    tracing::debug!("Received WebSocket frame");
                    // Frames are low-level and typically handled internally
                }
                Err(e) => {
                    tracing::error!("WebSocket error: {}", e);
                    break;
                }
            }
        }
    });

    // Task to forward messages from LSP server to WebSocket
    let server_to_ws_task = tokio::spawn(async move {
        let mut buffer = vec![0u8; 8192];
        let mut accumulated_data = String::new();

        loop {
            match server_reader.read(&mut buffer).await {
                Ok(0) => {
                    tracing::info!("LSP server closed connection");
                    break;
                }
                Ok(n) => {
                    let data = &buffer[..n];
                    if let Ok(text) = std::str::from_utf8(data) {
                        accumulated_data.push_str(text);

                        // Process complete LSP messages
                        while let Some((json_content, remaining)) =
                            extract_lsp_message(&accumulated_data)
                        {
                            tracing::debug!("Sending to WebSocket: {}", json_content);
                            if let Err(e) = ws_sender.send(Message::Text(json_content)).await {
                                tracing::error!("Failed to send WebSocket message: {}", e);
                                return;
                            }
                            accumulated_data = remaining;
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to read from LSP server: {}", e);
                    break;
                }
            }
        }
    });

    // Create LSP server with the pipes
    let server_task = tokio::spawn(async move {
        let server = Server::new(client_reader, server_writer, socket);
        server.serve(service).await;
    });

    // Wait for any task to complete
    tokio::select! {
        _ = ws_to_server_task => tracing::info!("WebSocket to server task completed"),
        _ = server_to_ws_task => tracing::info!("Server to WebSocket task completed"),
        _ = server_task => tracing::info!("LSP server task completed"),
    }

    Ok(())
}

// Create I/O bridge between WebSocket channels and LSP
fn create_lsp_io_bridge(
    ws_to_lsp_rx: tokio::sync::mpsc::UnboundedReceiver<Vec<u8>>,
    lsp_to_ws_tx: tokio::sync::mpsc::UnboundedSender<Vec<u8>>,
) -> (
    impl tokio::io::AsyncRead + Unpin,
    impl tokio::io::AsyncWrite + Unpin,
) {
    // Create a pipe for LSP communication
    let (read_tx, read_rx) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();
    let (write_tx, mut write_rx) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();

    // Task to handle incoming messages from WebSocket to LSP
    let mut ws_to_lsp_rx = ws_to_lsp_rx;
    tokio::spawn(async move {
        while let Some(data) = ws_to_lsp_rx.recv().await {
            if read_tx.send(data).is_err() {
                break;
            }
        }
    });

    // Task to handle outgoing messages from LSP to WebSocket
    tokio::spawn(async move {
        while let Some(bytes) = write_rx.recv().await {
            if lsp_to_ws_tx.send(bytes).is_err() {
                break;
            }
        }
    });

    // Create async readers and writers
    let reader = AsyncChannelReader::new(read_rx);
    let writer = AsyncChannelWriter::new(write_tx);

    (reader, writer)
}

// Async reader that reads from a channel
struct AsyncChannelReader {
    receiver: tokio::sync::mpsc::UnboundedReceiver<Vec<u8>>,
    buffer: Vec<u8>,
    pos: usize,
}

impl AsyncChannelReader {
    fn new(receiver: tokio::sync::mpsc::UnboundedReceiver<Vec<u8>>) -> Self {
        Self {
            receiver,
            buffer: Vec::new(),
            pos: 0,
        }
    }
}

impl tokio::io::AsyncRead for AsyncChannelReader {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        loop {
            // If we have data in the buffer, read from it
            if self.pos < self.buffer.len() {
                let remaining = self.buffer.len() - self.pos;
                let to_copy = std::cmp::min(remaining, buf.remaining());
                buf.put_slice(&self.buffer[self.pos..self.pos + to_copy]);
                self.pos += to_copy;

                if self.pos >= self.buffer.len() {
                    self.buffer.clear();
                    self.pos = 0;
                }

                return std::task::Poll::Ready(Ok(()));
            }

            // Try to receive new data
            match self.receiver.poll_recv(cx) {
                std::task::Poll::Ready(Some(data)) => {
                    self.buffer = data;
                    self.pos = 0;
                    // Continue to read from the new buffer
                }
                std::task::Poll::Ready(None) => {
                    // Channel closed
                    return std::task::Poll::Ready(Ok(()));
                }
                std::task::Poll::Pending => {
                    return std::task::Poll::Pending;
                }
            }
        }
    }
}

// Async writer that writes to a channel
struct AsyncChannelWriter {
    sender: tokio::sync::mpsc::UnboundedSender<Vec<u8>>,
}

impl AsyncChannelWriter {
    fn new(sender: tokio::sync::mpsc::UnboundedSender<Vec<u8>>) -> Self {
        Self { sender }
    }
}

impl tokio::io::AsyncWrite for AsyncChannelWriter {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        match self.sender.send(buf.to_vec()) {
            Ok(()) => std::task::Poll::Ready(Ok(buf.len())),
            Err(_) => std::task::Poll::Ready(Err(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "Channel closed",
            ))),
        }
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        std::task::Poll::Ready(Ok(()))
    }
}

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
    let (service, socket) = LspService::new(NagariLanguageServer::new);

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
        let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await?;

        loop {
            let (stream, addr) = listener.accept().await?;
            tracing::info!("New connection from {}", addr);

            // Handle WebSocket upgrade
            match tokio_tungstenite::accept_async(stream).await {
                Ok(ws_stream) => {
                    tracing::info!("WebSocket connection established with {}", addr);

                    // Handle the WebSocket connection
                    let connection_task = tokio::spawn(async move {
                        if let Err(e) = handle_websocket_connection(ws_stream).await {
                            tracing::error!("WebSocket connection error: {}", e);
                        }
                    });

                    // Don't wait for the connection to finish, accept new connections
                    tokio::spawn(async move {
                        if let Err(e) = connection_task.await {
                            tracing::error!("Connection task error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    tracing::error!("Failed to establish WebSocket connection: {}", e);
                }
            }
        }
    } else {
        // Default to stdio
        tracing::info!("Starting LSP server on stdio");
        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();
        Server::new(stdin, stdout, socket).serve(service).await;
    }

    Ok(())
}
