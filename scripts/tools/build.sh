#!/bin/bash

# Nagari Build Script
# Builds the compiler, runtime, and all advanced features

set -e

echo "Building Nagari Language Tools..."

# Build compiler
echo "Building compiler..."
cd nagari-compiler
cargo build --release
cd ..

# Build VM
echo "Building VM..."
cd nagari-vm
cargo build --release
cd ..

# Build runtime
echo "Building runtime..."
cd nagari-runtime
npm install
npm run build
cd ..

# Build CLI
echo "Building CLI..."
cd cli
cargo build --release
cd ..

# Build LSP server
echo "Building LSP server..."
cd lsp-server
cargo build --release
cd ..

# Build registry server
echo "Building registry server..."
cd registry-server
cargo build --release
cd ..

# Build advanced features
echo "Building WebAssembly runtime..."
./tools/build-wasm.sh

echo "Building embedded runtime..."
./tools/build-embedded.sh

echo "Build completed successfully!"
echo ""
echo "Core binaries located at:"
echo "  CLI Tool:        cli/target/release/nagari"
echo "  Compiler:        nagari-compiler/target/release/nagc"
echo "  VM:              nagari-vm/target/release/nagari-vm"
echo "  LSP Server:      lsp-server/target/release/nagari-lsp"
echo "  Registry Server: registry-server/target/release/nagari-registry"
echo "  Runtime:         nagari-runtime/dist/"
echo ""
echo "WebAssembly packages:"
echo "  Browser:   nagari-wasm/pkg/"
echo "  React:     nagari-wasm/pkg/react/"
echo ""
echo "Embedded runtimes:"
echo "  Python:    nagari-embedded/target/wheels/"
echo "  Node.js:   nagari-embedded/target/release/"
echo "  C Library: nagari-embedded/target/release/libnagari_embedded.a"
echo ""
echo "Add the CLI tool to your PATH to use globally."
