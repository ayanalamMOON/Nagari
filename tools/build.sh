#!/bin/bash

# Nagari Build Script
# Builds the compiler and runtime

set -e

echo "Building Nagari Language Tools..."

# Build compiler
echo "Building compiler..."
cd nagari-compiler
cargo build --release
cd ..

# Build runtime
echo "Building runtime..."
cd nagari-runtime
npm install
npm run build
cd ..

echo "Build completed successfully!"
echo ""
echo "Binaries located at:"
echo "  Compiler: nagari-compiler/target/release/nagc"
echo "  Runtime:  nagari-runtime/dist/"
echo ""
echo "Add the compiler to your PATH to use globally."
