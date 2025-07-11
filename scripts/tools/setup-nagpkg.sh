#!/bin/bash

# Nagari Package Manager Setup Script
# This script sets up the nagpkg package manager infrastructure

set -e

echo "🚀 Setting up Nagari Package Manager (nagpkg)..."

# Configuration
NAGARI_HOME="${NAGARI_HOME:-$HOME/.nagari}"
NAGPKG_CACHE="${NAGPKG_CACHE:-$NAGARI_HOME/cache}"
NAGPKG_REGISTRY="${NAGPKG_REGISTRY:-https://registry.nagari.dev}"
NAGPKG_CONFIG_FILE="$NAGARI_HOME/nagpkg.toml"

# Create directories
echo "📁 Creating directory structure..."
mkdir -p "$NAGARI_HOME"
mkdir -p "$NAGPKG_CACHE/packages"
mkdir -p "$NAGPKG_CACHE/tarballs"
mkdir -p "$NAGPKG_CACHE/metadata"
mkdir -p "$NAGPKG_CACHE/temp"
mkdir -p "$NAGARI_HOME/config"
mkdir -p "$NAGARI_HOME/sessions"
mkdir -p "$NAGARI_HOME/logs"

# Create nagpkg configuration file
echo "⚙️  Creating nagpkg configuration..."
cat > "$NAGPKG_CONFIG_FILE" << EOF
# Nagari Package Manager Configuration

[registry]
# Default registry URL
default = "$NAGPKG_REGISTRY"

# Alternative registries
[registry.sources]
# nagari = "https://registry.nagari.dev"
# local = "http://localhost:4873"

[cache]
# Cache directory
dir = "$NAGPKG_CACHE"

# Cache settings
max_size_mb = 1024
max_age_days = 30
prune_on_startup = false

[security]
# Package integrity verification
verify_integrity = true
verify_signatures = false

# Trusted publishers
trusted_publishers = []

[network]
# Network timeout in seconds
timeout = 30

# Retry settings
max_retries = 3
retry_delay_ms = 1000

# Proxy settings (optional)
# proxy = "http://proxy.example.com:8080"
# no_proxy = ["localhost", "127.0.0.1"]

[publishing]
# Default access level for published packages
default_access = "public"

# Package validation settings
require_readme = true
require_license = true
require_description = true

[workspace]
# Workspace settings
auto_detect = true
hoist_dependencies = true
save_exact = false

# Dev dependency handling
install_dev_by_default = false
EOF

# Create example nagari.json template
echo "📦 Creating package template..."
cat > "$NAGARI_HOME/package-template.json" << EOF
{
  "name": "my-nagari-package",
  "version": "1.0.0",
  "description": "A Nagari package",
  "main": "src/main.nag",
  "scripts": {
    "build": "nag build",
    "test": "nag test",
    "dev": "nag run --watch",
    "lint": "nag lint",
    "format": "nag format"
  },
  "keywords": [],
  "author": "",
  "license": "ISC",
  "dependencies": {},
  "devDependencies": {},
  "nagari": {
    "source_dir": "src",
    "output_dir": "dist",
    "target": "es2020",
    "module_format": "esm",
    "compiler_options": {
      "strict": true,
      "debug": false,
      "optimize": true,
      "emit_source_maps": true
    }
  }
}
EOF

# Create .nagignore template
cat > "$NAGARI_HOME/nagignore-template" << EOF
# Dependencies
node_modules/
.nagari-cache/

# Build output
dist/
build/
*.js.map

# Logs
*.log
npm-debug.log*
yarn-debug.log*
yarn-error.log*

# Environment variables
.env
.env.local
.env.development.local
.env.test.local
.env.production.local

# IDE/Editor files
.vscode/
.idea/
*.swp
*.swo
*~

# OS generated files
.DS_Store
.DS_Store?
._*
.Spotlight-V100
.Trashes
ehthumbs.db
Thumbs.db

# Temporary files
tmp/
temp/
*.tmp
*.temp

# Test coverage
coverage/
.nyc_output/

# Package manager lockfiles (optional - choose one)
# nag.lock
# package-lock.json
# yarn.lock
EOF

# Create initial cache metadata
echo "💾 Initializing cache metadata..."
cat > "$NAGPKG_CACHE/cache-metadata.json" << EOF
{
  "packages": {},
  "integrity_checks": {},
  "access_times": {}
}
EOF

# Set up logging configuration
echo "📊 Setting up logging..."
cat > "$NAGARI_HOME/logging.toml" << EOF
[logging]
level = "info"
file = "$NAGARI_HOME/logs/nagpkg.log"
max_size_mb = 10
max_files = 5
console_output = true

[logging.modules]
# Module-specific log levels
nagpkg = "info"
registry = "info"
resolver = "debug"
cache = "info"
EOF

# Create registry client configuration
echo "🌐 Setting up registry client..."
cat > "$NAGARI_HOME/registry.toml" << EOF
[registry]
url = "$NAGPKG_REGISTRY"
timeout = 30

[auth]
# Authentication token (will be set by 'nag package login')
token = ""

[features]
# Registry feature support
search = true
publish = true
unpublish = true
deprecate = true
statistics = true
EOF

# Create sample workspace configuration
echo "🏢 Creating workspace template..."
mkdir -p "$NAGARI_HOME/templates/workspace"
cat > "$NAGARI_HOME/templates/workspace/nagari-workspace.json" << EOF
{
  "name": "my-nagari-workspace",
  "version": "1.0.0",
  "private": true,
  "workspaces": [
    "packages/*",
    "apps/*"
  ],
  "scripts": {
    "build": "nag build --workspace",
    "test": "nag test --workspace",
    "lint": "nag lint --workspace",
    "format": "nag format --workspace"
  },
  "devDependencies": {},
  "nagari": {
    "workspace": {
      "hoist_dependencies": true,
      "parallel_builds": true,
      "shared_config": true
    }
  }
}
EOF

# Create development tools configuration
echo "🔧 Setting up development tools..."
cat > "$NAGARI_HOME/tools.toml" << EOF
[formatter]
indent_size = 2
use_tabs = false
max_line_length = 100
trailing_commas = true
semicolons = false

[linter]
rules = "recommended"
max_warnings = 100
treat_warnings_as_errors = false

[compiler]
target = "es2020"
module_format = "esm"
source_maps = true
minify = false

[bundler]
entry_points = ["src/main.nag"]
output_dir = "dist"
format = "esm"
splitting = true
external = []
EOF

# Set up shell completion
echo "🐚 Setting up shell completion..."
mkdir -p "$NAGARI_HOME/completion"

# Bash completion
cat > "$NAGARI_HOME/completion/nag.bash" << 'EOF'
# Bash completion for nag CLI
_nag_completion() {
    local cur prev opts
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"

    # Main commands
    opts="run build transpile bundle format lint test repl doc package lsp init serve --help --version"

    if [[ ${cur} == -* ]] ; then
        COMPREPLY=( $(compgen -W "${opts}" -- ${cur}) )
        return 0
    fi

    case "${prev}" in
        package)
            local package_opts="init install uninstall update list search info publish unpublish login logout"
            COMPREPLY=( $(compgen -W "${package_opts}" -- ${cur}) )
            return 0
            ;;
        repl)
            local repl_opts="--script --load --save --session"
            COMPREPLY=( $(compgen -W "${repl_opts}" -- ${cur}) )
            return 0
            ;;
        *)
            COMPREPLY=( $(compgen -W "${opts}" -- ${cur}) )
            return 0
            ;;
    esac
}

complete -F _nag_completion nag
EOF

# Zsh completion
cat > "$NAGARI_HOME/completion/nag.zsh" << 'EOF'
#compdef nag

_nag() {
    local context state line
    typeset -A opt_args

    _arguments \
        '1: :->command' \
        '*: :->args'

    case $state in
        command)
            _values 'command' \
                'run[Run a Nagari file]' \
                'build[Build/compile Nagari code]' \
                'transpile[Transpile to JavaScript]' \
                'bundle[Bundle for distribution]' \
                'format[Format source code]' \
                'lint[Lint source code]' \
                'test[Run tests]' \
                'repl[Start interactive REPL]' \
                'doc[Generate documentation]' \
                'package[Package management]' \
                'lsp[Language Server Protocol]' \
                'init[Initialize project]' \
                'serve[Development server]'
            ;;
        args)
            case $words[2] in
                package)
                    _values 'package command' \
                        'init[Initialize package]' \
                        'install[Install packages]' \
                        'uninstall[Uninstall packages]' \
                        'update[Update packages]' \
                        'list[List packages]' \
                        'search[Search packages]' \
                        'info[Package information]' \
                        'publish[Publish package]' \
                        'unpublish[Unpublish package]' \
                        'login[Login to registry]' \
                        'logout[Logout from registry]'
                    ;;
            esac
            ;;
    esac
}

_nag "$@"
EOF

# Fish completion
cat > "$NAGARI_HOME/completion/nag.fish" << 'EOF'
# Fish completion for nag CLI

complete -c nag -n "__fish_use_subcommand" -a "run" -d "Run a Nagari file"
complete -c nag -n "__fish_use_subcommand" -a "build" -d "Build/compile Nagari code"
complete -c nag -n "__fish_use_subcommand" -a "transpile" -d "Transpile to JavaScript"
complete -c nag -n "__fish_use_subcommand" -a "bundle" -d "Bundle for distribution"
complete -c nag -n "__fish_use_subcommand" -a "format" -d "Format source code"
complete -c nag -n "__fish_use_subcommand" -a "lint" -d "Lint source code"
complete -c nag -n "__fish_use_subcommand" -a "test" -d "Run tests"
complete -c nag -n "__fish_use_subcommand" -a "repl" -d "Start interactive REPL"
complete -c nag -n "__fish_use_subcommand" -a "doc" -d "Generate documentation"
complete -c nag -n "__fish_use_subcommand" -a "package" -d "Package management"
complete -c nag -n "__fish_use_subcommand" -a "lsp" -d "Language Server Protocol"
complete -c nag -n "__fish_use_subcommand" -a "init" -d "Initialize project"
complete -c nag -n "__fish_use_subcommand" -a "serve" -d "Development server"

# Package subcommands
complete -c nag -n "__fish_seen_subcommand_from package" -a "init" -d "Initialize package"
complete -c nag -n "__fish_seen_subcommand_from package" -a "install" -d "Install packages"
complete -c nag -n "__fish_seen_subcommand_from package" -a "uninstall" -d "Uninstall packages"
complete -c nag -n "__fish_seen_subcommand_from package" -a "update" -d "Update packages"
complete -c nag -n "__fish_seen_subcommand_from package" -a "list" -d "List packages"
complete -c nag -n "__fish_seen_subcommand_from package" -a "search" -d "Search packages"
complete -c nag -n "__fish_seen_subcommand_from package" -a "info" -d "Package information"
complete -c nag -n "__fish_seen_subcommand_from package" -a "publish" -d "Publish package"
complete -c nag -n "__fish_seen_subcommand_from package" -a "unpublish" -d "Unpublish package"
complete -c nag -n "__fish_seen_subcommand_from package" -a "login" -d "Login to registry"
complete -c nag -n "__fish_seen_subcommand_from package" -a "logout" -d "Logout from registry"
EOF

# Create environment setup script
echo "🌍 Creating environment setup..."
cat > "$NAGARI_HOME/setup-env.sh" << EOF
#!/bin/bash

# Nagari Environment Setup
export NAGARI_HOME="$NAGARI_HOME"
export NAGPKG_CACHE="$NAGPKG_CACHE"
export NAGPKG_REGISTRY="$NAGPKG_REGISTRY"

# Add to PATH if nag is not already available
if ! command -v nag &> /dev/null; then
    echo "Warning: 'nag' command not found in PATH"
    echo "Please install the Nagari CLI or add it to your PATH"
fi

# Load shell completion
case "\$0" in
    *bash*)
        source "$NAGARI_HOME/completion/nag.bash"
        ;;
    *zsh*)
        source "$NAGARI_HOME/completion/nag.zsh"
        ;;
    *fish*)
        source "$NAGARI_HOME/completion/nag.fish"
        ;;
esac

echo "Nagari environment loaded!"
echo "  NAGARI_HOME: \$NAGARI_HOME"
echo "  NAGPKG_CACHE: \$NAGPKG_CACHE"
echo "  NAGPKG_REGISTRY: \$NAGPKG_REGISTRY"
EOF

chmod +x "$NAGARI_HOME/setup-env.sh"

# Create cleanup script
echo "🧹 Creating cleanup script..."
cat > "$NAGARI_HOME/cleanup.sh" << EOF
#!/bin/bash

# Nagari Package Manager Cleanup Script

echo "🧹 Cleaning up Nagari package manager..."

# Clean cache
echo "Cleaning package cache..."
rm -rf "$NAGPKG_CACHE/packages"/*
rm -rf "$NAGPKG_CACHE/tarballs"/*
rm -rf "$NAGPKG_CACHE/temp"/*

# Reset cache metadata
echo '{"packages": {}, "integrity_checks": {}, "access_times": {}}' > "$NAGPKG_CACHE/cache-metadata.json"

# Clean old logs
echo "Cleaning old logs..."
find "$NAGARI_HOME/logs" -name "*.log" -mtime +7 -delete 2>/dev/null || true

echo "✅ Cleanup completed!"
EOF

chmod +x "$NAGARI_HOME/cleanup.sh"

# Create health check script
echo "🏥 Creating health check script..."
cat > "$NAGARI_HOME/health-check.sh" << EOF
#!/bin/bash

# Nagari Package Manager Health Check

echo "🏥 Nagari Package Manager Health Check"
echo "======================================"

# Check directories
echo "📁 Directory structure:"
for dir in "$NAGARI_HOME" "$NAGPKG_CACHE" "$NAGPKG_CACHE/packages" "$NAGPKG_CACHE/tarballs" "$NAGPKG_CACHE/metadata"; do
    if [ -d "\$dir" ]; then
        echo "  ✅ \$dir"
    else
        echo "  ❌ \$dir (missing)"
    fi
done

# Check configuration files
echo ""
echo "⚙️  Configuration files:"
for file in "$NAGPKG_CONFIG_FILE" "$NAGARI_HOME/logging.toml" "$NAGARI_HOME/registry.toml"; do
    if [ -f "\$file" ]; then
        echo "  ✅ \$file"
    else
        echo "  ❌ \$file (missing)"
    fi
done

# Check cache
echo ""
echo "💾 Cache status:"
if [ -f "$NAGPKG_CACHE/cache-metadata.json" ]; then
    cache_size=\$(du -sh "$NAGPKG_CACHE" 2>/dev/null | cut -f1)
    package_count=\$(find "$NAGPKG_CACHE/packages" -type d -mindepth 1 2>/dev/null | wc -l)
    echo "  ✅ Cache metadata exists"
    echo "  📊 Cache size: \$cache_size"
    echo "  📦 Cached packages: \$package_count"
else
    echo "  ❌ Cache metadata missing"
fi

# Check nag command
echo ""
echo "🔧 CLI tool:"
if command -v nag &> /dev/null; then
    version=\$(nag --version 2>/dev/null || echo "unknown")
    echo "  ✅ nag command available (\$version)"
else
    echo "  ❌ nag command not found"
fi

# Check registry connectivity
echo ""
echo "🌐 Registry connectivity:"
if curl -s --max-time 5 "$NAGPKG_REGISTRY" > /dev/null 2>&1; then
    echo "  ✅ Registry accessible: $NAGPKG_REGISTRY"
else
    echo "  ⚠️  Registry not accessible: $NAGPKG_REGISTRY"
fi

echo ""
echo "Health check completed!"
EOF

chmod +x "$NAGARI_HOME/health-check.sh"

# Print installation summary
echo ""
echo "✅ Nagari Package Manager setup completed!"
echo ""
echo "📍 Installation details:"
echo "  NAGARI_HOME: $NAGARI_HOME"
echo "  Cache directory: $NAGPKG_CACHE"
echo "  Registry: $NAGPKG_REGISTRY"
echo "  Configuration: $NAGPKG_CONFIG_FILE"
echo ""
echo "🚀 Next steps:"
echo "  1. Source the environment: source $NAGARI_HOME/setup-env.sh"
echo "  2. Run health check: $NAGARI_HOME/health-check.sh"
echo "  3. Initialize a project: nag package init"
echo "  4. Install packages: nag package install <package-name>"
echo ""
echo "📚 Documentation:"
echo "  - Package manager guide: docs/nagpkg-design.md"
echo "  - CLI reference: nag package --help"
echo "  - Configuration: $NAGPKG_CONFIG_FILE"
echo ""
echo "🆘 Support:"
echo "  - Cleanup: $NAGARI_HOME/cleanup.sh"
echo "  - Health check: $NAGARI_HOME/health-check.sh"
echo "  - Logs: $NAGARI_HOME/logs/"
