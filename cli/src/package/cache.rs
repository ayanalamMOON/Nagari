use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use anyhow::Result;
use sha2::{Sha256, Digest};

#[derive(Debug, Clone)]
pub struct PackageCache {
    cache_dir: PathBuf,
    metadata: CacheMetadata,
}

#[derive(Debug, Clone)]
pub struct CacheMetadata {
    pub packages: HashMap<String, CachedPackageInfo>,
    pub integrity_checks: HashMap<String, String>,
    pub access_times: HashMap<String, u64>,
}

#[derive(Debug, Clone)]
pub struct CachedPackageInfo {
    pub name: String,
    pub version: String,
    pub cache_key: String,
    pub extracted_path: PathBuf,
    pub tarball_path: PathBuf,
    pub metadata_path: PathBuf,
    pub size: u64,
    pub cached_at: u64,
    pub last_accessed: u64,
}

impl PackageCache {
    pub fn new<P: AsRef<Path>>(cache_dir: P) -> Result<Self> {
        let cache_dir = cache_dir.as_ref().to_path_buf();

        // Create cache directory structure
        fs::create_dir_all(&cache_dir)?;
        fs::create_dir_all(cache_dir.join("packages"))?;
        fs::create_dir_all(cache_dir.join("tarballs"))?;
        fs::create_dir_all(cache_dir.join("metadata"))?;
        fs::create_dir_all(cache_dir.join("temp"))?;

        let metadata = CacheMetadata::load(&cache_dir).unwrap_or_default();

        Ok(Self {
            cache_dir,
            metadata,
        })
    }

    pub fn get_package(&mut self, name: &str, version: &str) -> Option<&CachedPackageInfo> {
        let cache_key = self.generate_cache_key(name, version);

        if let Some(info) = self.metadata.packages.get(&cache_key) {
            // Update access time
            self.metadata.access_times.insert(
                cache_key.clone(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            );

            Some(info)
        } else {
            None
        }
    }

    pub async fn cache_package(
        &mut self,
        name: &str,
        version: &str,
        tarball_data: &[u8],
        metadata: serde_json::Value,
    ) -> Result<CachedPackageInfo> {
        let cache_key = self.generate_cache_key(name, version);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Calculate integrity hash
        let mut hasher = Sha256::new();
        hasher.update(tarball_data);
        let integrity = format!("sha256-{}", base64::encode(hasher.finalize()));

        // Define paths
        let tarball_path = self.cache_dir
            .join("tarballs")
            .join(format!("{}.tgz", cache_key));
        let extracted_path = self.cache_dir
            .join("packages")
            .join(&cache_key);
        let metadata_path = self.cache_dir
            .join("metadata")
            .join(format!("{}.json", cache_key));

        // Save tarball
        fs::write(&tarball_path, tarball_data)?;

        // Extract tarball
        self.extract_tarball(&tarball_path, &extracted_path).await?;

        // Save metadata
        fs::write(&metadata_path, serde_json::to_string_pretty(&metadata)?)?;

        // Create cache info
        let cache_info = CachedPackageInfo {
            name: name.to_string(),
            version: version.to_string(),
            cache_key: cache_key.clone(),
            extracted_path,
            tarball_path,
            metadata_path,
            size: tarball_data.len() as u64,
            cached_at: now,
            last_accessed: now,
        };

        // Update metadata
        self.metadata.packages.insert(cache_key.clone(), cache_info.clone());
        self.metadata.integrity_checks.insert(cache_key.clone(), integrity);
        self.metadata.access_times.insert(cache_key, now);

        // Save metadata
        self.save_metadata()?;

        Ok(cache_info)
    }

    pub fn remove_package(&mut self, name: &str, version: &str) -> Result<bool> {
        let cache_key = self.generate_cache_key(name, version);

        if let Some(info) = self.metadata.packages.remove(&cache_key) {
            // Remove files
            if info.tarball_path.exists() {
                fs::remove_file(&info.tarball_path)?;
            }
            if info.extracted_path.exists() {
                fs::remove_dir_all(&info.extracted_path)?;
            }
            if info.metadata_path.exists() {
                fs::remove_file(&info.metadata_path)?;
            }

            // Remove from metadata
            self.metadata.integrity_checks.remove(&cache_key);
            self.metadata.access_times.remove(&cache_key);

            self.save_metadata()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn clear_cache(&mut self) -> Result<()> {
        // Remove all cached packages
        for cache_key in self.metadata.packages.keys().cloned().collect::<Vec<_>>() {
            if let Some(info) = self.metadata.packages.get(&cache_key) {
                let name = &info.name;
                let version = &info.version;
                self.remove_package(name, version)?;
            }
        }

        // Clear temp directory
        let temp_dir = self.cache_dir.join("temp");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir)?;
            fs::create_dir_all(&temp_dir)?;
        }

        Ok(())
    }

    pub fn prune_cache(&mut self, max_age_days: u32, max_size_mb: u64) -> Result<Vec<String>> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let max_age_seconds = max_age_days as u64 * 24 * 60 * 60;
        let max_size_bytes = max_size_mb * 1024 * 1024;

        let mut removed = Vec::new();
        let mut candidates_for_removal = Vec::new();

        // Find packages to remove based on age
        for (cache_key, info) in &self.metadata.packages {
            let age = now - info.last_accessed;
            if age > max_age_seconds {
                candidates_for_removal.push((cache_key.clone(), info.clone()));
            }
        }

        // If still over size limit, remove least recently used packages
        let current_size: u64 = self.metadata.packages.values().map(|info| info.size).sum();
        if current_size > max_size_bytes {
            let mut all_packages: Vec<_> = self.metadata.packages.iter().collect();
            all_packages.sort_by_key(|(_, info)| info.last_accessed);

            let mut size_to_remove = current_size - max_size_bytes;
            for (cache_key, info) in all_packages {
                if size_to_remove == 0 {
                    break;
                }

                candidates_for_removal.push((cache_key.clone(), info.clone()));
                size_to_remove = size_to_remove.saturating_sub(info.size);
            }
        }

        // Remove the packages
        for (cache_key, info) in candidates_for_removal {
            if self.remove_package(&info.name, &info.version)? {
                removed.push(format!("{}@{}", info.name, info.version));
            }
        }

        Ok(removed)
    }

    pub fn verify_integrity(&self) -> Result<Vec<String>> {
        let mut corrupted = Vec::new();

        for (cache_key, info) in &self.metadata.packages {
            // Check if files exist
            if !info.tarball_path.exists() || !info.extracted_path.exists() {
                corrupted.push(format!("{}@{} (missing files)", info.name, info.version));
                continue;
            }

            // Verify tarball integrity
            if let Some(expected_integrity) = self.metadata.integrity_checks.get(cache_key) {
                let tarball_data = fs::read(&info.tarball_path)?;
                let mut hasher = Sha256::new();
                hasher.update(&tarball_data);
                let actual_integrity = format!("sha256-{}", base64::encode(hasher.finalize()));

                if &actual_integrity != expected_integrity {
                    corrupted.push(format!("{}@{} (integrity mismatch)", info.name, info.version));
                }
            }
        }

        Ok(corrupted)
    }

    pub fn get_cache_stats(&self) -> CacheStats {
        let total_packages = self.metadata.packages.len();
        let total_size: u64 = self.metadata.packages.values().map(|info| info.size).sum();

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let recently_accessed = self.metadata.access_times
            .values()
            .filter(|&&time| now - time < 7 * 24 * 60 * 60) // Last 7 days
            .count();

        CacheStats {
            total_packages,
            total_size_bytes: total_size,
            recently_accessed_packages: recently_accessed,
            cache_directory: self.cache_dir.clone(),
        }
    }

    pub fn list_packages(&self) -> Vec<&CachedPackageInfo> {
        self.metadata.packages.values().collect()
    }

    fn generate_cache_key(&self, name: &str, version: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(format!("{}@{}", name, version));
        format!("{:x}", hasher.finalize())[..16].to_string()
    }

    async fn extract_tarball(&self, tarball_path: &Path, extract_path: &Path) -> Result<()> {
        // Create extraction directory
        fs::create_dir_all(extract_path)?;

        // Read tarball
        let tarball_data = fs::read(tarball_path)?;

        // Extract using tar
        let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(tarball_data.as_slice()));
        archive.unpack(extract_path)?;

        Ok(())
    }

    fn save_metadata(&self) -> Result<()> {
        let metadata_path = self.cache_dir.join("cache-metadata.json");
        let metadata_json = serde_json::to_string_pretty(&self.metadata)?;
        fs::write(metadata_path, metadata_json)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_packages: usize,
    pub total_size_bytes: u64,
    pub recently_accessed_packages: usize,
    pub cache_directory: PathBuf,
}

impl CacheMetadata {
    fn load(cache_dir: &Path) -> Result<Self> {
        let metadata_path = cache_dir.join("cache-metadata.json");
        if metadata_path.exists() {
            let content = fs::read_to_string(metadata_path)?;
            let metadata: CacheMetadata = serde_json::from_str(&content)?;
            Ok(metadata)
        } else {
            Ok(Self::default())
        }
    }
}

impl Default for CacheMetadata {
    fn default() -> Self {
        Self {
            packages: HashMap::new(),
            integrity_checks: HashMap::new(),
            access_times: HashMap::new(),
        }
    }
}

impl std::fmt::Display for CacheStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Package Cache Statistics:")?;
        writeln!(f, "  Cache directory: {}", self.cache_directory.display())?;
        writeln!(f, "  Total packages: {}", self.total_packages)?;
        writeln!(f, "  Total size: {:.2} MB", self.total_size_bytes as f64 / 1024.0 / 1024.0)?;
        writeln!(f, "  Recently accessed: {}", self.recently_accessed_packages)
    }
}

// Add required dependencies to implement this module
use serde::{Deserialize, Serialize};

impl Serialize for CacheMetadata {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("CacheMetadata", 3)?;
        state.serialize_field("packages", &self.packages)?;
        state.serialize_field("integrity_checks", &self.integrity_checks)?;
        state.serialize_field("access_times", &self.access_times)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for CacheMetadata {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct CacheMetadataHelper {
            packages: HashMap<String, CachedPackageInfo>,
            integrity_checks: HashMap<String, String>,
            access_times: HashMap<String, u64>,
        }

        let helper = CacheMetadataHelper::deserialize(deserializer)?;
        Ok(CacheMetadata {
            packages: helper.packages,
            integrity_checks: helper.integrity_checks,
            access_times: helper.access_times,
        })
    }
}

impl Serialize for CachedPackageInfo {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("CachedPackageInfo", 8)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("version", &self.version)?;
        state.serialize_field("cache_key", &self.cache_key)?;
        state.serialize_field("extracted_path", &self.extracted_path)?;
        state.serialize_field("tarball_path", &self.tarball_path)?;
        state.serialize_field("metadata_path", &self.metadata_path)?;
        state.serialize_field("size", &self.size)?;
        state.serialize_field("cached_at", &self.cached_at)?;
        state.serialize_field("last_accessed", &self.last_accessed)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for CachedPackageInfo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct CachedPackageInfoHelper {
            name: String,
            version: String,
            cache_key: String,
            extracted_path: PathBuf,
            tarball_path: PathBuf,
            metadata_path: PathBuf,
            size: u64,
            cached_at: u64,
            last_accessed: u64,
        }

        let helper = CachedPackageInfoHelper::deserialize(deserializer)?;
        Ok(CachedPackageInfo {
            name: helper.name,
            version: helper.version,
            cache_key: helper.cache_key,
            extracted_path: helper.extracted_path,
            tarball_path: helper.tarball_path,
            metadata_path: helper.metadata_path,
            size: helper.size,
            cached_at: helper.cached_at,
            last_accessed: helper.last_accessed,
        })
    }
}
