pub mod cache;
pub mod lockfile;
pub mod manager;
pub mod manifest;
pub mod registry;
pub mod resolver;
pub mod utils;

#[cfg(test)]
pub mod tests;

pub use manager::PackageManager;
// Temporarily commented out until package functionality is fully integrated
// pub use manifest::{PackageManifest, NagariConfig, DependencySpec};
// pub use registry::{RegistryClient, PackageInfo, VersionInfo};
// pub use resolver::{DependencyResolver, ResolutionResult};
// pub use cache::PackageCache;
// pub use lockfile::{LockFile, LockedDependency};
// pub use utils::PackageUtils;
