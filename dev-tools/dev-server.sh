#!/bin/bash

# Nagari Development Server
# Hot-reload development server for rapid iteration

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WATCH_DIRS=("src" "stdlib" "examples" "tests")
BUILD_TARGET="debug"
PORT=3000
WEBSOCKET_PORT=3001

print_header() {
    echo -e "${BLUE}================================${NC}"
    echo -e "${BLUE}  Nagari Development Server${NC}"
    echo -e "${BLUE}================================${NC}"
    echo
}

print_info() {
    echo -e "${CYAN}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_build() {
    echo -e "${PURPLE}[BUILD]${NC} $1"
}

cleanup() {
    echo
    print_info "Shutting down development server..."

    # Kill background processes
    if [ ! -z "$CARGO_WATCH_PID" ]; then
        kill $CARGO_WATCH_PID 2>/dev/null || true
    fi

    if [ ! -z "$FILE_SERVER_PID" ]; then
        kill $FILE_SERVER_PID 2>/dev/null || true
    fi

    if [ ! -z "$WEBSOCKET_PID" ]; then
        kill $WEBSOCKET_PID 2>/dev/null || true
    fi

    print_success "Development server stopped"
    exit 0
}

# Trap cleanup on exit
trap cleanup SIGINT SIGTERM EXIT

check_dependencies() {
    print_info "Checking dependencies..."

    if ! command -v cargo &> /dev/null; then
        print_error "Cargo not found. Please install Rust."
        exit 1
    fi

    if ! command -v node &> /dev/null; then
        print_warning "Node.js not found. Some features may be limited."
    fi

    # Check for cargo-watch
    if ! command -v cargo-watch &> /dev/null; then
        print_info "Installing cargo-watch..."
        cargo install cargo-watch
    fi

    print_success "Dependencies check passed"
}

build_project() {
    print_build "Building project..."

    cd "$PROJECT_ROOT"

    # Build the main project
    if cargo build --target-dir target; then
        print_success "Build completed successfully"
        return 0
    else
        print_error "Build failed"
        return 1
    fi
}

start_file_watcher() {
    print_info "Starting file watcher..."

    cd "$PROJECT_ROOT"

    # Start cargo watch for Rust files
    cargo watch \
        --watch src \
        --watch stdlib \
        --watch Cargo.toml \
        --clear \
        --exec "check" \
        --exec "test" \
        --shell 'echo "🔄 Rebuilding..." && cargo build 2>&1' &

    CARGO_WATCH_PID=$!

    print_success "File watcher started (PID: $CARGO_WATCH_PID)"
}

start_example_runner() {
    print_info "Starting example runner..."

    # Create a simple web server for serving examples
    if command -v python3 &> /dev/null; then
        cd "$PROJECT_ROOT"
        python3 -m http.server $PORT --directory . &
        FILE_SERVER_PID=$!
        print_success "Example server started on http://localhost:$PORT"
    elif command -v python &> /dev/null; then
        cd "$PROJECT_ROOT"
        python -m SimpleHTTPServer $PORT &
        FILE_SERVER_PID=$!
        print_success "Example server started on http://localhost:$PORT"
    else
        print_warning "Python not found. Example server not started."
    fi
}

start_live_reload() {
    print_info "Starting live reload server..."

    # Simple WebSocket server for live reload
    if command -v node &> /dev/null; then
        cat > "$PROJECT_ROOT/live-reload-server.js" << 'EOF'
const WebSocket = require('ws');
const fs = require('fs');
const path = require('path');

const wss = new WebSocket.Server({ port: process.argv[2] || 3001 });
const watchDirs = ['src', 'stdlib', 'examples', 'tests'];

console.log('Live reload server started on port', process.argv[2] || 3001);

let clients = [];

wss.on('connection', (ws) => {
    clients.push(ws);
    console.log('Client connected. Total clients:', clients.length);

    ws.on('close', () => {
        clients = clients.filter(client => client !== ws);
        console.log('Client disconnected. Total clients:', clients.length);
    });
});

// Watch for file changes
watchDirs.forEach(dir => {
    if (fs.existsSync(dir)) {
        fs.watch(dir, { recursive: true }, (eventType, filename) => {
            if (filename && (filename.endsWith('.rs') || filename.endsWith('.nag'))) {
                console.log('File changed:', filename);
                clients.forEach(client => {
                    if (client.readyState === WebSocket.OPEN) {
                        client.send(JSON.stringify({ type: 'reload', file: filename }));
                    }
                });
            }
        });
    }
});
EOF

        node "$PROJECT_ROOT/live-reload-server.js" $WEBSOCKET_PORT &
        WEBSOCKET_PID=$!
        print_success "Live reload server started on port $WEBSOCKET_PORT"
    else
        print_warning "Node.js not found. Live reload not available."
    fi
}

create_dev_dashboard() {
    print_info "Creating development dashboard..."

    cat > "$PROJECT_ROOT/dev-dashboard.html" << 'EOF'
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Nagari Development Dashboard</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            margin: 0;
            padding: 20px;
            background: #1e1e1e;
            color: #fff;
        }
        .header {
            text-align: center;
            margin-bottom: 30px;
        }
        .grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 20px;
            max-width: 1200px;
            margin: 0 auto;
        }
        .card {
            background: #2d2d2d;
            border-radius: 8px;
            padding: 20px;
            border: 1px solid #404040;
        }
        .card h3 {
            margin-top: 0;
            color: #4CAF50;
        }
        .status {
            display: inline-block;
            padding: 4px 8px;
            border-radius: 4px;
            font-size: 12px;
            font-weight: bold;
        }
        .status.online { background: #4CAF50; color: white; }
        .status.offline { background: #f44336; color: white; }
        .button {
            display: inline-block;
            padding: 8px 16px;
            background: #2196F3;
            color: white;
            text-decoration: none;
            border-radius: 4px;
            margin: 5px;
        }
        .button:hover {
            background: #1976D2;
        }
        .log {
            background: #1a1a1a;
            border: 1px solid #404040;
            border-radius: 4px;
            padding: 10px;
            font-family: monospace;
            font-size: 12px;
            max-height: 200px;
            overflow-y: auto;
        }
        .log-entry {
            margin: 2px 0;
        }
        .log-entry.info { color: #2196F3; }
        .log-entry.success { color: #4CAF50; }
        .log-entry.warning { color: #FF9800; }
        .log-entry.error { color: #f44336; }
    </style>
</head>
<body>
    <div class="header">
        <h1>🚀 Nagari Development Dashboard</h1>
        <p>Real-time development environment status</p>
    </div>

    <div class="grid">
        <div class="card">
            <h3>Build Status</h3>
            <div id="build-status">
                <span class="status online">Building...</span>
            </div>
            <div>
                <a href="#" class="button" onclick="triggerBuild()">Rebuild</a>
                <a href="#" class="button" onclick="runTests()">Run Tests</a>
            </div>
        </div>

        <div class="card">
            <h3>Examples</h3>
            <div>
                <a href="/examples/" class="button">Browse Examples</a>
                <a href="/samples/" class="button">View Samples</a>
                <a href="/docs/" class="button">Documentation</a>
            </div>
        </div>

        <div class="card">
            <h3>Tools</h3>
            <div>
                <a href="#" class="button" onclick="formatCode()">Format Code</a>
                <a href="#" class="button" onclick="runLint()">Run Lint</a>
                <a href="#" class="button" onclick="cleanBuild()">Clean Build</a>
            </div>
        </div>

        <div class="card">
            <h3>Live Reload</h3>
            <div id="reload-status">
                <span class="status offline">Connecting...</span>
            </div>
            <div id="last-change">No recent changes</div>
        </div>

        <div class="card" style="grid-column: 1 / -1;">
            <h3>Development Log</h3>
            <div id="log" class="log">
                <div class="log-entry info">Development server started</div>
            </div>
        </div>
    </div>

    <script>
        // Live reload WebSocket connection
        let ws = null;

        function connectWebSocket() {
            try {
                ws = new WebSocket('ws://localhost:3001');

                ws.onopen = function() {
                    document.getElementById('reload-status').innerHTML =
                        '<span class="status online">Connected</span>';
                    addLogEntry('WebSocket connected', 'success');
                };

                ws.onmessage = function(event) {
                    const data = JSON.parse(event.data);
                    if (data.type === 'reload') {
                        document.getElementById('last-change').textContent =
                            `Last change: ${data.file}`;
                        addLogEntry(`File changed: ${data.file}`, 'info');
                    }
                };

                ws.onclose = function() {
                    document.getElementById('reload-status').innerHTML =
                        '<span class="status offline">Disconnected</span>';
                    addLogEntry('WebSocket disconnected', 'warning');

                    // Reconnect after 5 seconds
                    setTimeout(connectWebSocket, 5000);
                };

                ws.onerror = function() {
                    addLogEntry('WebSocket error', 'error');
                };
            } catch (e) {
                addLogEntry('Failed to connect to live reload server', 'error');
                setTimeout(connectWebSocket, 5000);
            }
        }

        function addLogEntry(message, type = 'info') {
            const log = document.getElementById('log');
            const entry = document.createElement('div');
            entry.className = `log-entry ${type}`;
            entry.textContent = `[${new Date().toLocaleTimeString()}] ${message}`;
            log.appendChild(entry);
            log.scrollTop = log.scrollHeight;
        }

        function triggerBuild() {
            addLogEntry('Build triggered', 'info');
            // In a real implementation, this would trigger a build via API
        }

        function runTests() {
            addLogEntry('Tests started', 'info');
            // In a real implementation, this would run tests via API
        }

        function formatCode() {
            addLogEntry('Code formatting triggered', 'info');
        }

        function runLint() {
            addLogEntry('Linting started', 'info');
        }

        function cleanBuild() {
            addLogEntry('Clean build triggered', 'info');
        }

        // Initialize
        connectWebSocket();

        // Auto-refresh build status every 10 seconds
        setInterval(() => {
            // In a real implementation, this would check build status via API
        }, 10000);
    </script>
</body>
</html>
EOF

    print_success "Development dashboard created at http://localhost:$PORT/dev-dashboard.html"
}

show_status() {
    echo
    print_success "🚀 Nagari Development Server is running!"
    echo
    echo -e "${YELLOW}Available URLs:${NC}"
    echo -e "  • Dashboard:     ${BLUE}http://localhost:$PORT/dev-dashboard.html${NC}"
    echo -e "  • Examples:      ${BLUE}http://localhost:$PORT/examples/${NC}"
    echo -e "  • Samples:       ${BLUE}http://localhost:$PORT/samples/${NC}"
    echo -e "  • Documentation: ${BLUE}http://localhost:$PORT/docs/${NC}"
    echo
    echo -e "${YELLOW}Services:${NC}"
    echo -e "  • File Watcher:  ${GREEN}Active${NC} (Cargo Watch)"
    if [ ! -z "$FILE_SERVER_PID" ]; then
        echo -e "  • File Server:   ${GREEN}Active${NC} (Port $PORT)"
    fi
    if [ ! -z "$WEBSOCKET_PID" ]; then
        echo -e "  • Live Reload:   ${GREEN}Active${NC} (Port $WEBSOCKET_PORT)"
    fi
    echo
    echo -e "${YELLOW}Press Ctrl+C to stop the development server${NC}"
    echo
}

main() {
    print_header
    check_dependencies

    cd "$PROJECT_ROOT"

    # Initial build
    if ! build_project; then
        print_warning "Initial build failed, but continuing..."
    fi

    # Start services
    start_file_watcher
    start_example_runner
    start_live_reload
    create_dev_dashboard

    # Show status
    show_status

    # Keep the script running
    while true; do
        sleep 1
    done
}

# Help text
if [[ "$1" == "--help" || "$1" == "-h" ]]; then
    echo "Nagari Development Server"
    echo
    echo "A comprehensive development server with hot-reload, file watching,"
    echo "and a web-based dashboard for rapid Nagari development."
    echo
    echo "Usage: $0 [options]"
    echo
    echo "Options:"
    echo "  --port PORT        Set file server port (default: 3000)"
    echo "  --ws-port PORT     Set WebSocket port (default: 3001)"
    echo "  --target TARGET    Set build target (default: debug)"
    echo "  --help, -h         Show this help message"
    echo
    echo "Features:"
    echo "  • Automatic rebuilding on file changes"
    echo "  • Live reload for web-based examples"
    echo "  • Web dashboard for development tools"
    echo "  • File server for examples and documentation"
    echo "  • Real-time build status and logs"
    echo
    echo "The development server will start multiple services:"
    echo "  1. Cargo watch for automatic rebuilding"
    echo "  2. HTTP server for serving files"
    echo "  3. WebSocket server for live reload"
    echo "  4. Development dashboard"
    exit 0
fi

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --port)
            PORT="$2"
            shift 2
            ;;
        --ws-port)
            WEBSOCKET_PORT="$2"
            shift 2
            ;;
        --target)
            BUILD_TARGET="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

main "$@"
