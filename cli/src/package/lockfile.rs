#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::fs;
use anyhow::Result;
use semver::Version;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockFile {
    pub lockfile_version: String,
    pub name: String,
    pub version: String,
    pub requires: bool,
    pub packages: HashMap<String, LockedDependency>,
    pub dependencies: HashMap<String, DependencyReference>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedDependency {
    pub version: String,
    pub resolved: String,
    pub integrity: String,
    pub dev: Option<bool>,
    pub optional: Option<bool>,
    pub peer: Option<bool>,
    pub requires: Option<HashMap<String, String>>,
    pub dependencies: Option<HashMap<String, DependencyReference>>,
    pub engines: Option<HashMap<String, String>>,
    pub os: Option<Vec<String>>,
    pub cpu: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyReference {
    pub version: String,
    pub requires: Option<HashMap<String, String>>,
}

impl LockFile {
    pub fn new(name: String, version: String) -> Self {
        Self {
            lockfile_version: "3".to_string(),
            name,
            version,
            requires: true,
            packages: HashMap::new(),
            dependencies: HashMap::new(),
        }
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let lockfile: LockFile = serde_json::from_str(&content)?;
        Ok(lockfile)
    }

    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn get_package(&self, name: &str) -> Option<&LockedDependency> {
        // Try direct lookup first
        if let Some(package) = self.packages.get(name) {
            return Some(package);
        }

        // Try with node_modules prefix for nested dependencies
        let node_modules_key = format!("node_modules/{}", name);
        self.packages.get(&node_modules_key)
    }

    pub fn add_package(&mut self, name: String, package: LockedDependency) {
        self.packages.insert(name.clone(), package.clone());

        // Also add to top-level dependencies if it's a direct dependency
        if !package.dev.unwrap_or(false) && !package.optional.unwrap_or(false) {
            self.dependencies.insert(name, DependencyReference {
                version: package.version.clone(),
                requires: package.requires.clone(),
            });
        }
    }

    pub fn remove_package(&mut self, name: &str) -> bool {
        let removed_from_packages = self.packages.remove(name).is_some();
        let removed_from_deps = self.dependencies.remove(name).is_some();

        // Also remove any nested packages under this name
        let prefix = format!("{}/node_modules/", name);
        let keys_to_remove: Vec<_> = self.packages
            .keys()
            .filter(|k| k.starts_with(&prefix))
            .cloned()
            .collect();

        for key in keys_to_remove {
            self.packages.remove(&key);
        }

        removed_from_packages || removed_from_deps
    }

    pub fn update_package(&mut self, name: &str, package: LockedDependency) -> bool {
        if self.packages.contains_key(name) {
            self.packages.insert(name.to_string(), package.clone());

            // Update top-level dependency reference if it exists
            if let Some(dep_ref) = self.dependencies.get_mut(name) {
                dep_ref.version = package.version.clone();
                dep_ref.requires = package.requires.clone();
            }

            true
        } else {
            false
        }
    }

    pub fn get_all_package_names(&self) -> Vec<String> {
        self.packages.keys().cloned().collect()
    }

    pub fn get_direct_dependencies(&self) -> &HashMap<String, DependencyReference> {
        &self.dependencies
    }

    pub fn get_dev_dependencies(&self) -> HashMap<String, &LockedDependency> {
        self.packages
            .iter()
            .filter(|(_, package)| package.dev.unwrap_or(false))
            .map(|(name, package)| (name.clone(), package))
            .collect()
    }

    pub fn get_optional_dependencies(&self) -> HashMap<String, &LockedDependency> {
        self.packages
            .iter()
            .filter(|(_, package)| package.optional.unwrap_or(false))
            .map(|(name, package)| (name.clone(), package))
            .collect()
    }

    pub fn get_peer_dependencies(&self) -> HashMap<String, &LockedDependency> {
        self.packages
            .iter()
            .filter(|(_, package)| package.peer.unwrap_or(false))
            .map(|(name, package)| (name.clone(), package))
            .collect()
    }

    pub fn validate_integrity(&self) -> Result<Vec<String>> {
        let mut errors = Vec::new();        // Check that all required dependencies exist
        for name in self.dependencies.keys() {
            if !self.packages.contains_key(name) {
                errors.push(format!("Missing package: {}", name));
            }
        }

        // Check that all package requirements are satisfied
        for (name, package) in &self.packages {
            if let Some(ref requires) = package.requires {
                for (req_name, req_version) in requires {
                    match self.get_package(req_name) {
                        Some(req_package) => {
                            if !version_satisfies(&req_package.version, req_version) {
                                errors.push(format!(
                                    "Package {} requires {} {} but found {}",
                                    name, req_name, req_version, req_package.version
                                ));
                            }
                        }
                        None => {
                            errors.push(format!(
                                "Package {} requires {} {} but it's not installed",
                                name, req_name, req_version
                            ));
                        }
                    }
                }
            }
        }

        Ok(errors)
    }

    pub fn compute_stats(&self) -> LockFileStats {
        let total_packages = self.packages.len();
        let direct_deps = self.dependencies.len();
        let dev_deps = self.get_dev_dependencies().len();
        let optional_deps = self.get_optional_dependencies().len();
        let peer_deps = self.get_peer_dependencies().len();

        let total_size = self.packages
            .values()
            .filter_map(|p| p.dependencies.as_ref())
            .map(|deps| deps.len())
            .sum::<usize>();

        LockFileStats {
            total_packages,
            direct_dependencies: direct_deps,
            dev_dependencies: dev_deps,
            optional_dependencies: optional_deps,
            peer_dependencies: peer_deps,
            transitive_dependencies: total_packages - direct_deps,
            total_dependency_graph_size: total_size,
        }
    }

    pub fn find_duplicate_packages(&self) -> HashMap<String, Vec<String>> {
        let mut package_versions: HashMap<String, Vec<String>> = HashMap::new();

        for (full_name, package) in &self.packages {
            let package_name = extract_package_name(full_name);
            package_versions
                .entry(package_name)
                .or_default()
                .push(package.version.clone());
        }

        package_versions
            .into_iter()
            .filter(|(_, versions)| {
                let mut unique_versions: Vec<_> = versions.iter().collect();
                unique_versions.sort();
                unique_versions.dedup();
                unique_versions.len() > 1
            })
            .map(|(name, versions)| {
                let mut unique_versions: Vec<_> = versions.into_iter().collect();
                unique_versions.sort();
                unique_versions.dedup();
                (name, unique_versions)
            })
            .collect()
    }

    pub fn prune_unused_packages(&mut self, keep_dev: bool, keep_optional: bool) -> Vec<String> {
        let mut removed = Vec::new();
        let mut packages_to_remove = Vec::new();

        for (name, package) in &self.packages {
            let should_remove = (!keep_dev && package.dev.unwrap_or(false))
                || (!keep_optional && package.optional.unwrap_or(false));

            if should_remove {
                packages_to_remove.push(name.clone());
            }
        }

        for name in packages_to_remove {
            if self.remove_package(&name) {
                removed.push(name);
            }
        }

        removed
    }
}

#[derive(Debug, Clone)]
pub struct LockFileStats {
    pub total_packages: usize,
    pub direct_dependencies: usize,
    pub dev_dependencies: usize,
    pub optional_dependencies: usize,
    pub peer_dependencies: usize,
    pub transitive_dependencies: usize,
    pub total_dependency_graph_size: usize,
}

impl LockedDependency {
    pub fn new(version: String, resolved: String, integrity: String) -> Self {
        Self {
            version,
            resolved,
            integrity,
            dev: None,
            optional: None,
            peer: None,
            requires: None,
            dependencies: None,
            engines: None,
            os: None,
            cpu: None,
        }
    }

    pub fn with_dev(mut self, dev: bool) -> Self {
        self.dev = Some(dev);
        self
    }

    pub fn with_optional(mut self, optional: bool) -> Self {
        self.optional = Some(optional);
        self
    }

    pub fn with_peer(mut self, peer: bool) -> Self {
        self.peer = Some(peer);
        self
    }

    pub fn with_requires(mut self, requires: HashMap<String, String>) -> Self {
        self.requires = Some(requires);
        self
    }

    pub fn with_dependencies(mut self, dependencies: HashMap<String, DependencyReference>) -> Self {
        self.dependencies = Some(dependencies);
        self
    }

    pub fn is_dev(&self) -> bool {
        self.dev.unwrap_or(false)
    }

    pub fn is_optional(&self) -> bool {
        self.optional.unwrap_or(false)
    }

    pub fn is_peer(&self) -> bool {
        self.peer.unwrap_or(false)
    }
}

// Helper functions

fn version_satisfies(installed_version: &str, required_version: &str) -> bool {
    // Simple version satisfaction check
    // In a real implementation, this would use semver matching
    if required_version.starts_with('^') || required_version.starts_with('~') {
        // Handle semver ranges
        match (Version::parse(installed_version), Version::parse(&required_version[1..])) {
            (Ok(installed), Ok(required)) => {
                if required_version.starts_with('^') {
                    installed.major == required.major && installed >= required
                } else {
                    installed.major == required.major
                        && installed.minor == required.minor
                        && installed >= required
                }
            }
            _ => false,
        }
    } else {
        // Exact version match
        installed_version == required_version
    }
}

fn extract_package_name(full_name: &str) -> String {
    // Extract package name from full path (e.g., "node_modules/@scope/package" -> "@scope/package")
    if let Some(stripped) = full_name.strip_prefix("node_modules/") {
        stripped.to_string()
    } else {
        full_name.to_string()
    }
}

impl std::fmt::Display for LockFileStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Lock File Statistics:")?;
        writeln!(f, "  Total packages: {}", self.total_packages)?;
        writeln!(f, "  Direct dependencies: {}", self.direct_dependencies)?;
        writeln!(f, "  Dev dependencies: {}", self.dev_dependencies)?;
        writeln!(f, "  Optional dependencies: {}", self.optional_dependencies)?;
        writeln!(f, "  Peer dependencies: {}", self.peer_dependencies)?;
        writeln!(f, "  Transitive dependencies: {}", self.transitive_dependencies)?;
        writeln!(f, "  Total dependency graph size: {}", self.total_dependency_graph_size)
    }
}
