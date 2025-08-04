# Nagari LSP Connection Troubleshooting Guide

## Issue: "Nagari Language Server client: couldn't create connection to server"

This error indicates that VS Code cannot establish a connection to the Nagari Language Server. Here are the solutions:

## Solution 1: VS Code Extension Configuration

Since VS Code doesn't have a native Nagari extension yet, you need to configure LSP manually:

### Step 1: Install Generic LSP Extension
```bash
code --install-extension ms-vscode.vscode-languageserver-node
# OR
code --install-extension mattn.languageserver-settings
```

### Step 2: Configure VS Code Settings
Add to your `settings.json` (`.vscode/settings.json` in workspace or global settings):

```json
{
  "files.associations": {
    "*.nag": "nagari"
  },
  "languageserver": {
    "nagari": {
      "command": ["nag", "lsp", "--mode", "stdio"],
      "filetypes": ["nagari"],
      "rootPatterns": ["nagari.toml", "Cargo.toml", ".git"]
    }
  }
}
```

## Solution 2: Using Generic LSP Client Extension

### Install LSP Client
```bash
code --install-extension ms-vscode.vscode-languageserver-node
```

### Configure in settings.json
```json
{
  "files.associations": {
    "*.nag": "nagari"
  },
  "languageServerExample.maxNumberOfProblems": 100,
  "languageServerExample.trace.server": "verbose",
  "nagari.lsp.enabled": true,
  "nagari.lsp.serverPath": "nag",
  "nagari.lsp.serverArgs": ["lsp", "--mode", "stdio"],
  "nagari.lsp.trace": "verbose"
}
```

## Solution 3: Manual LSP Setup with CodeLLDB

### Create VSCode Task
Create `.vscode/tasks.json`:

```json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "Start Nagari LSP",
      "type": "shell",
      "command": "nag",
      "args": ["lsp", "--mode", "stdio"],
      "group": "build",
      "presentation": {
        "reveal": "always",
        "panel": "new"
      },
      "problemMatcher": []
    }
  ]
}
```

## Solution 4: Direct Binary Path

If the `nag` command is not in PATH:

```json
{
  "files.associations": {
    "*.nag": "nagari"
  },
  "languageserver": {
    "nagari": {
      "command": ["C:/Users/ayana/Projects/Nagari/target/debug/nag.exe", "lsp", "--mode", "stdio"],
      "filetypes": ["nagari"],
      "rootPatterns": ["nagari.toml", "Cargo.toml", ".git"]
    }
  }
}
```

## Verification Steps

### 1. Test LSP Server Manually
```bash
# Test if LSP server starts
nag lsp --mode stdio

# Test TCP mode for debugging
nag lsp --mode tcp --port 9257

# Test standalone server
nagari-lsp --stdio --debug
```

### 2. Check VS Code Output
1. Open VS Code
2. Go to View > Output
3. Select "Nagari" or "Language Server" from dropdown
4. Look for connection logs

### 3. Test with Sample File
Create a test file `test.nag`:

```nagari
function greet(name: string): string {
    return "Hello, " + name + "!";
}

let message = greet("World");
console.log(message);
```

## Advanced Debugging

### Enable Debug Logs
```json
{
  "nagari.lsp.trace": "verbose",
  "nagari.lsp.debug": true
}
```

### Check LSP Communication
```bash
# Start LSP with debug output
nag lsp --mode stdio --verbose > lsp.log 2>&1
```

### VS Code Developer Console
1. Help > Toggle Developer Tools
2. Console tab
3. Look for LSP-related errors

## Alternative: Using Neovim or Other Editors

If VS Code continues to have issues, try other editors:

### Neovim LSP Config
```lua
require('lspconfig').nagari.setup({
  cmd = {'nag', 'lsp', '--mode', 'stdio'},
  filetypes = {'nagari'},
  root_dir = require('lspconfig.util').root_pattern('nagari.toml', '.git'),
})
```

### Emacs LSP Config
```elisp
(require 'lsp-mode)
(add-to-list 'lsp-language-id-configuration '(nagari-mode . "nagari"))
(lsp-register-client
 (make-lsp-client :new-connection (lsp-stdio-connection '("nag" "lsp" "--mode" "stdio"))
                  :major-modes '(nagari-mode)
                  :server-id 'nagari-lsp))
```

## Current Status

✅ **Working Components:**
- Nagari Language Server (`nagari-lsp`) - Compiled and functional
- CLI LSP integration (`nag lsp`) - Working
- Code Actions implementation - Fixed and functional
- All LSP capabilities implemented (completion, hover, diagnostics, etc.)

❓ **Missing Component:**
- VS Code extension for automatic setup
- File type registration for `.nag` files

## Quick Fix: Create Simple VS Code Extension

You can create a minimal VS Code extension:

### package.json
```json
{
  "name": "nagari-language-support",
  "displayName": "Nagari Language Support",
  "version": "0.1.0",
  "engines": { "vscode": "^1.60.0" },
  "categories": ["Programming Languages"],
  "contributes": {
    "languages": [{
      "id": "nagari",
      "aliases": ["Nagari", "nagari"],
      "extensions": [".nag"],
      "configuration": "./language-configuration.json"
    }],
    "configurationDefaults": {
      "[nagari]": {
        "editor.tabSize": 2,
        "editor.insertSpaces": true
      }
    }
  },
  "activationEvents": ["onLanguage:nagari"],
  "main": "./out/extension.js"
}
```

This should resolve your LSP connection issues!
