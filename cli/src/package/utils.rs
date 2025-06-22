#![allow(dead_code)]

use super::{
    cache::PackageCache,
    lockfile::LockedDependency,
    manifest::{DependencySpec, NagariConfig, PackageManifest},
    registry::{PackageInfo, RegistryClient, VersionInfo},
    resolver::{DependencyResolver, ResolutionContext, ResolutionResult},
};
use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;

/// Package utilities that use all the imported types
pub struct PackageUtils {
    config: NagariConfig,
    cache: PackageCache,
    resolver: DependencyResolver,
    registry: RegistryClient,
}

impl PackageUtils {
    pub fn new(config: NagariConfig) -> Result<Self> {
        let cache_dir = PathBuf::from(".nagari_cache");
        let cache = PackageCache::new(cache_dir)?;
        let registry_url = "https://registry.nagari.dev";
        let registry = RegistryClient::new(registry_url)?;
        let resolver = DependencyResolver::new(registry.clone());

        Ok(Self {
            config,
            cache,
            resolver,
            registry,
        })
    }
    pub async fn validate_manifest(&self, manifest: &PackageManifest) -> Result<Vec<String>> {
        let mut warnings = Vec::new();

        // Check dependencies
        for (name, spec) in &manifest.dependencies {
            match spec {
                DependencySpec::Version(version) => {
                    if version.is_empty() {
                        warnings.push(format!("Empty version for dependency: {}", name));
                    }
                }
                DependencySpec::Detailed { version, .. } => {
                    if let Some(ver) = version {
                        if ver.is_empty() {
                            warnings.push(format!("Empty version for dependency: {}", name));
                        }
                    }
                }
            }
        }

        Ok(warnings)
    }

    pub async fn get_package_info(&self, name: &str) -> Result<Option<PackageInfo>> {
        self.registry.get_package_info(name).await
    }

    pub async fn get_version_info(&self, name: &str, version: &str) -> Result<Option<VersionInfo>> {
        self.registry.get_version_info(name, version).await
    }
    pub async fn resolve_dependencies(
        &mut self,
        manifest: &PackageManifest,
        context: &ResolutionContext,
    ) -> Result<ResolutionResult> {
        self.resolver.resolve_dependencies(manifest, context).await
    }
    pub fn create_dependency_spec(version: &str) -> DependencySpec {
        DependencySpec::Version(version.to_string())
    }

    pub fn create_locked_dependency(
        _name: &str,
        version: &str,
        resolved: &str,
    ) -> LockedDependency {
        LockedDependency {
            version: version.to_string(),
            resolved: resolved.to_string(),
            integrity: "".to_string(), // This would be calculated from actual file
            dev: None,
            optional: None,
            peer: None,
            requires: Some(HashMap::new()),
            dependencies: Some(HashMap::new()),
            engines: None,
            os: None,
            cpu: None,
        }
    }
}
