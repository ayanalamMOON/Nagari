#![allow(dead_code)]

use anyhow::Result;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;
use std::path::Path;
use std::pin::Pin;
use std::process::Command;

use crate::package::manifest::{DependencySpec, PackageManifest};
use crate::package::registry::{RegistryClient, VersionInfo};
use tempfile::TempDir;

#[derive(Debug, Clone)]
pub struct DependencyResolver {
    registry: RegistryClient,
    cache: ResolverCache,
}

#[derive(Debug, Clone)]
pub struct ResolverCache {
    package_info: HashMap<String, CachedPackageInfo>,
    resolutions: HashMap<String, ResolutionResult>,
}

#[derive(Debug, Clone)]
pub struct CachedPackageInfo {
    versions: Vec<Version>,
    version_info: HashMap<Version, VersionInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionResult {
    pub resolved: HashMap<String, ResolvedDependency>,
    pub conflicts: Vec<DependencyConflict>,
    pub warnings: Vec<ResolutionWarning>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedDependency {
    pub name: String,
    pub version: Version,
    pub resolved_url: String,
    pub integrity: String,
    pub dependencies: HashMap<String, Version>,
    pub dev: bool,
    pub optional: bool,
    pub peer: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyConflict {
    pub package: String,
    pub conflicting_versions: Vec<ConflictingVersion>,
    pub resolution: ConflictResolution,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictingVersion {
    pub version: Version,
    pub required_by: Vec<String>,
    pub requirement: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictResolution {
    UseLatest(Version),
    UseExplicit(Version),
    Failed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionWarning {
    pub kind: WarningKind,
    pub message: String,
    pub package: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WarningKind {
    PeerDependencyMissing,
    PeerDependencyConflict,
    OptionalDependencyFailed,
    DeprecatedPackage,
    VulnerablePackage,
    LicenseConflict,
}

#[derive(Debug, Clone)]
pub struct ResolutionContext {
    pub include_dev: bool,
    pub include_optional: bool,
    pub include_peer: bool,
    pub prefer_latest: bool,
    pub allow_prereleases: bool,
    pub update_strategy: UpdateStrategy,
}

#[derive(Debug, Clone)]
pub enum UpdateStrategy {
    None,   // Use exact versions from lockfile
    Patch,  // Allow patch updates (~1.2.3)
    Minor,  // Allow minor updates (^1.2.3)
    Major,  // Allow major updates (>=1.2.3)
    Latest, // Use latest available
}

impl DependencyResolver {
    pub fn new(registry: RegistryClient) -> Self {
        Self {
            registry,
            cache: ResolverCache::new(),
        }
    }

    pub async fn resolve_dependencies(
        &mut self,
        manifest: &PackageManifest,
        context: &ResolutionContext,
    ) -> Result<ResolutionResult> {
        let mut resolution = ResolutionResult {
            resolved: HashMap::new(),
            conflicts: Vec::new(),
            warnings: Vec::new(),
        };

        // Collect all dependencies
        let mut all_deps = HashMap::new();

        // Add production dependencies
        for (name, spec) in &manifest.dependencies {
            all_deps.insert(name.clone(), (spec.clone(), false, false, false));
        }

        // Add dev dependencies if requested
        if context.include_dev {
            for (name, spec) in &manifest.dev_dependencies {
                all_deps.insert(name.clone(), (spec.clone(), true, false, false));
            }
        }

        // Add optional dependencies if requested
        if context.include_optional {
            for (name, spec) in &manifest.optional_dependencies {
                all_deps.insert(name.clone(), (spec.clone(), false, true, false));
            }
        }

        // Add peer dependencies if requested
        if context.include_peer {
            for (name, spec) in &manifest.peer_dependencies {
                all_deps.insert(name.clone(), (spec.clone(), false, false, true));
            }
        }

        // Resolve each dependency tree
        let mut resolution_graph = HashMap::new();

        for (name, (spec, is_dev, is_optional, is_peer)) in all_deps {
            match self
                .resolve_dependency_tree(&name, &spec, context, &mut resolution_graph)
                .await
            {
                Ok(resolved) => {
                    resolution.resolved.insert(
                        name.clone(),
                        ResolvedDependency {
                            name: name.clone(),
                            version: resolved.version.clone(),
                            resolved_url: resolved.resolved_url.clone(),
                            integrity: resolved.integrity.clone(),
                            dependencies: resolved.dependencies.clone(),
                            dev: is_dev,
                            optional: is_optional,
                            peer: is_peer,
                        },
                    );
                }
                Err(e) => {
                    if is_optional {
                        resolution.warnings.push(ResolutionWarning {
                            kind: WarningKind::OptionalDependencyFailed,
                            message: format!(
                                "Failed to resolve optional dependency {}: {}",
                                name, e
                            ),
                            package: Some(name),
                        });
                    } else {
                        return Err(e);
                    }
                }
            }
        }

        // Check for conflicts
        self.detect_conflicts(&mut resolution).await?;

        // Check for warnings
        self.detect_warnings(&mut resolution).await?;

        Ok(resolution)
    }

    fn resolve_dependency_tree_boxed<'a>(
        &'a mut self,
        name: &'a str,
        spec: &'a DependencySpec,
        context: &'a ResolutionContext,
        _resolution_graph: &'a mut HashMap<String, ResolvedDependency>,
    ) -> Pin<Box<dyn Future<Output = Result<ResolvedDependency>> + Send + 'a>> {
        Box::pin(async move {            // Handle local path dependencies
            if let DependencySpec::Detailed {
                path: Some(_path), ..
            } = spec
            {
                return self.resolve_git_dependency(name, "", None, None).await;
            }

            // Handle git dependencies
            if let DependencySpec::Detailed {
                git: Some(git_url),
                branch,
                tag,
                ..
            } = spec
            {
                return self
                    .resolve_git_dependency(name, git_url, branch.as_deref(), tag.as_deref())
                    .await;
            }

            // Handle registry dependencies
            let version_req = self.parse_version_requirement(spec)?;
            // Clone package_info to avoid holding a reference across await
            let package_info = self.get_package_info(name).await?.clone();

            let suitable_version =
                self.find_suitable_version(&package_info.versions, &version_req, context)?;
            let version_info = package_info
                .version_info
                .get(&suitable_version)
                .ok_or_else(|| {
                    anyhow::anyhow!("Version info not found for {} {}", name, suitable_version)
                })?;

            // Clone dependencies to avoid borrow checker issues
            let deps_to_resolve: Vec<_> = version_info
                .dependencies
                .iter()
                .map(|(name, version)| (name.clone(), version.clone()))
                .collect();

            // Recursively resolve dependencies
            let mut dependencies = HashMap::new();
            for (dep_name, dep_version_req) in deps_to_resolve {
                let dep_spec = DependencySpec::Version(dep_version_req);
                let resolved_dep = self
                    .resolve_dependency_tree_boxed(&dep_name, &dep_spec, context, _resolution_graph)
                    .await?;
                dependencies.insert(dep_name, resolved_dep.version);
            }

            Ok(ResolvedDependency {
                name: name.to_string(),
                version: suitable_version,
                resolved_url: version_info.dist.tarball.clone(),
                integrity: version_info.dist.integrity.clone().unwrap_or_default(),
                dependencies,
                dev: false,
                optional: false,
                peer: false,
            })
        })
    }

    async fn resolve_dependency_tree(
        &mut self,
        name: &str,
        spec: &DependencySpec,
        context: &ResolutionContext,
        resolution_graph: &mut HashMap<String, ResolvedDependency>,
    ) -> Result<ResolvedDependency> {
        self.resolve_dependency_tree_boxed(name, spec, context, resolution_graph)
            .await
    }

    async fn resolve_local_dependency(
        &self,
        name: &str,
        path: &Path,
    ) -> Result<ResolvedDependency> {
        let manifest_path = path.join("nagari.json");
        let manifest = PackageManifest::from_file(&manifest_path)?;

        let version = Version::parse(&manifest.version)?;

        Ok(ResolvedDependency {
            name: name.to_string(),
            version,
            resolved_url: format!("file:{}", path.display()),
            integrity: String::new(),
            dependencies: HashMap::new(),
            dev: false,
            optional: false,
            peer: false,
        })
    }

    async fn resolve_git_dependency(
        &self,
        name: &str,
        git_url: &str,
        branch: Option<&str>,
        tag: Option<&str>,
    ) -> Result<ResolvedDependency> {
        // Implement git dependency resolution
        // 1. Clone or fetch the git repository to a temp/cache directory.
        // 2. Checkout the specified branch or tag if provided.
        // 3. Read the nagari.json manifest from the repo.
        // 4. Parse the version and dependencies.
        // 5. Return a ResolvedDependency.

        // Create a temporary directory for the git clone
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path();

        // Prepare git clone command
        let mut clone_args = vec![git_url, repo_path.to_str().unwrap()];
        if let Some(branch) = branch {
            clone_args.insert(0, "--branch");
            clone_args.insert(1, branch);
        }
        if let Some(tag) = tag {
            clone_args.insert(0, "--branch");
            clone_args.insert(1, tag);
        }

        // Clone the repository
        let status = Command::new("git")
            .arg("clone")
            .args(&clone_args)
            .arg("--depth=1")
            .status()?;
        if !status.success() {
            anyhow::bail!("Failed to clone git repository: {}", git_url);
        }

        // Read the manifest file
        let manifest_path = repo_path.join("nagari.json");
        let manifest = PackageManifest::from_file(&manifest_path)?;

        let version = Version::parse(&manifest.version)?;

        // Collect dependencies (only names and version requirements)
        let dependencies = manifest            .dependencies
            .iter()
            .map(|(dep_name, dep_spec)| {
                let _req = match dep_spec {
                    DependencySpec::Version(v) => VersionReq::parse(v).unwrap_or(VersionReq::STAR),
                    DependencySpec::Detailed {
                        version: Some(v), ..
                    } => VersionReq::parse(v).unwrap_or(VersionReq::STAR),
                    _ => VersionReq::STAR,
                };
                // Use 0.0.0 as placeholder, since we don't resolve transitive git deps here
                (dep_name.clone(), Version::new(0, 0, 0))
            })
            .collect();

        Ok(ResolvedDependency {
            name: name.to_string(),
            version,
            resolved_url: git_url.to_string(),
            integrity: String::new(),
            dependencies,
            dev: false,
            optional: false,
            peer: false,
        })
    }

    async fn get_package_info(&mut self, name: &str) -> Result<&CachedPackageInfo> {
        if !self.cache.package_info.contains_key(name) {
            let package_info = self
                .registry
                .get_package_info(name)
                .await?
                .ok_or_else(|| anyhow::anyhow!("Package {} not found", name))?;

            let mut versions = Vec::new();
            let mut version_info = HashMap::new();

            for (version_str, info) in package_info.versions {
                if let Ok(version) = Version::parse(&version_str) {
                    versions.push(version.clone());
                    version_info.insert(version, info);
                }
            }

            versions.sort();

            self.cache.package_info.insert(
                name.to_string(),
                CachedPackageInfo {
                    versions,
                    version_info,
                },
            );
        }

        Ok(self.cache.package_info.get(name).unwrap())
    }

    fn parse_version_requirement(&self, spec: &DependencySpec) -> Result<VersionReq> {
        let version_str = match spec {
            DependencySpec::Version(version) => version,
            DependencySpec::Detailed {
                version: Some(version),
                ..
            } => version,
            _ => return Err(anyhow::anyhow!("No version specified")),
        };

        VersionReq::parse(version_str)
            .map_err(|e| anyhow::anyhow!("Invalid version requirement '{}': {}", version_str, e))
    }

    fn find_suitable_version(
        &self,
        versions: &[Version],
        requirement: &VersionReq,
        context: &ResolutionContext,
    ) -> Result<Version> {
        let mut suitable_versions: Vec<_> = versions
            .iter()
            .filter(|v| requirement.matches(v) && (context.allow_prereleases || v.pre.is_empty()))
            .cloned()
            .collect();

        if suitable_versions.is_empty() {
            return Err(anyhow::anyhow!(
                "No suitable version found for requirement: {}",
                requirement
            ));
        }

        suitable_versions.sort();

        match context.update_strategy {
            UpdateStrategy::Latest => Ok(suitable_versions.into_iter().next_back().unwrap()),
            _ => Ok(suitable_versions.into_iter().next_back().unwrap()),
        }
    }

    async fn detect_conflicts(&self, _resolution: &mut ResolutionResult) -> Result<()> {
        // TODO: Implement conflict detection logic
        // This would check for version conflicts between dependencies
        Ok(())
    }

    async fn detect_warnings(&self, _resolution: &mut ResolutionResult) -> Result<()> {
        // TODO: Implement warning detection logic
        // This would check for deprecated packages, security vulnerabilities, etc.
        Ok(())
    }
}

impl ResolverCache {
    pub fn new() -> Self {
        Self {
            package_info: HashMap::new(),
            resolutions: HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.package_info.clear();
        self.resolutions.clear();
    }
}

impl Default for ResolutionContext {
    fn default() -> Self {
        Self {
            include_dev: false,
            include_optional: true,
            include_peer: false,
            prefer_latest: false,
            allow_prereleases: false,
            update_strategy: UpdateStrategy::None,
        }
    }
}

impl ResolutionContext {
    pub fn production() -> Self {
        Self {
            include_dev: false,
            include_optional: true,
            include_peer: false,
            ..Default::default()
        }
    }

    pub fn development() -> Self {
        Self {
            include_dev: true,
            include_optional: true,
            include_peer: true,
            ..Default::default()
        }
    }

    pub fn with_update_strategy(mut self, strategy: UpdateStrategy) -> Self {
        self.update_strategy = strategy;
        self
    }

    pub fn allow_prereleases(mut self) -> Self {
        self.allow_prereleases = true;
        self
    }
}
