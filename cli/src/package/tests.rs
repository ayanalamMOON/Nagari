use crate::package::{PackageManager, Manifest, Package, VersionReq};
use std::collections::HashMap;
use tempfile::TempDir;
use tokio;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_package_manager_new() {
        let temp_dir = TempDir::new().unwrap();
        let manager = PackageManager::new(temp_dir.path().to_path_buf()).await.unwrap();

        assert_eq!(manager.get_cache_dir(), temp_dir.path().join(".nagari"));
        assert!(manager.get_cache_dir().exists());
    }

    #[tokio::test]
    async fn test_manifest_creation() {
        let manifest = Manifest::new("test-package".to_string(), "1.0.0".to_string());

        assert_eq!(manifest.package.name, "test-package");
        assert_eq!(manifest.package.version, "1.0.0");
        assert!(manifest.dependencies.is_empty());
        assert!(manifest.dev_dependencies.is_empty());
    }

    #[tokio::test]
    async fn test_manifest_add_dependency() {
        let mut manifest = Manifest::new("test-package".to_string(), "1.0.0".to_string());

        manifest.add_dependency("lodash".to_string(), VersionReq::parse("^4.0.0").unwrap());

        assert_eq!(manifest.dependencies.len(), 1);
        assert!(manifest.dependencies.contains_key("lodash"));
    }

    #[tokio::test]
    async fn test_manifest_remove_dependency() {
        let mut manifest = Manifest::new("test-package".to_string(), "1.0.0".to_string());

        manifest.add_dependency("lodash".to_string(), VersionReq::parse("^4.0.0").unwrap());
        manifest.remove_dependency("lodash");

        assert!(manifest.dependencies.is_empty());
    }

    #[tokio::test]
    async fn test_manifest_serialization() {
        let mut manifest = Manifest::new("test-package".to_string(), "1.0.0".to_string());
        manifest.package.description = Some("A test package".to_string());
        manifest.package.author = Some("Test Author".to_string());
        manifest.add_dependency("dep1".to_string(), VersionReq::parse("1.0.0").unwrap());

        let toml_str = manifest.to_toml().unwrap();
        let parsed_manifest = Manifest::from_toml(&toml_str).unwrap();

        assert_eq!(parsed_manifest.package.name, "test-package");
        assert_eq!(parsed_manifest.package.version, "1.0.0");
        assert_eq!(parsed_manifest.package.description, Some("A test package".to_string()));
        assert_eq!(parsed_manifest.dependencies.len(), 1);
    }

    #[tokio::test]
    async fn test_package_manager_install() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = PackageManager::new(temp_dir.path().to_path_buf()).await.unwrap();

        // Create a mock manifest
        let mut manifest = Manifest::new("test-project".to_string(), "1.0.0".to_string());
        manifest.add_dependency("test-dep".to_string(), VersionReq::parse("1.0.0").unwrap());

        // Note: This would normally contact a registry, but for testing we'd mock it
        // let result = manager.install(&manifest).await;
        // assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_package_validation() {
        let package = Package {
            name: "test-package".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Test package".to_string()),
            author: Some("Test Author".to_string()),
            license: Some("MIT".to_string()),
            repository: None,
            homepage: None,
            keywords: vec![],
            main: Some("index.nag".to_string()),
            files: vec!["src/".to_string(), "README.md".to_string()],
        };

        assert!(package.validate().is_ok());
    }

    #[tokio::test]
    async fn test_package_validation_invalid_name() {
        let package = Package {
            name: "".to_string(), // Invalid empty name
            version: "1.0.0".to_string(),
            description: None,
            author: None,
            license: None,
            repository: None,
            homepage: None,
            keywords: vec![],
            main: None,
            files: vec![],
        };

        assert!(package.validate().is_err());
    }

    #[tokio::test]
    async fn test_package_validation_invalid_version() {
        let package = Package {
            name: "test-package".to_string(),
            version: "invalid-version".to_string(), // Invalid version
            description: None,
            author: None,
            license: None,
            repository: None,
            homepage: None,
            keywords: vec![],
            main: None,
            files: vec![],
        };

        assert!(package.validate().is_err());
    }
}

#[cfg(test)]
mod resolver_tests {
    use super::*;
    use crate::package::DependencyResolver;

    #[tokio::test]
    async fn test_dependency_resolver() {
        let resolver = DependencyResolver::new();

        let mut dependencies = HashMap::new();
        dependencies.insert("package-a".to_string(), VersionReq::parse("1.0.0").unwrap());
        dependencies.insert("package-b".to_string(), VersionReq::parse("^2.0.0").unwrap());

        // Mock resolution - in reality this would contact registries
        // let result = resolver.resolve(dependencies).await;
        // assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_circular_dependency_detection() {
        let resolver = DependencyResolver::new();

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
        let cache = PackageCache::new(temp_dir.path().to_path_buf()).await.unwrap();

        let package_name = "test-package";
        let version = "1.0.0";
        let package_data = b"mock package data";

        // Test storing package
        cache.store_package(package_name, version, package_data).await.unwrap();

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
        let cache = PackageCache::new(temp_dir.path().to_path_buf()).await.unwrap();

        // Store multiple packages
        for i in 0..5 {
            let package_name = format!("package-{}", i);
            let version = "1.0.0";
            let package_data = format!("data for package {}", i).as_bytes();

            cache.store_package(&package_name, version, package_data).await.unwrap();
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
    use crate::package::{Lockfile, LockEntry};

    #[tokio::test]
    async fn test_lockfile_creation() {
        let lockfile = Lockfile::new();
        assert!(lockfile.packages.is_empty());
    }

    #[tokio::test]
    async fn test_lockfile_add_entry() {
        let mut lockfile = Lockfile::new();

        let entry = LockEntry {
            name: "test-package".to_string(),
            version: "1.0.0".to_string(),
            resolved: "https://registry.example.com/test-package/-/test-package-1.0.0.tgz".to_string(),
            integrity: "sha512-...".to_string(),
            dependencies: HashMap::new(),
        };

        lockfile.add_entry(entry.clone());

        assert_eq!(lockfile.packages.len(), 1);
        assert!(lockfile.packages.contains_key("test-package"));
        assert_eq!(lockfile.packages["test-package"].version, "1.0.0");
    }

    #[tokio::test]
    async fn test_lockfile_serialization() {
        let mut lockfile = Lockfile::new();

        let entry = LockEntry {
            name: "test-package".to_string(),
            version: "1.0.0".to_string(),
            resolved: "https://registry.example.com/test-package/-/test-package-1.0.0.tgz".to_string(),
            integrity: "sha512-test".to_string(),
            dependencies: HashMap::new(),
        };

        lockfile.add_entry(entry);

        let json_str = lockfile.to_json().unwrap();
        let parsed_lockfile = Lockfile::from_json(&json_str).unwrap();

        assert_eq!(parsed_lockfile.packages.len(), 1);
        assert!(parsed_lockfile.packages.contains_key("test-package"));
    }
}
