# LSP Integration Guide

Language Server Protocol integration for enhanced Nagari development experience.

## Overview

The Nagari LSP (Language Server Protocol) provides intelligent code features in any LSP-compatible editor, including syntax highlighting, auto-completion, error detection, and more.

## Quick Setup

### VS Code (Recommended)

1. **Install the Extension:**
   ```bash
   code --install-extension nagari.nagari-lsp
   ```

2. **Automatic Configuration:**
   The extension automatically configures the LSP server for `.nag` files.

3. **Verify Setup:**
   Open a `.nag` file and check for:
   - Syntax highlighting
   - Error squiggles
   - Auto-completion (Ctrl+Space)

### Manual LSP Server

For other editors or custom setups:

```bash
# Start the LSP server
nagari lsp --stdio

# Or with TCP for debugging
nagari lsp --tcp 9257 --debug
```

## Editor Support

### VS Code

**Features Available:**
- ✅ Syntax highlighting
- ✅ Error diagnostics
- ✅ Auto-completion
- ✅ Go to definition
- ✅ Find references
- ✅ Symbol search
- ✅ Code formatting
- ✅ Hover information
- ✅ Inlay hints

**Configuration:**
```json
{
  "nagari.lsp.enabled": true,
  "nagari.lsp.serverPath": "nagari-lsp",
  "nagari.lsp.debug": false,
  "nagari.lsp.trace": "off",
  "files.associations": {
    "*.nag": "nagari"
  },
  "editor.tabSize": 2,
  "editor.insertSpaces": true
}
```

### Neovim

**LSP Configuration:**
```lua
require('lspconfig').nagari.setup({
  cmd = {'nagari', 'lsp', '--stdio'},
  filetypes = {'nagari'},
  root_dir = require('lspconfig.util').root_pattern('nagari.toml', '.git'),
  settings = {
    nagari = {
      lsp = {
        enabled = true,
        debug = false
      }
    }
  }
})
```

**File Type Detection:**
```lua
vim.filetype.add({
  extension = {
    nag = 'nagari'
  }
})
```

### Emacs

**LSP Mode Configuration:**
```elisp
(require 'lsp-mode)

(add-to-list 'lsp-language-id-configuration '(nagari-mode . "nagari"))

(lsp-register-client
 (make-lsp-client :new-connection (lsp-stdio-connection '("nagari" "lsp" "--stdio"))
                  :major-modes '(nagari-mode)
                  :server-id 'nagari-lsp))

(add-hook 'nagari-mode-hook #'lsp)
```

### Sublime Text

**LSP-nagari Package Configuration:**
```json
{
  "clients": {
    "nagari": {
      "enabled": true,
      "command": ["nagari", "lsp", "--stdio"],
      "selector": "source.nagari",
      "schemes": ["file"]
    }
  }
}
```

### Vim/Vim-Plug

**ALE Configuration:**
```vim
let g:ale_linters = {
\   'nagari': ['nagari-lsp'],
\}

let g:ale_nagari_nagari_lsp_executable = 'nagari'
let g:ale_nagari_nagari_lsp_options = 'lsp --stdio'
```

## Features in Detail

### Syntax Highlighting

The LSP provides semantic highlighting for:
- Keywords (`let`, `const`, `function`, `async`, `await`)
- Operators (`+`, `-`, `*`, `/`, `&&`, `||`)
- Literals (strings, numbers, booleans)
- Comments and documentation
- Identifiers and function names

### Auto-completion

**Variable Completion:**
```nagari
let userName = "Alice"
let user// <- Ctrl+Space shows: userName, userAgent, etc.
```

**Function Completion:**
```nagari
import { fetch } from 'http'
fet// <- Shows: fetch with signature
```

**Method Completion:**
```nagari
let arr = [1, 2, 3]
arr.// <- Shows: map, filter, reduce, push, pop, etc.
```

### Error Diagnostics

**Syntax Errors:**
```nagari
function test( {  // Missing parameter
  return "test"
}
// Error: Expected parameter name
```

**Type Errors:**
```nagari
let num = 42
num.toUpperCase()  // Error: toUpperCase is not a function
```

**Runtime Errors:**
```nagari
console.log(undefinedVariable)  // Warning: Variable may be undefined
```

### Go to Definition

Navigate to symbol definitions:
- Function declarations
- Variable assignments
- Import sources
- Class definitions

**Usage:**
- VS Code: F12 or Ctrl+Click
- Neovim: `gd` with LSP
- Emacs: `M-.`

### Find References

Find all usages of a symbol:
- Variable references
- Function calls
- Import statements

**Usage:**
- VS Code: Shift+F12
- Neovim: `gr` with LSP
- Emacs: `M-?`

### Symbol Search

Workspace-wide symbol search:
- Functions
- Variables
- Classes
- Modules

**Usage:**
- VS Code: Ctrl+T
- Neovim: `:LspWorkspaceSymbol`
- Emacs: `C-c l s`

### Code Formatting

Automatic code formatting with customizable rules:

```nagari
// Before formatting
function test(){
let x=1+2;
return x;
}

// After formatting
function test() {
  let x = 1 + 2;
  return x;
}
```

### Hover Information

Rich hover tooltips showing:
- Type information
- Documentation
- Function signatures
- Value previews

### Inlay Hints

Inline code annotations:
- Parameter names in function calls
- Type annotations
- Return types

```nagari
calculateArea(/* width: */ 10, /* height: */ 20)
//            ^^^^^^^^^^      ^^^^^^^^^^^^
//            Inlay hints for parameter names
```

## Configuration

### LSP Server Options

```bash
# Basic server
nagari lsp --stdio

# With debug logging
nagari lsp --stdio --debug --log-file lsp.log

# TCP server for testing
nagari lsp --tcp 9257 --debug
```

### Project Configuration

**nagari.toml:**
```toml
[lsp]
enabled = true
debug = false
completion = true
diagnostics = true
formatting = true
hover = true

[lsp.completion]
auto_import = true
snippets = true
documentation = true

[lsp.diagnostics]
severity = "warning"
debounce = 500

[lsp.formatting]
indent_size = 2
max_line_length = 100
trailing_newline = true
```

### Editor-Specific Settings

**VS Code settings.json:**
```json
{
  "nagari.lsp.completion.autoImport": true,
  "nagari.lsp.diagnostics.enable": true,
  "nagari.lsp.formatting.enable": true,
  "nagari.lsp.hover.enable": true,
  "nagari.lsp.inlayHints.enable": true,
  "nagari.lsp.references.enable": true
}
```

## Advanced Features

### Code Actions

Quick fixes and refactoring:
- Add missing imports
- Remove unused variables
- Extract functions
- Rename symbols

**Usage:**
- VS Code: Ctrl+. (Cmd+. on macOS)
- Neovim: `:lua vim.lsp.buf.code_action()`
- Emacs: `C-c l a`

### Workspace Symbols

Search across entire workspace:
```
@function calculateArea
@variable userName
@class HttpClient
```

### Call Hierarchy

Explore function call relationships:
- Incoming calls (who calls this function)
- Outgoing calls (what this function calls)

### Signature Help

Function parameter assistance:
```nagari
fetch(|)  // Shows: fetch(url: string, options?: RequestInit)
//    ^
//    Cursor position
```

## Debugging LSP Issues

### Enable Debug Mode

```bash
# Start LSP with debug logging
nagari lsp --stdio --debug --log-file nagari-lsp.log
```

### Check LSP Status

**VS Code:**
1. Open Command Palette (Ctrl+Shift+P)
2. Run "Developer: Reload Window"
3. Check "Output" panel for LSP logs

**Neovim:**
```vim
:LspInfo
:LspLog
```

### Common Issues

**LSP Server Not Starting:**
```bash
# Check Nagari installation
nagari --version

# Verify LSP command
nagari lsp --help

# Check file associations
file example.nag
```

**No Completions:**
- Verify project has `nagari.toml`
- Check file is saved
- Restart LSP server

**Slow Performance:**
- Reduce workspace size
- Disable unused features
- Increase debounce time

### Manual Testing

Test LSP server manually:
```bash
# Start server
nagari lsp --stdio --debug

# Send LSP requests (JSON-RPC 2.0)
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}
```

## Performance Optimization

### Large Workspaces

```toml
[lsp]
max_workspace_size = "100MB"
exclude_patterns = ["target/", "node_modules/", "*.log"]
index_timeout = 30000
```

### Memory Management

```toml
[lsp.memory]
max_cache_size = "64MB"
gc_interval = 300000  # 5 minutes
symbol_limit = 10000
```

### Network Optimization

For remote development:
```bash
# Use TCP with compression
nagari lsp --tcp 9257 --compress
```

## Integration Examples

### CI/CD Pipeline

```yaml
# .github/workflows/lint.yml
name: Lint
on: [push, pull_request]
jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install Nagari
        run: cargo install nagari
      - name: Check syntax
        run: nagari lsp --check src/
```

### Pre-commit Hook

```bash
#!/bin/sh
# .git/hooks/pre-commit
nagari lsp --check --format json src/ | jq -e '.errors | length == 0'
```

### Automated Formatting

```bash
# Format on save
nagari lsp --format src/**/*.nag --write
```

## Extending LSP

### Custom Diagnostics

Add project-specific linting rules:
```toml
[lsp.diagnostics.custom]
rules = ["no-console", "prefer-const", "max-complexity"]
```

### Custom Completions

Add workspace-specific completions:
```json
{
  "completions": {
    "api": ["fetch", "post", "get", "put", "delete"],
    "utils": ["formatDate", "validateEmail", "sanitize"]
  }
}
```

## Next Steps

- **[Getting Started](getting-started.md)** - Basic Nagari setup
- **[CLI Reference](cli-reference.md)** - Command-line tools
- **[API Reference](api-reference.md)** - Language documentation
- **[Troubleshooting](troubleshooting.md)** - Common solutions

---

*Maximize your productivity with intelligent Nagari development tools!*
