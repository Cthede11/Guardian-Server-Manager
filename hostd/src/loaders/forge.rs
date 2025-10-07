use crate::core::error_handler::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgeVersion {
    pub version: String,
    pub stable: bool,
    pub recommended: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgeGameVersion {
    pub version: String,
    pub stable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgeVersionManifest {
    pub game: Vec<ForgeGameVersion>,
    pub forge: Vec<ForgeVersion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgeInstallerInfo {
    pub version: String,
    pub installer_url: String,
    pub universal_url: String,
    pub changelog_url: String,
    pub minecraft_version: String,
    pub recommended: bool,
    pub latest: bool,
}

/// Client for interacting with Forge's version API
pub struct ForgeClient {
    client: reqwest::Client,
}

impl ForgeClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    /// Get available Forge versions for a specific Minecraft version
    pub async fn get_versions_for_minecraft(&self, minecraft_version: &str) -> Result<Vec<ForgeInstallerInfo>> {
        let url = format!("https://files.minecraftforge.net/net/minecraftforge/forge/promotions_slim.json");
        
        let response = self.client.get(&url).send().await.map_err(|e| AppError::NetworkError {
            message: format!("Failed to fetch Forge versions: {}", e),
            endpoint: "get_versions_for_minecraft".to_string(),
            status_code: None,
        })?;

        if !response.status().is_success() {
            return Err(AppError::NetworkError {
                message: format!("Failed to fetch Forge versions: HTTP {}", response.status()),
                endpoint: "get_versions_for_minecraft".to_string(),
            status_code: None,
            });
        }

        let manifest: serde_json::Value = response.json().await.map_err(|e| AppError::NetworkError {
            message: format!("Failed to parse Forge versions: {}", e),
            endpoint: "parse_versions".to_string(),
            status_code: None,
        })?;

        // Parse the Forge promotions JSON
        let mut versions = Vec::new();
        
        if let Some(promos) = manifest["promos"].as_object() {
            for (key, value) in promos {
                if let Some(version_str) = value.as_str() {
                    if key.starts_with(&format!("{}-", minecraft_version)) {
                        let forge_version = key.strip_prefix(&format!("{}-", minecraft_version))
                            .unwrap_or(key);
                        
                        let installer_url = format!(
                            "https://maven.minecraftforge.net/net/minecraftforge/forge/{}-{}/forge-{}-{}-installer.jar",
                            minecraft_version, forge_version, minecraft_version, forge_version
                        );
                        
                        let universal_url = format!(
                            "https://maven.minecraftforge.net/net/minecraftforge/forge/{}-{}/forge-{}-{}-universal.jar",
                            minecraft_version, forge_version, minecraft_version, forge_version
                        );
                        
                        let changelog_url = format!(
                            "https://files.minecraftforge.net/net/minecraftforge/forge/{}-{}/",
                            minecraft_version, forge_version
                        );
                        
                        versions.push(ForgeInstallerInfo {
                            version: forge_version.to_string(),
                            installer_url,
                            universal_url,
                            changelog_url,
                            minecraft_version: minecraft_version.to_string(),
                            recommended: key.ends_with("-recommended"),
                            latest: key.ends_with("-latest"),
                        });
                    }
                }
            }
        }

        // Sort by version (latest first)
        versions.sort_by(|a, b| b.version.cmp(&a.version));
        
        Ok(versions)
    }

    /// Get the latest recommended Forge version for a specific Minecraft version
    pub async fn get_latest_recommended_version(&self, minecraft_version: &str) -> Result<ForgeInstallerInfo> {
        let versions = self.get_versions_for_minecraft(minecraft_version).await?;
        
        versions.into_iter()
            .find(|v| v.recommended)
            .ok_or_else(|| AppError::ValidationError {
                message: format!("No recommended Forge version found for Minecraft {}", minecraft_version),
                field: "forge_version".to_string(),
                value: minecraft_version.to_string(),
                constraint: "recommended version required".to_string(),
            })
    }

    /// Get the latest Forge version for a specific Minecraft version
    pub async fn get_latest_version(&self, minecraft_version: &str) -> Result<ForgeInstallerInfo> {
        let versions = self.get_versions_for_minecraft(minecraft_version).await?;
        
        versions.into_iter()
            .find(|v| v.latest)
            .ok_or_else(|| AppError::ValidationError {
                message: format!("No latest Forge version found for Minecraft {}", minecraft_version),
                field: "forge_version".to_string(),
                value: minecraft_version.to_string(),
                constraint: "latest version required".to_string(),
            })
    }

    /// Check if a specific Minecraft version is supported by Forge
    pub async fn is_version_supported(&self, minecraft_version: &str) -> Result<bool> {
        let versions = self.get_versions_for_minecraft(minecraft_version).await?;
        Ok(!versions.is_empty())
    }

    /// Get a specific Forge version for a Minecraft version
    pub async fn get_specific_version(&self, minecraft_version: &str, forge_version: &str) -> Result<ForgeInstallerInfo> {
        let versions = self.get_versions_for_minecraft(minecraft_version).await?;
        
        versions.into_iter()
            .find(|v| v.version == forge_version)
            .ok_or_else(|| AppError::ValidationError {
                message: format!("Forge version {} not found for Minecraft {}", forge_version, minecraft_version),
                field: "forge_version".to_string(),
                value: forge_version.to_string(),
                constraint: "valid version required".to_string(),
            })
    }
}
