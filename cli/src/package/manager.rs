use crate::config::NagConfig;
use crate::package::{
    manifest::{PackageManifest, DependencySpec},
    registry::RegistryClient,
    resolver::{DependencyResolver, ResolutionContext, UpdateStrategy},
    cache::PackageCache,
    lockfile::LockFile,
};
use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;

pub struct PackageManager {
    config: NagConfig,
    registry: RegistryClient,
    resolver: DependencyResolver,
    cache: PackageCache,
}

impl PackageManager {
    pub fn new(config: NagConfig) -> Result<Self> {
        let registry_url = config.package.registry_url.as_deref()
            .unwrap_or("https://registry.nagari.dev");

        let registry = RegistryClient::new(registry_url)?;
        let resolver = DependencyResolver::new(registry.clone());

        let cache_dir = config.package.cache_dir.clone()
            .unwrap_or_else(|| dirs::cache_dir()
                .unwrap_or_else(|| PathBuf::from(".nagari-cache"))
                .join("nagari"));

        let cache = PackageCache::new(cache_dir)?;

        Ok(Self {
            config,
            registry,
            resolver,
            cache,
        })
    }

    pub async fn init_package(&self, name: Option<String>, yes: bool) -> Result<()> {
        let package_file = PathBuf::from("nagari.json");

        if package_file.exists() && !yes {
            println!("Package file already exists. Overwrite? (y/N)");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if !input.trim().to_lowercase().starts_with('y') {
                println!("Aborted.");
                return Ok(());
            }
        }

        let package_name = name.unwrap_or_else(|| {
            std::env::current_dir()
                .ok()
                .and_then(|path| path.file_name())
                .and_then(|name| name.to_str())
                .unwrap_or("my-nagari-package")
                .to_string()
        });

        let manifest = PackageManifest::new(package_name, "1.0.0".to_string());
        manifest.to_file(&package_file)?;

        // Create basic project structure
        fs::create_dir_all("src")?;
        if !PathBuf::from("src/main.nag").exists() {
            fs::write("src/main.nag", "# Main module\nprint(\"Hello, Nagari!\")\n")?;
        }

        fs::create_dir_all("tests")?;
        fs::create_dir_all("docs")?;

        if !PathBuf::from("README.md").exists() {
            fs::write("README.md", &format!("# {}\n\nA Nagari package.\n", manifest.name))?;
        }

        if !PathBuf::from(".nagignore").exists() {
            fs::write(".nagignore", "node_modules/\ndist/\n*.log\n.env\n")?;
        }

        println!("Initialized package: {}", manifest.name);
        Ok(())
    }

    pub async fn install(&mut self, packages: Vec<String>, save_dev: bool) -> Result<()> {
        let manifest_path = PathBuf::from("nagari.json");
        let mut manifest = if manifest_path.exists() {
            PackageManifest::from_file(&manifest_path)?
        } else {
            return Err(anyhow::anyhow!("No nagari.json found. Run 'nag package init' first."));
        };

        // Add packages to manifest
        for package_spec in packages {
            let (name, version) = self.parse_package_spec(&package_spec)?;
            let dep_spec = DependencySpec::version(&version);

            if save_dev {
                manifest.add_dev_dependency(name, dep_spec);
            } else {
                manifest.add_dependency(name, dep_spec);
            }
        }

        // Resolve all dependencies
        let context = if save_dev {
            ResolutionContext::development()
        } else {
            ResolutionContext::production()
        };

        let resolution = self.resolver.resolve_dependencies(&manifest, &context).await?;

        // Display resolution results
        if !resolution.conflicts.is_empty() {
            println!("⚠️  Dependency conflicts detected:");
            for conflict in &resolution.conflicts {
                println!("  - {}: {}", conflict.package, conflict.conflicting_versions.len());
            }
        }

        if !resolution.warnings.is_empty() {
            println!("⚠️  Warnings:");
            for warning in &resolution.warnings {
                println!("  - {}", warning.message);
            }
        }

        // Install packages
        for (name, resolved_dep) in &resolution.resolved {
            println!("📦 Installing {}@{}", name, resolved_dep.version);

            // Download and cache package
            let package_data = self.registry.download_package(name, &resolved_dep.version.to_string()).await?;
            let metadata = serde_json::json!({
                "name": name,
                "version": resolved_dep.version.to_string(),
                "resolved": resolved_dep.resolved_url,
                "integrity": resolved_dep.integrity
            });

            self.cache.cache_package(name, &resolved_dep.version.to_string(), &package_data, metadata).await?;
        }

        // Update manifest
        manifest.to_file(&manifest_path)?;

        // Create/update lock file
        let lockfile_path = PathBuf::from("nag.lock");
        let mut lockfile = if lockfile_path.exists() {
            LockFile::from_file(&lockfile_path)?
        } else {
            LockFile::new(manifest.name.clone(), manifest.version.clone())
        };

        for (name, resolved_dep) in &resolution.resolved {
            use crate::package::lockfile::LockedDependency;

            let locked_dep = LockedDependency::new(
                resolved_dep.version.to_string(),
                resolved_dep.resolved_url.clone(),
                resolved_dep.integrity.clone(),
            )
            .with_dev(resolved_dep.dev)
            .with_optional(resolved_dep.optional)
            .with_peer(resolved_dep.peer);

            lockfile.add_package(name.clone(), locked_dep);
        }

        lockfile.to_file(&lockfile_path)?;

        println!("✅ Installation completed!");
        Ok(())
    }

    pub async fn uninstall(&mut self, packages: Vec<String>) -> Result<()> {
        let manifest_path = PathBuf::from("nagari.json");
        let mut manifest = PackageManifest::from_file(&manifest_path)?;

        for package_name in &packages {
            if manifest.remove_dependency(package_name) {
                println!("📦 Removed {}", package_name);
            } else {
                println!("⚠️  Package {} not found in dependencies", package_name);
            }
        }

        manifest.to_file(&manifest_path)?;

        // Update lock file
        let lockfile_path = PathBuf::from("nag.lock");
        if lockfile_path.exists() {
            let mut lockfile = LockFile::from_file(&lockfile_path)?;

            for package_name in &packages {
                lockfile.remove_package(package_name);
            }

            lockfile.to_file(&lockfile_path)?;
        }

        println!("✅ Uninstall completed!");
        Ok(())
    }

    pub async fn update(&mut self, packages: Option<Vec<String>>) -> Result<()> {
        let manifest_path = PathBuf::from("nagari.json");
        let manifest = PackageManifest::from_file(&manifest_path)?;

        let context = ResolutionContext::development()
            .with_update_strategy(UpdateStrategy::Minor);

        let resolution = self.resolver.resolve_dependencies(&manifest, &context).await?;

        // Show update information
        println!("📦 Checking for updates...");

        let lockfile_path = PathBuf::from("nag.lock");
        let old_lockfile = if lockfile_path.exists() {
            Some(LockFile::from_file(&lockfile_path)?)
        } else {
            None
        };

        if let Some(ref old_lock) = old_lockfile {
            for (name, resolved_dep) in &resolution.resolved {
                if let Some(old_dep) = old_lock.get_package(name) {
                    if old_dep.version != resolved_dep.version.to_string() {
                        println!("⬆️  {}@{} → {}", name, old_dep.version, resolved_dep.version);
                    }
                }
            }
        }

        // Install updated packages (same logic as install)
        self.install_resolved_dependencies(&resolution).await?;

        println!("✅ Update completed!");
        Ok(())
    }

    pub async fn list(&self) -> Result<()> {
        let manifest_path = PathBuf::from("nagari.json");
        let manifest = PackageManifest::from_file(&manifest_path)?;

        if !manifest.dependencies.is_empty() {
            println!("📦 Dependencies:");
            for (name, spec) in &manifest.dependencies {
                if let Some(version) = spec.get_version() {
                    println!("  {} {}", name, version);
                } else {
                    println!("  {} {:?}", name, spec);
                }
            }
            println!();
        }

        if !manifest.dev_dependencies.is_empty() {
            println!("🔧 Dev Dependencies:");
            for (name, spec) in &manifest.dev_dependencies {
                if let Some(version) = spec.get_version() {
                    println!("  {} {}", name, version);
                } else {
                    println!("  {} {:?}", name, spec);
                }
            }
            println!();
        }

        Ok(())
    }

    pub async fn search(&self, query: String) -> Result<()> {
        println!("🔍 Searching for '{}'...", query);

        let results = self.registry.search_packages(&query, Some(20)).await?;

        if results.objects.is_empty() {
            println!("No packages found.");
            return Ok(());
        }

        for result in results.objects {
            let package = result.package;
            println!("📦 {}", package.name);
            if let Some(ref description) = package.description {
                println!("   {}", description);
            }
            println!("   Latest: {} | Score: {:.2}", package.version, result.searchScore);
            if !package.keywords.is_empty() {
                println!("   Keywords: {}", package.keywords.join(", "));
            }
            println!();
        }

        Ok(())
    }

    pub async fn info(&self, package_name: String) -> Result<()> {
        println!("📦 Package information for '{}'", package_name);

        let package_info = self.registry.get_package_info(&package_name).await?
            .ok_or_else(|| anyhow::anyhow!("Package '{}' not found", package_name))?;

        println!("Name: {}", package_info.name);
        if let Some(ref description) = package_info.description {
            println!("Description: {}", description);
        }

        if let Some(ref author) = package_info.author {
            println!("Author: {}", author.name);
        }

        if let Some(ref license) = package_info.license {
            println!("License: {}", license);
        }

        if let Some(ref homepage) = package_info.homepage {
            println!("Homepage: {}", homepage);
        }

        if !package_info.keywords.is_empty() {
            println!("Keywords: {}", package_info.keywords.join(", "));
        }

        // Show latest versions
        let mut versions: Vec<_> = package_info.versions.keys().collect();
        versions.sort();
        versions.reverse();

        println!("Versions: {}", versions.iter().take(10).map(|s| s.as_str()).collect::<Vec<_>>().join(", "));
        if versions.len() > 10 {
            println!("  ... and {} more", versions.len() - 10);
        }

        Ok(())
    }

    pub async fn cache_info(&self) -> Result<()> {
        let stats = self.cache.get_cache_stats();
        println!("{}", stats);
        Ok(())
    }

    pub async fn cache_clean(&mut self) -> Result<()> {
        println!("🧹 Cleaning package cache...");
        self.cache.clear_cache()?;
        println!("✅ Cache cleaned!");
        Ok(())
    }

    fn parse_package_spec(&self, spec: &str) -> Result<(String, String)> {
        if let Some(at_pos) = spec.rfind('@') {
            let name = spec[..at_pos].to_string();
            let version = spec[at_pos + 1..].to_string();
            Ok((name, version))
        } else {
            // Default to latest version
            Ok((spec.to_string(), "latest".to_string()))
        }
    }

    async fn install_resolved_dependencies(
        &mut self,
        resolution: &crate::package::resolver::ResolutionResult,
    ) -> Result<()> {
        // This would contain the actual installation logic
        // For now, just cache the packages
        for (name, resolved_dep) in &resolution.resolved {
            let package_data = self.registry.download_package(name, &resolved_dep.version.to_string()).await?;
            let metadata = serde_json::json!({
                "name": name,
                "version": resolved_dep.version.to_string(),
                "resolved": resolved_dep.resolved_url,
                "integrity": resolved_dep.integrity
            });

            self.cache.cache_package(name, &resolved_dep.version.to_string(), &package_data, metadata).await?;
        }

        Ok(())
    }
}
