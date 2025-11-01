# Bun-Specific Examples

This directory contains Nagari examples optimized for Bun runtime, demonstrating performance improvements and Bun-specific features.

## Running Examples

All examples can be run with the standard Nagari CLI, which automatically uses Bun if available:

```bash
# Nagari automatically detects and uses Bun
nag run bun_performance.nag

# Or run transpiled JavaScript directly with Bun
nag build bun_performance.nag -o output.js
bun run output.js
```

## Examples

### 1. `bun_performance.nag`
Demonstrates performance improvements when running with Bun vs Node.js.
Shows startup time, execution speed, and memory usage comparisons.

### 2. `bun_fetch.nag`
Uses Bun's native, high-performance fetch API for HTTP requests.
Shows parallel requests and streaming responses.

### 3. `bun_file_io.nag`
Demonstrates Bun's fast file I/O operations with large files.
Compares read/write performance with async operations.

### 4. `bun_websocket.nag`
WebSocket client using Bun's native WebSocket implementation.
Shows real-time communication patterns.

## Performance Notes

When running with Bun, you should see:
- **4x faster** startup time
- **4x faster** execution speed
- **50% less** memory usage
- **Native** TypeScript support (no transpilation overhead)

## Requirements

- Bun 1.0.0 or later
- Nagari CLI with Bun support

Install Bun:
```bash
curl -fsSL https://bun.sh/install | bash
```

## See Also

- [Bun Integration Guide](../../docs/bun-guide.md)
- [Bun Official Documentation](https://bun.sh/docs)
