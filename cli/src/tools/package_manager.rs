use crate::config::NagConfig;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageJson {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub main: Option<String>,
    pub author: Option<String>,
    pub license: Option<String>,
    pub repository: Option<String>,
    pub keywords: Vec<String>,
    pub dependencies: HashMap<String, String>,
    pub dev_dependencies: HashMap<String, String>,
    pub scripts: HashMap<String, String>,
    pub nagari: Option<NagariConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NagariConfig {
    pub source_dir: String,
    pub output_dir: String,
    pub target: String,
    pub module_format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockFile {
    pub version: String,
    pub dependencies: HashMap<String, LockedDependency>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedDependency {
    pub version: String,
    pub resolved: String,
    pub integrity: String,
    pub dependencies: Option<HashMap<String, String>>,
}

pub async fn init_package(yes: bool, _config: &NagConfig) -> Result<()> {
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

    let mut package = PackageJson {
        name: "nagari-project".to_string(),
        version: "0.1.0".to_string(),
        description: Some("A Nagari project".to_string()),
        main: Some("main.nag".to_string()),
        author: None,
        license: Some("MIT".to_string()),
        repository: None,
        keywords: vec!["nagari".to_string()],
        dependencies: HashMap::new(),
        dev_dependencies: HashMap::new(),
        scripts: HashMap::new(),
        nagari: Some(NagariConfig {
            source_dir: "src".to_string(),
            output_dir: "dist".to_string(),
            target: "js".to_string(),
            module_format: "es6".to_string(),
        }),
    };

    // Add default scripts
    package.scripts.insert("build".to_string(), "nag build".to_string());
    package.scripts.insert("start".to_string(), "nag run main.nag".to_string());
    package.scripts.insert("test".to_string(), "nag test".to_string());
    package.scripts.insert("lint".to_string(), "nag lint src/".to_string());
    package.scripts.insert("format".to_string(), "nag format src/".to_string());

    if !yes {
        // Interactive prompts
        package.name = prompt_with_default("Package name", &package.name)?;
        package.version = prompt_with_default("Version", &package.version)?;

        if let Some(desc) = prompt_optional("Description")? {
            package.description = Some(desc);
        }

        if let Some(author) = prompt_optional("Author")? {
            package.author = Some(author);
        }

        package.license = Some(prompt_with_default("License", package.license.as_ref().unwrap())?);
    }

    // Write package file
    let content = serde_json::to_string_pretty(&package)?;
    std::fs::write(&package_file, content)?;

    println!("✓ Created nagari.json");

    // Create .gitignore if it doesn't exist
    let gitignore_file = PathBuf::from(".gitignore");
    if !gitignore_file.exists() {
        let gitignore_content = "# Nagari build outputs\ndist/\n*.js.map\n\n# Dependencies\nnode_modules/\nnag_modules/\n\n# IDE\n.vscode/\n.idea/\n\n# OS\n.DS_Store\nThumbs.db\n";
        std::fs::write(&gitignore_file, gitignore_content)?;
        println!("✓ Created .gitignore");
    }

    Ok(())
}

pub async fn install_packages(
    packages: Vec<String>,
    dev: bool,
    global: bool,
    _exact: bool,
    _config: &NagConfig,
) -> Result<()> {
    if global {
        println!("Global package installation not yet implemented");
        return Ok(());
    }

    let mut package = load_package_json()?;

    for package_spec in packages {
        let (name, version) = parse_package_spec(&package_spec);

        println!("Installing {}@{}...", name, version);

        // TODO: Implement actual package resolution and download
        // For now, just add to package.json
        let target_deps = if dev {
            &mut package.dev_dependencies
        } else {
            &mut package.dependencies
        };

        target_deps.insert(name.clone(), version.clone());

        println!("✓ Added {} to {}", name, if dev { "devDependencies" } else { "dependencies" });
    }

    save_package_json(&package)?;
    update_lock_file(&package).await?;

    Ok(())
}

pub async fn add_package(
    package: String,
    version: Option<String>,
    dev: bool,
    config: &NagConfig,
) -> Result<()> {
    let version = version.unwrap_or_else(|| "latest".to_string());
    install_packages(vec![format!("{}@{}", package, version)], dev, false, false, config).await
}

pub async fn remove_packages(packages: Vec<String>, _config: &NagConfig) -> Result<()> {
    let mut package = load_package_json()?;

    for pkg_name in packages {
        let mut removed = false;

        if package.dependencies.remove(&pkg_name).is_some() {
            println!("✓ Removed {} from dependencies", pkg_name);
            removed = true;
        }

        if package.dev_dependencies.remove(&pkg_name).is_some() {
            println!("✓ Removed {} from devDependencies", pkg_name);
            removed = true;
        }

        if !removed {
            println!("⚠️ Package {} not found in dependencies", pkg_name);
        }
    }

    save_package_json(&package)?;
    update_lock_file(&package).await?;

    Ok(())
}

pub async fn update_packages(packages: Vec<String>, _config: &NagConfig) -> Result<()> {
    let _package = load_package_json()?;

    if packages.is_empty() {
        println!("Updating all packages...");
        // TODO: Update all packages
    } else {
        for pkg_name in packages {
            println!("Updating {}...", pkg_name);
            // TODO: Update specific package
        }
    }

    println!("Package updates not yet implemented");
    Ok(())
}

pub async fn list_packages(tree: bool, outdated: bool, _config: &NagConfig) -> Result<()> {
    let package = load_package_json()?;

    if outdated {
        println!("Checking for outdated packages...");
        // TODO: Check for outdated packages
        println!("Outdated package checking not yet implemented");
        return Ok(());
    }

    if tree {
        println!("Dependency tree:");
        println!("├── Dependencies:");
        for (name, version) in &package.dependencies {
            println!("│   ├── {}@{}", name, version);
        }

        println!("└── Dev Dependencies:");
        for (name, version) in &package.dev_dependencies {
            println!("    ├── {}@{}", name, version);
        }
    } else {
        println!("Dependencies:");
        for (name, version) in &package.dependencies {
            println!("  {}@{}", name, version);
        }

        if !package.dev_dependencies.is_empty() {
            println!("\nDev Dependencies:");
            for (name, version) in &package.dev_dependencies {
                println!("  {}@{}", name, version);
            }
        }
    }

    Ok(())
}

pub async fn publish_package(
    registry: Option<String>,
    dry_run: bool,
    config: &NagConfig,
) -> Result<()> {
    let package = load_package_json()?;
    let registry_url = registry.unwrap_or_else(|| config.package.registry.clone());

    println!("Publishing {} v{} to {}", package.name, package.version, registry_url);

    if dry_run {
        println!("DRY RUN - would publish:");
        println!("  Package: {}", package.name);
        println!("  Version: {}", package.version);
        println!("  Registry: {}", registry_url);
        return Ok(());
    }

    // TODO: Implement actual publishing
    println!("Package publishing not yet implemented");

    Ok(())
}

pub async fn pack_package(output: Option<PathBuf>, _config: &NagConfig) -> Result<()> {
    let package = load_package_json()?;
    let output_file = output.unwrap_or_else(|| {
        PathBuf::from(format!("{}-{}.tgz", package.name, package.version))
    });

    println!("Packing {} v{} to {}", package.name, package.version, output_file.display());

    // TODO: Implement package creation (tar.gz with package contents)
    println!("Package packing not yet implemented");

    Ok(())
}

// Helper functions
fn load_package_json() -> Result<PackageJson> {
    let package_file = PathBuf::from("nagari.json");

    if !package_file.exists() {
        anyhow::bail!("No nagari.json found. Run 'nag package init' first.");
    }

    let content = std::fs::read_to_string(&package_file)?;
    let package: PackageJson = serde_json::from_str(&content)?;

    Ok(package)
}

fn save_package_json(package: &PackageJson) -> Result<()> {
    let content = serde_json::to_string_pretty(package)?;
    std::fs::write("nagari.json", content)?;
    Ok(())
}

async fn update_lock_file(_package: &PackageJson) -> Result<()> {
    // TODO: Implement lock file generation
    // This would resolve all dependencies and create a lock file
    // with exact versions and integrity hashes

    let lock_file = LockFile {
        version: "1.0.0".to_string(),
        dependencies: HashMap::new(),
    };

    let content = serde_json::to_string_pretty(&lock_file)?;
    std::fs::write("nag.lock", content)?;

    Ok(())
}

fn parse_package_spec(spec: &str) -> (String, String) {
    if let Some(at_pos) = spec.rfind('@') {
        if at_pos > 0 {
            let name = spec[..at_pos].to_string();
            let version = spec[at_pos + 1..].to_string();
            return (name, version);
        }
    }

    (spec.to_string(), "latest".to_string())
}

fn prompt_with_default(prompt: &str, default: &str) -> Result<String> {
    print!("{} ({}): ", prompt, default);
    std::io::Write::flush(&mut std::io::stdout())?;

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    let trimmed = input.trim();
    if trimmed.is_empty() {
        Ok(default.to_string())
    } else {
        Ok(trimmed.to_string())
    }
}

fn prompt_optional(prompt: &str) -> Result<Option<String>> {
    print!("{}: ", prompt);
    std::io::Write::flush(&mut std::io::stdout())?;

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    let trimmed = input.trim();
    if trimmed.is_empty() {
        Ok(None)
    } else {
        Ok(Some(trimmed.to_string()))
    }
}
