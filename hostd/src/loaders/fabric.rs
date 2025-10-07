use crate::core::error_handler::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FabricVersion {
    pub version: String,
    pub stable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FabricLoaderVersion {
    pub separator: String,
    pub build: u32,
    pub maven: String,
    pub version: String,
    pub stable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FabricGameVersion {
    pub version: String,
    pub stable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FabricVersionManifest {
    pub game: Vec<FabricGameVersion>,
    pub loader: Vec<FabricLoaderVersion>,
}

/// Client for interacting with Fabric's version API
pub struct FabricClient {
    client: reqwest::Client,
}

impl FabricClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    /// Get available Fabric loader versions
    pub async fn get_loader_versions(&self) -> Result<Vec<FabricLoaderVersion>> {
        let url = "https://meta.fabricmc.net/v2/versions/loader";
        
        let response = self.client.get(url).send().await.map_err(|e| AppError::NetworkError {
            message: format!("Failed to fetch Fabric loader versions: {}", e),
            endpoint: "get_loader_versions".to_string(),
            status_code: None,
        })?;

        if !response.status().is_success() {
            return Err(AppError::NetworkError {
                message: format!("Failed to fetch Fabric loader versions: HTTP {}", response.status()),
                endpoint: "get_loader_versions".to_string(),
            status_code: None,
            });
        }

        let versions: Vec<FabricLoaderVersion> = response.json().await.map_err(|e| AppError::NetworkError {
            message: format!("Failed to parse Fabric loader versions: {}", e),
            endpoint: "parse_loader_versions".to_string(),
            status_code: None,
        })?;

        Ok(versions)
    }

    /// Get available Minecraft versions supported by Fabric
    pub async fn get_game_versions(&self) -> Result<Vec<FabricGameVersion>> {
        let url = "https://meta.fabricmc.net/v2/versions/game";
        
        let response = self.client.get(url).send().await.map_err(|e| AppError::NetworkError {
            message: format!("Failed to fetch Fabric game versions: {}", e),
            endpoint: "get_game_versions".to_string(),
            status_code: None,
        })?;

        if !response.status().is_success() {
            return Err(AppError::NetworkError {
                message: format!("Failed to fetch Fabric game versions: HTTP {}", response.status()),
                endpoint: "get_game_versions".to_string(),
            status_code: None,
            });
        }

        let versions: Vec<FabricGameVersion> = response.json().await.map_err(|e| AppError::NetworkError {
            message: format!("Failed to parse Fabric game versions: {}", e),
            endpoint: "parse_game_versions".to_string(),
            status_code: None,
        })?;

        Ok(versions)
    }

    /// Get the latest stable Fabric loader version
    pub async fn get_latest_loader_version(&self) -> Result<String> {
        let versions = self.get_loader_versions().await?;
        
        // Find the latest stable version
        let latest = versions
            .into_iter()
            .find(|v| v.stable)
            .ok_or_else(|| AppError::ValidationError {
                message: "No stable Fabric loader versions found".to_string(),
                field: "loader_version".to_string(),
                value: "".to_string(),
                constraint: "stable version required".to_string(),
            })?;

        Ok(latest.version)
    }

    /// Get the latest stable Minecraft version supported by Fabric
    pub async fn get_latest_game_version(&self) -> Result<String> {
        let versions = self.get_game_versions().await?;
        
        // Find the latest stable version
        let latest = versions
            .into_iter()
            .find(|v| v.stable)
            .ok_or_else(|| AppError::ValidationError {
                message: "No stable Minecraft versions found".to_string(),
                field: "minecraft_version".to_string(),
                value: "".to_string(),
                constraint: "stable version required".to_string(),
            })?;

        Ok(latest.version)
    }

    /// Check if a specific Minecraft version is supported by Fabric
    pub async fn is_version_supported(&self, minecraft_version: &str) -> Result<bool> {
        let versions = self.get_game_versions().await?;
        Ok(versions.iter().any(|v| v.version == minecraft_version))
    }

    /// Get the latest stable Fabric loader version for a specific Minecraft version
    pub async fn get_latest_loader_for_version(&self, minecraft_version: &str) -> Result<String> {
        // First check if the Minecraft version is supported
        if !self.is_version_supported(minecraft_version).await? {
            return Err(AppError::ValidationError {
                message: format!("Minecraft version {} is not supported by Fabric", minecraft_version),
                field: "minecraft_version".to_string(),
                value: minecraft_version.to_string(),
                constraint: "supported version required".to_string(),
            });
        }

        // Get the latest stable loader version
        self.get_latest_loader_version().await
    }
}
