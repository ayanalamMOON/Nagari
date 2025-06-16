use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::Result;
use reqwest::Client;
use url::Url;

#[derive(Debug, Clone)]
pub struct RegistryClient {
    client: Client,
    registry_url: Url,
    auth_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub description: Option<String>,
    pub versions: HashMap<String, VersionInfo>,
    pub dist_tags: HashMap<String, String>,
    pub time: HashMap<String, String>,
    pub keywords: Vec<String>,
    pub author: Option<AuthorInfo>,
    pub license: Option<String>,
    pub repository: Option<RepositoryInfo>,
    pub homepage: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    pub version: String,
    pub description: Option<String>,
    pub main: Option<String>,
    pub exports: Option<HashMap<String, String>>,
    pub dependencies: HashMap<String, String>,
    pub dev_dependencies: HashMap<String, String>,
    pub peer_dependencies: HashMap<String, String>,
    pub optional_dependencies: HashMap<String, String>,
    pub dist: DistInfo,
    pub engines: Option<HashMap<String, String>>,
    pub os: Option<Vec<String>>,
    pub cpu: Option<Vec<String>>,
    pub deprecated: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistInfo {
    pub tarball: String,
    pub shasum: String,
    pub integrity: Option<String>,
    pub file_count: Option<u32>,
    pub unpacked_size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorInfo {
    pub name: String,
    pub email: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryInfo {
    pub r#type: String,
    pub url: String,
    pub directory: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub objects: Vec<SearchObject>,
    pub total: u32,
    pub time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchObject {
    pub package: SearchPackage,
    pub score: SearchScore,
    pub searchScore: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchPackage {
    pub name: String,
    pub scope: Option<String>,
    pub version: String,
    pub description: Option<String>,
    pub keywords: Vec<String>,
    pub date: String,
    pub links: SearchLinks,
    pub author: Option<AuthorInfo>,
    pub publisher: Option<AuthorInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchScore {
    pub final_score: f64,
    pub detail: SearchScoreDetail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchScoreDetail {
    pub quality: f64,
    pub popularity: f64,
    pub maintenance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchLinks {
    pub npm: Option<String>,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub bugs: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishRequest {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub tarball_data: Vec<u8>,
    pub metadata: VersionInfo,
}

impl RegistryClient {
    pub fn new(registry_url: &str) -> Result<Self> {
        Ok(Self {
            client: Client::new(),
            registry_url: Url::parse(registry_url)?,
            auth_token: None,
        })
    }

    pub fn with_auth(registry_url: &str, token: String) -> Result<Self> {
        Ok(Self {
            client: Client::new(),
            registry_url: Url::parse(registry_url)?,
            auth_token: Some(token),
        })
    }

    pub async fn get_package_info(&self, name: &str) -> Result<Option<PackageInfo>> {
        let url = self.registry_url.join(&format!("packages/{}", name))?;

        let mut request = self.client.get(url);

        if let Some(ref token) = self.auth_token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await?;

        match response.status() {
            reqwest::StatusCode::OK => {
                let package_info: PackageInfo = response.json().await?;
                Ok(Some(package_info))
            }
            reqwest::StatusCode::NOT_FOUND => Ok(None),
            _ => {
                anyhow::bail!("Registry request failed: {}", response.status());
            }
        }
    }

    pub async fn get_version_info(&self, name: &str, version: &str) -> Result<Option<VersionInfo>> {
        let url = self.registry_url.join(&format!("packages/{}/{}", name, version))?;

        let mut request = self.client.get(url);

        if let Some(ref token) = self.auth_token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await?;

        match response.status() {
            reqwest::StatusCode::OK => {
                let version_info: VersionInfo = response.json().await?;
                Ok(Some(version_info))
            }
            reqwest::StatusCode::NOT_FOUND => Ok(None),
            _ => {
                anyhow::bail!("Registry request failed: {}", response.status());
            }
        }
    }

    pub async fn search_packages(&self, query: &str, size: Option<u32>) -> Result<SearchResult> {
        let mut url = self.registry_url.join("search")?;

        {
            let mut query_params = url.query_pairs_mut();
            query_params.append_pair("text", query);
            if let Some(size) = size {
                query_params.append_pair("size", &size.to_string());
            }
        }

        let response = self.client.get(url).send().await?;

        if response.status().is_success() {
            let search_result: SearchResult = response.json().await?;
            Ok(search_result)
        } else {
            anyhow::bail!("Search request failed: {}", response.status());
        }
    }

    pub async fn download_package(&self, name: &str, version: &str) -> Result<Vec<u8>> {
        let package_info = self.get_version_info(name, version).await?
            .ok_or_else(|| anyhow::anyhow!("Package {} version {} not found", name, version))?;

        let response = self.client.get(&package_info.dist.tarball).send().await?;

        if response.status().is_success() {
            let bytes = response.bytes().await?;
            Ok(bytes.to_vec())
        } else {
            anyhow::bail!("Download failed: {}", response.status());
        }
    }

    pub async fn publish_package(&self, request: PublishRequest) -> Result<()> {
        if self.auth_token.is_none() {
            anyhow::bail!("Authentication required for publishing");
        }

        let url = self.registry_url.join(&format!("packages/{}", request.name))?;

        let response = self.client
            .put(url)
            .bearer_auth(self.auth_token.as_ref().unwrap())
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Publish failed: {}", error_text);
        }
    }

    pub async fn unpublish_package(&self, name: &str, version: Option<&str>) -> Result<()> {
        if self.auth_token.is_none() {
            anyhow::bail!("Authentication required for unpublishing");
        }

        let url = if let Some(version) = version {
            self.registry_url.join(&format!("packages/{}/{}", name, version))?
        } else {
            self.registry_url.join(&format!("packages/{}", name))?
        };

        let response = self.client
            .delete(url)
            .bearer_auth(self.auth_token.as_ref().unwrap())
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Unpublish failed: {}", error_text);
        }
    }

    pub async fn deprecate_package(&self, name: &str, version: &str, message: &str) -> Result<()> {
        if self.auth_token.is_none() {
            anyhow::bail!("Authentication required for deprecation");
        }

        let url = self.registry_url.join(&format!("packages/{}/{}/deprecate", name, version))?;

        let mut body = HashMap::new();
        body.insert("message", message);

        let response = self.client
            .post(url)
            .bearer_auth(self.auth_token.as_ref().unwrap())
            .json(&body)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Deprecation failed: {}", error_text);
        }
    }

    pub fn set_auth_token(&mut self, token: String) {
        self.auth_token = Some(token);
    }

    pub fn clear_auth_token(&mut self) {
        self.auth_token = None;
    }
}
