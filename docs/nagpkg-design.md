# Nagari Package Manager (nagpkg) Design Document

## Overview

The Nagari Package Manager (`nagpkg`) is a comprehensive dependency management system for the Nagari programming language ecosystem. It provides package discovery, installation, version management, and distribution capabilities.

## Architecture

### Core Components

1. **Registry Client** - Interfaces with package registries
2. **Dependency Resolver** - Resolves version conflicts and dependencies
3. **Package Cache** - Local storage for downloaded packages
4. **Lock File Manager** - Manages exact dependency versions
5. **Build Integration** - Integrates with the Nagari compiler
6. **Publishing Tools** - Package creation and publishing utilities

### Package Structure

```
my-package/
├── nagari.json          # Package manifest
├── nag.lock            # Lockfile (auto-generated)
├── src/                # Source code
│   ├── main.nag
│   └── lib/
├── tests/              # Test files
├── docs/               # Documentation
├── examples/           # Example code
├── .nagignore          # Ignore patterns
└── README.md           # Package documentation
```

### Package Manifest (nagari.json)

```json
{
  "name": "my-awesome-package",
  "version": "1.2.0",
  "description": "An awesome Nagari package",
  "author": "Developer Name <dev@example.com>",
  "license": "MIT",
  "repository": "https://github.com/user/my-awesome-package",
  "homepage": "https://my-awesome-package.dev",
  "keywords": ["utility", "helper", "awesome"],

  "main": "src/main.nag",
  "exports": {
    ".": "./src/main.nag",
    "./utils": "./src/utils.nag",
    "./types": "./src/types.nag"
  },

  "dependencies": {
    "http-client": "^2.1.0",
    "json-parser": "~1.5.2",
    "math-utils": ">=1.0.0 <2.0.0"
  },

  "devDependencies": {
    "test-framework": "^3.0.0",
    "mock-server": "^1.2.0"
  },

  "peerDependencies": {
    "nagari-runtime": ">=0.3.0"
  },

  "optionalDependencies": {
    "performance-monitor": "^1.0.0"
  },

  "scripts": {
    "build": "nag build src/",
    "test": "nag test tests/",
    "lint": "nag lint src/",
    "format": "nag format src/",
    "dev": "nag run src/main.nag --watch",
    "prepublish": "nag test && nag build"
  },

  "engines": {
    "nagari": ">=0.3.0",
    "node": ">=16.0.0"
  },

  "os": ["linux", "darwin", "win32"],
  "cpu": ["x64", "arm64"],

  "files": [
    "src/",
    "dist/",
    "README.md",
    "LICENSE"
  ],

  "nagari": {
    "target": "js",
    "moduleFormat": "esm",
    "sourceDir": "src",
    "outputDir": "dist",
    "declarations": true,
    "sourcemap": true
  },

  "funding": {
    "type": "github",
    "url": "https://github.com/sponsors/username"
  }
}
```

### Lock File Format (nag.lock)

```json
{
  "lockfileVersion": "1.0.0",
  "nagariVersion": "0.3.0",
  "generated": "2025-06-16T10:30:00.000Z",

  "dependencies": {
    "http-client": {
      "version": "2.1.3",
      "resolved": "https://registry.nagari-lang.org/http-client/-/http-client-2.1.3.tgz",
      "integrity": "sha256-abc123...",
      "dependencies": {
        "url-parser": "^1.0.0"
      }
    },
    "json-parser": {
      "version": "1.5.7",
      "resolved": "https://registry.nagari-lang.org/json-parser/-/json-parser-1.5.7.tgz",
      "integrity": "sha256-def456...",
      "dependencies": {}
    },
    "url-parser": {
      "version": "1.0.2",
      "resolved": "https://registry.nagari-lang.org/url-parser/-/url-parser-1.0.2.tgz",
      "integrity": "sha256-ghi789..."
    }
  },

  "devDependencies": {
    "test-framework": {
      "version": "3.0.1",
      "resolved": "https://registry.nagari-lang.org/test-framework/-/test-framework-3.0.1.tgz",
      "integrity": "sha256-jkl012..."
    }
  }
}
```

## CLI Commands

### Installation Commands

```bash
# Initialize new package
nagpkg init
nagpkg init --name my-package --template library

# Install dependencies
nagpkg install                    # Install all dependencies
nagpkg install package-name      # Install specific package
nagpkg install package@version   # Install specific version
nagpkg install --dev package     # Install as dev dependency
nagpkg install --global package  # Install globally
nagpkg install --exact package   # Exact version match

# Add/remove dependencies
nagpkg add express@4.18.0
nagpkg add --dev typescript
nagpkg remove package-name
nagpkg remove --dev package-name

# Update dependencies
nagpkg update                     # Update all packages
nagpkg update package-name        # Update specific package
nagpkg outdated                   # Show outdated packages
```

### Information Commands

```bash
# List packages
nagpkg list                       # List installed packages
nagpkg list --tree               # Show dependency tree
nagpkg list --global             # List global packages

# Package information
nagpkg info package-name         # Show package information
nagpkg search query              # Search packages
nagpkg view package-name         # View package details

# Audit and security
nagpkg audit                     # Check for vulnerabilities
nagpkg audit fix                 # Fix vulnerabilities
```

### Publishing Commands

```bash
# Prepare package
nagpkg pack                      # Create package archive
nagpkg pack --output my-pkg.tgz

# Publishing
nagpkg login                     # Login to registry
nagpkg publish                   # Publish package
nagpkg publish --dry-run         # Test publish
nagpkg publish --tag beta        # Publish with tag

# Registry management
nagpkg whoami                    # Show current user
nagpkg logout                    # Logout from registry
```

### Workspace Commands

```bash
# Workspace management (monorepo support)
nagpkg workspace list            # List workspace packages
nagpkg workspace run build       # Run script in all workspaces
nagpkg workspace version patch   # Bump version in all packages
```

## Registry Architecture

### Package Registry API

```
GET    /api/packages                    # Search packages
GET    /api/packages/{name}             # Package metadata
GET    /api/packages/{name}/{version}   # Specific version info
GET    /api/packages/{name}/versions    # All versions
POST   /api/packages                    # Publish package
DELETE /api/packages/{name}/{version}   # Unpublish version

GET    /api/users/{username}            # User profile
POST   /api/users/login                 # Authentication
POST   /api/users/logout                # Logout

GET    /api/stats                       # Registry statistics
GET    /api/health                      # Health check
```

### Package Storage

```
registry.nagari-lang.org/
├── packages/
│   ├── {package-name}/
│   │   ├── {version}/
│   │   │   ├── package.tgz
│   │   │   ├── metadata.json
│   │   │   └── checksum.txt
│   │   └── index.json
│   └── index/
│       ├── by-name.json
│       ├── by-tag.json
│       └── search-index.json
├── users/
└── stats/
```

## Dependency Resolution Algorithm

### Version Resolution Strategy

1. **Collect Dependencies** - Gather all direct and transitive dependencies
2. **Build Dependency Graph** - Create directed acyclic graph
3. **Resolve Conflicts** - Use semantic versioning rules
4. **Select Versions** - Choose optimal versions
5. **Validate Constraints** - Ensure all constraints are satisfied
6. **Generate Lock File** - Create deterministic installation record

### Conflict Resolution Rules

1. **Exact Match** - `package@1.2.3` gets exactly 1.2.3
2. **Caret Range** - `^1.2.3` allows >=1.2.3 <2.0.0
3. **Tilde Range** - `~1.2.3` allows >=1.2.3 <1.3.0
4. **Range** - `>=1.0.0 <2.0.0` explicit range
5. **Latest** - `latest` gets newest stable version

### Peer Dependencies

```json
{
  "peerDependencies": {
    "react": ">=16.8.0",
    "nagari-runtime": "^0.3.0"
  },
  "peerDependenciesMeta": {
    "react": {
      "optional": true
    }
  }
}
```

## Security Features

### Package Integrity

- **Checksums** - SHA-256 verification for all packages
- **Signatures** - Cryptographic signing of packages
- **Audit Trail** - Complete version history tracking

### Vulnerability Management

- **Security Database** - Known vulnerability tracking
- **Automated Scanning** - Dependency vulnerability detection
- **Update Notifications** - Security update alerts

### Registry Security

- **Authentication** - Multi-factor authentication support
- **Authorization** - Role-based access control
- **Rate Limiting** - API abuse prevention
- **Audit Logging** - Complete operation logging

## Integration with Build System

### Compiler Integration

```rust
// Package resolution during compilation
impl Compiler {
    fn resolve_imports(&self, imports: &[Import]) -> Result<Vec<ResolvedImport>> {
        let resolver = PackageResolver::new(&self.config.package_config);
        resolver.resolve_imports(imports)
    }
}
```

### Build Configuration

```toml
# nagari.toml
[package]
name = "my-project"
version = "0.1.0"

[dependencies]
http-client = "^2.1.0"
json-parser = "~1.5.2"

[build]
target = "js"
output_dir = "dist"
include_dependencies = true
tree_shake = true
```

## Workspace Support (Monorepo)

### Workspace Configuration

```json
{
  "name": "my-monorepo",
  "private": true,
  "workspaces": [
    "packages/*",
    "apps/*",
    "tools/build-utils"
  ],
  "scripts": {
    "build:all": "nagpkg workspace run build",
    "test:all": "nagpkg workspace run test",
    "lint:all": "nagpkg workspace run lint"
  }
}
```

### Cross-Package Dependencies

```json
{
  "name": "@myorg/frontend",
  "dependencies": {
    "@myorg/shared-utils": "workspace:*",
    "@myorg/api-client": "workspace:^1.0.0"
  }
}
```

## Performance Optimizations

### Caching Strategy

- **Global Cache** - Shared package cache across projects
- **Content Addressing** - Deduplicate identical packages
- **Incremental Updates** - Only download changed parts
- **Parallel Downloads** - Concurrent package fetching

### Network Optimizations

- **CDN Distribution** - Global content delivery
- **Compression** - Gzip/Brotli compression
- **HTTP/2** - Multiplexed connections
- **Offline Mode** - Work without network connectivity

## Future Enhancements

### Planned Features

1. **Plugin System** - Extensible package manager
2. **Private Registries** - Enterprise package hosting
3. **Mirror Support** - Regional registry mirrors
4. **Dependency Analytics** - Usage and performance metrics
5. **Auto-Updates** - Automated dependency updates
6. **Integration APIs** - Third-party tool integration

### Roadmap

- **Phase 1** - Basic package management (Q3 2025)
- **Phase 2** - Registry and publishing (Q4 2025)
- **Phase 3** - Workspace support (Q1 2026)
- **Phase 4** - Advanced features (Q2 2026)

This design provides a comprehensive foundation for the Nagari package management ecosystem, ensuring scalability, security, and developer productivity.
