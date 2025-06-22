#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageManifest {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub license: Option<String>,
    pub repository: Option<String>,
    pub homepage: Option<String>,
    pub keywords: Vec<String>,

    pub main: Option<String>,
    pub exports: Option<HashMap<String, String>>,
    pub bin: Option<HashMap<String, String>>,

    pub dependencies: HashMap<String, DependencySpec>,
    pub dev_dependencies: HashMap<String, DependencySpec>,
    pub peer_dependencies: HashMap<String, DependencySpec>,
    pub optional_dependencies: HashMap<String, DependencySpec>,

    pub scripts: HashMap<String, String>,
    pub nagari: Option<NagariConfig>,
    pub engines: Option<EngineRequirements>,
    pub os: Option<Vec<String>>,
    pub cpu: Option<Vec<String>>,

    pub files: Option<Vec<String>>,
    pub publish_config: Option<PublishConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DependencySpec {
    Version(String),
    Detailed {
        version: Option<String>,
        path: Option<PathBuf>,
        git: Option<String>,
        branch: Option<String>,
        tag: Option<String>,
        registry: Option<String>,
        optional: Option<bool>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NagariConfig {
    pub source_dir: String,
    pub output_dir: String,
    pub target: String,
    pub module_format: String,
    pub compiler_options: Option<CompilerOptions>,
    pub runtime_options: Option<RuntimeOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilerOptions {
    pub strict: Option<bool>,
    pub debug: Option<bool>,
    pub optimize: Option<bool>,
    pub emit_source_maps: Option<bool>,
    pub target_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeOptions {
    pub polyfills: Option<Vec<String>>,
    pub interop_mode: Option<String>,
    pub async_runtime: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineRequirements {
    pub nagari: Option<String>,
    pub node: Option<String>,
    pub npm: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishConfig {
    pub registry: Option<String>,
    pub access: Option<String>,
    pub tag: Option<String>,
}

impl PackageManifest {
    pub fn new(name: String, version: String) -> Self {
        Self {
            name,
            version,
            description: None,
            author: None,
            license: Some("ISC".to_string()),
            repository: None,
            homepage: None,
            keywords: Vec::new(),
            main: Some("src/main.nag".to_string()),
            exports: None,
            bin: None,
            dependencies: HashMap::new(),
            dev_dependencies: HashMap::new(),
            peer_dependencies: HashMap::new(),
            optional_dependencies: HashMap::new(),
            scripts: HashMap::new(),
            nagari: Some(NagariConfig::default()),
            engines: None,
            os: None,
            cpu: None,
            files: None,
            publish_config: None,
        }
    }

    pub fn from_file(path: &PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let manifest: PackageManifest = serde_json::from_str(&content)?;
        Ok(manifest)
    }

    pub fn to_file(&self, path: &PathBuf) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn add_dependency(&mut self, name: String, spec: DependencySpec) {
        self.dependencies.insert(name, spec);
    }

    pub fn add_dev_dependency(&mut self, name: String, spec: DependencySpec) {
        self.dev_dependencies.insert(name, spec);
    }

    pub fn remove_dependency(&mut self, name: &str) -> bool {
        self.dependencies.remove(name).is_some()
            || self.dev_dependencies.remove(name).is_some()
            || self.peer_dependencies.remove(name).is_some()
            || self.optional_dependencies.remove(name).is_some()
    }

    pub fn get_all_dependencies(&self) -> HashMap<String, &DependencySpec> {
        let mut deps = HashMap::new();

        for (name, spec) in &self.dependencies {
            deps.insert(name.clone(), spec);
        }

        for (name, spec) in &self.dev_dependencies {
            deps.insert(name.clone(), spec);
        }

        for (name, spec) in &self.peer_dependencies {
            deps.insert(name.clone(), spec);
        }

        for (name, spec) in &self.optional_dependencies {
            deps.insert(name.clone(), spec);
        }

        deps
    }
}

impl Default for NagariConfig {
    fn default() -> Self {
        Self {
            source_dir: "src".to_string(),
            output_dir: "dist".to_string(),
            target: "es2020".to_string(),
            module_format: "esm".to_string(),
            compiler_options: None,
            runtime_options: None,
        }
    }
}

impl DependencySpec {
    pub fn version(version: &str) -> Self {
        DependencySpec::Version(version.to_string())
    }

    pub fn path(path: PathBuf) -> Self {
        DependencySpec::Detailed {
            version: None,
            path: Some(path),
            git: None,
            branch: None,
            tag: None,
            registry: None,
            optional: None,
        }
    }

    pub fn git(url: &str, branch: Option<&str>, tag: Option<&str>) -> Self {
        DependencySpec::Detailed {
            version: None,
            path: None,
            git: Some(url.to_string()),
            branch: branch.map(|s| s.to_string()),
            tag: tag.map(|s| s.to_string()),
            registry: None,
            optional: None,
        }
    }

    pub fn get_version(&self) -> Option<&str> {
        match self {
            DependencySpec::Version(version) => Some(version),
            DependencySpec::Detailed { version, .. } => version.as_deref(),
        }
    }

    pub fn is_optional(&self) -> bool {
        match self {
            DependencySpec::Version(_) => false,
            DependencySpec::Detailed { optional, .. } => optional.unwrap_or(false),
        }
    }
}
