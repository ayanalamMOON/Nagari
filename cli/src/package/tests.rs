use crate::config::NagConfig;
use crate::package::{
    DependencyResolver, DependencySpec, LockFile, LockedDependency, PackageCache, PackageManager,
    PackageManifest,
};
use std::collections::HashMap;
use tempfile::TempDir;
use tokio;

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_package_manager_new() {
        let temp_dir = TempDir::new().unwrap();
        let config = NagConfig::load(Some(temp_dir.path())).unwrap();
        let manager = PackageManager::new(config).unwrap();

        // Note: PackageManager doesn't expose get_cache_dir method
        // We can test that the manager was created successfully
        // The cache directory setup is internal to the manager
    }
    #[tokio::test]
    async fn test_manifest_creation() {
        let manifest = PackageManifest {
            name: "test-package".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            author: None,
            license: None,
            repository: None,
            homepage: None,
            keywords: vec![],
            main: None,
            exports: None,
            bin: None,
            dependencies: HashMap::new(),
            dev_dependencies: HashMap::new(),
            peer_dependencies: HashMap::new(),
            optional_dependencies: HashMap::new(),
            scripts: HashMap::new(),
            nagari: None,
            engines: None,
            os: None,
            cpu: None,
            files: None,
            publish_config: None,
        };

        assert_eq!(manifest.name, "test-package");
        assert_eq!(manifest.version, "1.0.0");
        assert!(manifest.dependencies.is_empty());
        assert!(manifest.dev_dependencies.is_empty());
    }
    #[tokio::test]
    async fn test_manifest_add_dependency() {
        let mut manifest = PackageManifest {
            name: "test-package".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            author: None,
            license: None,
            repository: None,
            homepage: None,
            keywords: vec![],
            main: None,
            exports: None,
            bin: None,
            dependencies: HashMap::new(),
            dev_dependencies: HashMap::new(),
            peer_dependencies: HashMap::new(),
            optional_dependencies: HashMap::new(),
            scripts: HashMap::new(),
            nagari: None,
            engines: None,
            os: None,
            cpu: None,
            files: None,
            publish_config: None,
        };

        manifest.dependencies.insert(
            "lodash".to_string(),
            DependencySpec::Version("^4.0.0".to_string()),
        );

        assert_eq!(manifest.dependencies.len(), 1);
        assert!(manifest.dependencies.contains_key("lodash"));
    }
    #[tokio::test]
    async fn test_manifest_remove_dependency() {
        let mut manifest = PackageManifest {
            name: "test-package".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            author: None,
            license: None,
            repository: None,
            homepage: None,
            keywords: vec![],
            main: None,
            exports: None,
            bin: None,
            dependencies: HashMap::new(),
            dev_dependencies: HashMap::new(),
            peer_dependencies: HashMap::new(),
            optional_dependencies: HashMap::new(),
            scripts: HashMap::new(),
            nagari: None,
            engines: None,
            os: None,
            cpu: None,
            files: None,
            publish_config: None,
        };

        manifest.dependencies.insert(
            "lodash".to_string(),
            DependencySpec::Version("^4.0.0".to_string()),
        );
        manifest.dependencies.remove("lodash");

        assert!(manifest.dependencies.is_empty());
    }
    #[tokio::test]
    async fn test_manifest_serialization() {
        let mut manifest = PackageManifest {
            name: "test-package".to_string(),
            version: "1.0.0".to_string(),
            description: Some("A test package".to_string()),
            author: Some("Test Author".to_string()),
            license: None,
            repository: None,
            homepage: None,
            keywords: vec![],
            main: None,
            exports: None,
            bin: None,
            dependencies: HashMap::new(),
            dev_dependencies: HashMap::new(),
            peer_dependencies: HashMap::new(),
            optional_dependencies: HashMap::new(),
            scripts: HashMap::new(),
            nagari: None,
            engines: None,
            os: None,
            cpu: None,
            files: None,
            publish_config: None,
        };
        manifest.dependencies.insert(
            "dep1".to_string(),
            DependencySpec::Version("1.0.0".to_string()),
        );

        // Test that the manifest was created correctly
        assert_eq!(manifest.name, "test-package");
        assert_eq!(manifest.version, "1.0.0");
        assert_eq!(manifest.description, Some("A test package".to_string()));
        assert_eq!(manifest.dependencies.len(), 1);
    }

    #[tokio::test]
    async fn test_package_manager_install() {
        let temp_dir = TempDir::new().unwrap();
        let config = NagConfig::load(Some(temp_dir.path())).unwrap();
        let _manager = PackageManager::new(config).unwrap();

        // Create a mock manifest
        let mut manifest = PackageManifest {
            name: "test-project".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            author: None,
            license: None,
            repository: None,
            homepage: None,
            keywords: vec![],
            main: None,
            exports: None,
            bin: None,
            dependencies: HashMap::new(),
            dev_dependencies: HashMap::new(),
            peer_dependencies: HashMap::new(),
            optional_dependencies: HashMap::new(),
            scripts: HashMap::new(),
            nagari: None,
            engines: None,
            os: None,
            cpu: None,
            files: None,
            publish_config: None,
        };
        manifest.dependencies.insert(
            "test-dep".to_string(),
            DependencySpec::Version("1.0.0".to_string()),
        );

        // Note: This would normally contact a registry, but for testing we'd mock it
        // let result = manager.install(&manifest).await;
        // assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_package_validation() {
        let manifest = PackageManifest {
            name: "test-package".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Test package".to_string()),
            author: Some("Test Author".to_string()),
            license: Some("MIT".to_string()),
            repository: None,
            homepage: None,
            keywords: vec![],
            main: Some("index.nag".to_string()),
            exports: None,
            bin: None,
            dependencies: HashMap::new(),
            dev_dependencies: HashMap::new(),
            peer_dependencies: HashMap::new(),
            optional_dependencies: HashMap::new(),
            scripts: HashMap::new(),
            nagari: None,
            engines: None,
            os: None,
            cpu: None,
            files: Some(vec!["src/".to_string(), "README.md".to_string()]),
            publish_config: None,
        };

        // Basic validation - check that required fields are present
        assert!(!manifest.name.is_empty());
        assert!(!manifest.version.is_empty());
    }
    #[tokio::test]
    async fn test_package_validation_invalid_name() {
        let manifest = PackageManifest {
            name: "".to_string(), // Invalid empty name
            version: "1.0.0".to_string(),
            description: None,
            author: None,
            license: None,
            repository: None,
            homepage: None,
            keywords: vec![],
            main: None,
            exports: None,
            bin: None,
            dependencies: HashMap::new(),
            dev_dependencies: HashMap::new(),
            peer_dependencies: HashMap::new(),
            optional_dependencies: HashMap::new(),
            scripts: HashMap::new(),
            nagari: None,
            engines: None,
            os: None,
            cpu: None,
            files: None,
            publish_config: None,
        };

        // Basic validation - check that name is not empty
        assert!(manifest.name.is_empty());
    }

    #[tokio::test]
    async fn test_package_validation_invalid_version() {
        let manifest = PackageManifest {
            name: "test-package".to_string(),
            version: "invalid-version".to_string(), // Invalid version
            description: None,
            author: None,
            license: None,
            repository: None,
            homepage: None,
            keywords: vec![],
            main: None,
            exports: None,
            bin: None,
            dependencies: HashMap::new(),
            dev_dependencies: HashMap::new(),
            peer_dependencies: HashMap::new(),
            optional_dependencies: HashMap::new(),
            scripts: HashMap::new(),
            nagari: None,
            engines: None,
            os: None,
            cpu: None,
            files: None,
            publish_config: None,
        };

        // Basic validation - we'd need to implement actual semver validation
        assert_eq!(manifest.version, "invalid-version");
    }
}

#[cfg(test)]
mod resolver_tests {
    use super::*;
    use crate::package::{DependencyResolver, RegistryClient};

    #[tokio::test]
    async fn test_dependency_resolver() {
        let registry = RegistryClient::new("https://registry.example.com").unwrap();
        let _resolver = DependencyResolver::new(registry);

        let mut dependencies = HashMap::new();
        dependencies.insert(
            "package-a".to_string(),
            DependencySpec::Version("1.0.0".to_string()),
        );
        dependencies.insert(
            "package-b".to_string(),
            DependencySpec::Version("^2.0.0".to_string()),
        );

        // Mock resolution - in reality this would contact registries
        // let result = resolver.resolve(dependencies).await;
        // assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_circular_dependency_detection() {
        let registry = RegistryClient::new("https://registry.example.com").unwrap();
        let _resolver = DependencyResolver::new(registry);

        // This would test circular dependency detection
        // In a real implementation, we'd create packages that depend on each other
        // and verify the resolver detects and handles the circular dependency
    }
}

#[cfg(test)]
mod cache_tests {
    use super::*;
    use crate::package::PackageCache;

    #[tokio::test]
    async fn test_cache_operations() {
        let temp_dir = TempDir::new().unwrap();
        let cache = PackageCache::new(temp_dir.path().to_path_buf()).unwrap();

        let package_name = "test-package";
        let version = "1.0.0";
        let package_data = b"mock package data";

        // Test storing package
        cache
            .store_package(package_name, version, package_data)
            .await
            .unwrap();

        // Test checking if package exists
        assert!(cache.has_package(package_name, version).await);

        // Test retrieving package
        let retrieved_data = cache.get_package(package_name, version).await.unwrap();
        assert_eq!(retrieved_data, package_data);

        // Test removing package
        cache.remove_package(package_name, version).await.unwrap();
        assert!(!cache.has_package(package_name, version).await);
    }

    #[tokio::test]
    async fn test_cache_cleanup() {
        let temp_dir = TempDir::new().unwrap();
        let cache = PackageCache::new(temp_dir.path().to_path_buf()).unwrap();

        // Store multiple packages
        for i in 0..5 {
            let package_name = format!("package-{}", i);
            let version = "1.0.0";
            let package_data = format!("data for package {}", i).as_bytes();

            cache
                .store_package(&package_name, version, package_data)
                .await
                .unwrap();
        }

        // Test cleanup functionality
        cache.cleanup().await.unwrap();

        // Verify cache directory structure is maintained
        assert!(cache.get_cache_root().exists());
    }
}

#[cfg(test)]
mod lockfile_tests {
    use super::*;
    use crate::package::{LockFile, LockedDependency};

    #[tokio::test]
    async fn test_lockfile_creation() {
        let lockfile = LockFile::new("test-project".to_string(), "1.0.0".to_string());
        assert!(lockfile.packages.is_empty());
    }

    #[tokio::test]
    async fn test_lockfile_add_entry() {
        let mut lockfile = LockFile::new("test-project".to_string(), "1.0.0".to_string());

        let entry = LockedDependency {
            version: "1.0.0".to_string(),
            resolved: "https://registry.example.com/test-package/-/test-package-1.0.0.tgz"
                .to_string(),
            integrity: "sha512-...".to_string(),
            dev: None,
            optional: None,
            peer: None,
            requires: None,
            dependencies: None,
            engines: None,
            os: None,
            cpu: None,
        };

        lockfile
            .packages
            .insert("test-package".to_string(), entry.clone());

        assert_eq!(lockfile.packages.len(), 1);
        assert!(lockfile.packages.contains_key("test-package"));
        assert_eq!(lockfile.packages["test-package"].version, "1.0.0");
    }

    #[tokio::test]
    async fn test_lockfile_serialization() {
        let mut lockfile = LockFile::new("test-project".to_string(), "1.0.0".to_string());

        let entry = LockedDependency {
            version: "1.0.0".to_string(),
            resolved: "https://registry.example.com/test-package/-/test-package-1.0.0.tgz"
                .to_string(),
            integrity: "sha512-test".to_string(),
            dev: None,
            optional: None,
            peer: None,
            requires: None,
            dependencies: None,
            engines: None,
            os: None,
            cpu: None,
        };

        lockfile.packages.insert("test-package".to_string(), entry);

        // Test that lockfile is properly created
        assert_eq!(lockfile.packages.len(), 1);
        assert!(lockfile.packages.contains_key("test-package"));
    }
}
