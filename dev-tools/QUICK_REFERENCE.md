# 🚀 Nagari Development Tools - Quick Reference Card

## 🎯 One-Command Workflows

```bash
# Complete development setup + start server
./dev-tools/dev.sh dev

# Full quality check (lint + test)
./dev-tools/dev.sh check

# Ship ready (lint + test + release)
./dev-tools/dev.sh ship
```

## 📋 Individual Tools

| Command         | Purpose            | Quick Usage                         |
| --------------- | ------------------ | ----------------------------------- |
| `setup-dev-env` | Environment setup  | `./setup-dev-env.sh`                |
| `dev-server`    | Development server | `./dev-server.sh --port 3000`       |
| `test-runner`   | Test suite         | `./test-runner.sh --coverage`       |
| `lint-check`    | Code quality       | `./lint-check.sh --fix`             |
| `version-bump`  | Version management | `./version-bump.sh minor`           |
| `release-prep`  | Release packaging  | `./release-prep.sh --all-platforms` |

## 🔧 Master Launcher Commands

```bash
# Using the master dev.sh launcher:
./dev-tools/dev.sh [command] [options]

# Available commands:
setup      # Setup development environment
server     # Start development server with hot reload
test       # Run comprehensive test suite
lint       # Run code quality checks and formatting
version    # Bump version (major|minor|patch)
release    # Prepare release packages
build      # Build project (debug|release)
clean      # Clean build artifacts
watch      # Watch for changes and rebuild
status     # Show project status
help       # Show help message

# Quick workflows:
dev        # setup + server
check      # lint + test
ship       # lint + test + release
```

## 💡 Common Examples

```bash
# First time setup
./dev-tools/dev.sh setup

# Start development
./dev-tools/dev.sh dev

# Before committing
./dev-tools/dev.sh check

# Prepare for release
./dev-tools/dev.sh ship

# Custom port development server
./dev-tools/dev.sh server --port 4000

# Run tests with coverage
./dev-tools/dev.sh test --coverage

# Version bump
./dev-tools/dev.sh version minor

# Clean everything
./dev-tools/dev.sh clean
```

## 📂 Tool Organization

```
dev-tools/
├── dev.sh/bat          # 🎯 Master launcher (START HERE)
├── setup-dev-env.*     # 🔧 Environment setup
├── dev-server.*        # 🌐 Development server
├── test-runner.*       # 🧪 Testing suite
├── lint-check.sh       # ✨ Code quality
├── version-bump.sh     # 📦 Version management
├── release-prep.sh     # 🚀 Release preparation
├── config.json         # ⚙️  Configuration
└── README.md          # 📖 Full documentation
```

## 🔑 Key Features

- **Cross-Platform**: Both `.sh` and `.bat` versions
- **Hot Reload**: File watching and auto-rebuild
- **Comprehensive Testing**: Unit, integration, doc, examples
- **Code Quality**: Formatting, linting, validation
- **Release Management**: Multi-platform builds and packaging
- **Web Dashboard**: Development server with live monitoring
- **Version Control**: Automated tagging and changelog updates

## 🆘 Need Help?

```bash
# Tool-specific help
./dev-tools/setup-dev-env.sh --help
./dev-tools/dev-server.sh --help
./dev-tools/test-runner.sh --help

# Master launcher help
./dev-tools/dev.sh help

# Project status
./dev-tools/dev.sh status
```

## 🎨 Color Coding

- 🟢 **Green**: Success messages
- 🔴 **Red**: Error messages
- 🟡 **Yellow**: Warning messages
- 🔵 **Blue**: Information messages
- 🟣 **Purple**: Headers and banners
- 🟦 **Cyan**: Section headers

---

**Quick Start**: `./dev-tools/dev.sh dev` - Sets up everything and starts development server!
