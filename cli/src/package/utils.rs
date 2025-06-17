use anyhow::Result;
use super::{
    PackageManifest, NagariConfig, DependencySpec,
    RegistryClient, PackageInfo, VersionInfo,
    DependencyResolver, ResolutionResult,
    PackageCache, LockFile, LockedDependency
};
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
        let cache = PackageCache::new(config.cache_dir.clone().unwrap_or_else(|| PathBuf::from(".nagari_cache")))?;
        let resolver = DependencyResolver::new(cache.clone());
        let registry = RegistryClient::new(&config.registry_url.unwrap_or_else(|| "https://registry.nagari.dev".to_string()))?;
        
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
            if let DependencySpec::Version { version, .. } = spec {
                if version.is_empty() {
                    warnings.push(format!("Empty version for dependency: {}", name));
                }
            }
        }
        
        Ok(warnings)
    }
    
    pub async fn get_package_info(&self, name: &str) -> Result<PackageInfo> {
        self.registry.get_package_info(name).await
    }
    
    pub async fn get_version_info(&self, name: &str, version: &str) -> Result<VersionInfo> {
        self.registry.get_version_info(name, version).await
    }
    
    pub async fn resolve_dependencies(&self, manifest: &PackageManifest, lockfile: Option<&LockFile>) -> Result<ResolutionResult> {
        self.resolver.resolve_dependencies(manifest, lockfile).await
    }
    
    pub fn create_dependency_spec(version: &str) -> DependencySpec {
        DependencySpec::Version {
            version: version.to_string(),
            registry: None,
            features: Vec::new(),
            optional: false,
        }
    }
    
    pub fn create_locked_dependency(name: &str, version: &str, source: &str) -> LockedDependency {
        LockedDependency {
            name: name.to_string(),
            version: version.to_string(),
            source: source.to_string(),
            checksum: None,
            requires: std::collections::HashMap::new(),
            dependencies: std::collections::HashMap::new(),
        }
    }
}
