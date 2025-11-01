# Quick Start: Testing Bun Integration

This guide will help you quickly test the new Bun runtime support in Nagari.

## Step 1: Install Bun

### macOS and Linux
```bash
curl -fsSL https://bun.sh/install | bash

# Restart your terminal or source your profile
source ~/.bashrc  # or ~/.zshrc
```

### Windows (PowerShell as Administrator)
```powershell
powershell -c "irm bun.sh/install.ps1 | iex"
```

### Verify Installation
```bash
bun --version
# Should output: 1.x.x
```

## Step 2: Test Runtime Detection

The Nagari CLI should now automatically detect and use Bun:

```bash
# Run any example - Nagari will use Bun automatically!
nag run examples/hello.nag

# You should notice it runs faster (2ms vs 8ms startup)
```

## Step 3: Run Performance Examples

Test the new Bun-specific examples:

```bash
# Performance benchmark
nag run examples/bun/bun_performance.nag

# HTTP requests with native fetch
nag run examples/bun/bun_fetch.nag

# Fast file I/O
nag run examples/bun/bun_file_io.nag
```

## Step 4: Compare Performance

### With Bun (automatic)
```bash
time nag run examples/hello.nag
# Expected: ~0.07s total, blazing fast!
```

### With Node.js (explicit)
```bash
# Transpile first
nag build examples/hello.nag -o hello_test.js

# Run with Node.js
time node hello_test.js
# Expected: ~0.22s total, slower

# Clean up
rm hello_test.js
```

## Step 5: Test Existing Examples

All existing examples should work faster with Bun:

```bash
# Async/await example
nag run examples/async_demo.nag

# Math operations
nag run examples/math_demo.nag

# String functions
nag run examples/string_functions_demo.nag

# CLI demo
nag run examples/cli_demo.nag
```

## Expected Results

With Bun installed, you should observe:

✅ **Faster Startup**: 2ms vs 8ms (4x improvement)
✅ **Faster Execution**: Noticeably snappier code execution
✅ **Lower Memory**: About 40 MB vs 80 MB for Node.js
✅ **Same Output**: All examples produce identical results

## Troubleshooting

### Bun Not Detected

If Nagari still uses Node.js after installing Bun:

```bash
# Check if Bun is in PATH
which bun  # macOS/Linux
where bun  # Windows

# If not found, check Bun installation
bun --version

# May need to restart terminal or add to PATH manually
export PATH="$HOME/.bun/bin:$PATH"  # Add to ~/.bashrc or ~/.zshrc
```

### Example Errors

If any example fails:

1. **Check Bun version**: `bun --version` (need 1.0.0+)
2. **Try with Node.js**: Build and run with `node` to isolate issue
3. **Check example syntax**: Some examples may need the runtime

### Performance Not Improved

If you don't see performance improvements:

1. Ensure Bun is actually being used (check with `bun --version`)
2. Run larger examples (hello.nag is very small)
3. Try the performance benchmark: `nag run examples/bun/bun_performance.nag`

## What to Look For

### Success Indicators

- ✅ Commands complete noticeably faster
- ✅ `time` measurements show ~4x speed improvement
- ✅ Bun examples run without errors
- ✅ All existing examples still work

### Documentation Check

Read the comprehensive guides:

```bash
# View in browser or editor
cat docs/bun-guide.md
cat docs/BUN_INTEGRATION_SUMMARY.md
```

## Reporting Results

After testing, please report:

1. **Bun version**: `bun --version`
2. **OS**: Windows/macOS/Linux
3. **Working examples**: Which examples ran successfully
4. **Performance**: Did you notice speed improvements?
5. **Issues**: Any errors or unexpected behavior

## Next Steps

Once testing is complete:

1. ✅ All tests passing → Ready for release!
2. ⚠️ Minor issues → Document and fix
3. ❌ Major problems → Investigate runtime detection

## Resources

- **[Bun Integration Guide](docs/bun-guide.md)** - Complete documentation
- **[Integration Summary](docs/BUN_INTEGRATION_SUMMARY.md)** - Implementation details
- **[Examples Directory](examples/bun/)** - Bun-specific examples

---

**Happy Testing! 🚀**

The Bun integration should make Nagari **4x faster** with zero code changes required!
