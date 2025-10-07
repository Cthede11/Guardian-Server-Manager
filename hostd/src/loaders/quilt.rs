use crate::core::error_handler::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuiltVersion {
    pub version: String,
    pub stable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuiltLoaderVersion {
    pub version: String,
    pub stable: bool,
    pub maven: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuiltGameVersion {
    pub version: String,
    pub stable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuiltVersionManifest {
    pub game: Vec<QuiltGameVersion>,
    pub loader: Vec<QuiltLoaderVersion>,
}

/// Client for interacting with Quilt's version API
pub struct QuiltClient {
    client: reqwest::Client,
}

impl QuiltClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    /// Get available Quilt loader versions
    pub async fn get_loader_versions(&self) -> Result<Vec<QuiltLoaderVersion>> {
        let url = "https://meta.quiltmc.org/v3/versions/loader";
        
        let response = self.client.get(url).send().await.map_err(|e| AppError::NetworkError {
            message: format!("Failed to fetch Quilt loader versions: {}", e),
            endpoint: "get_loader_versions".to_string(),
            status_code: None,
        })?;

        if !response.status().is_success() {
            return Err(AppError::NetworkError {
                message: format!("Failed to fetch Quilt loader versions: HTTP {}", response.status()),
                endpoint: "get_loader_versions".to_string(),
            status_code: None,
            });
        }

        let versions: Vec<QuiltLoaderVersion> = response.json().await.map_err(|e| AppError::NetworkError {
            message: format!("Failed to parse Quilt loader versions: {}", e),
            endpoint: "parse_loader_versions".to_string(),
            status_code: None,
        })?;

        Ok(versions)
    }

    /// Get available Minecraft versions supported by Quilt
    pub async fn get_game_versions(&self) -> Result<Vec<QuiltGameVersion>> {
        let url = "https://meta.quiltmc.org/v3/versions/game";
        
        let response = self.client.get(url).send().await.map_err(|e| AppError::NetworkError {
            message: format!("Failed to fetch Quilt game versions: {}", e),
            endpoint: "get_game_versions".to_string(),
            status_code: None,
        })?;

        if !response.status().is_success() {
            return Err(AppError::NetworkError {
                message: format!("Failed to fetch Quilt game versions: HTTP {}", response.status()),
                endpoint: "get_game_versions".to_string(),
            status_code: None,
            });
        }

        let versions: Vec<QuiltGameVersion> = response.json().await.map_err(|e| AppError::NetworkError {
            message: format!("Failed to parse Quilt game versions: {}", e),
            endpoint: "parse_game_versions".to_string(),
            status_code: None,
        })?;

        Ok(versions)
    }

    /// Get the latest stable Quilt loader version
    pub async fn get_latest_loader_version(&self) -> Result<String> {
        let versions = self.get_loader_versions().await?;
        
        // Find the latest stable version
        let latest = versions
            .into_iter()
            .find(|v| v.stable)
            .ok_or_else(|| AppError::ValidationError {
                message: "No stable Quilt loader versions found".to_string(),
                field: "loader_version".to_string(),
                value: "".to_string(),
                constraint: "stable version required".to_string(),
            })?;

        Ok(latest.version)
    }

    /// Get the latest stable Minecraft version supported by Quilt
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

    /// Check if a specific Minecraft version is supported by Quilt
    pub async fn is_version_supported(&self, minecraft_version: &str) -> Result<bool> {
        let versions = self.get_game_versions().await?;
        Ok(versions.iter().any(|v| v.version == minecraft_version))
    }

    /// Get the latest stable Quilt loader version for a specific Minecraft version
    pub async fn get_latest_loader_for_version(&self, minecraft_version: &str) -> Result<String> {
        // First check if the Minecraft version is supported
        if !self.is_version_supported(minecraft_version).await? {
            return Err(AppError::ValidationError {
                message: format!("Minecraft version {} is not supported by Quilt", minecraft_version),
                field: "minecraft_version".to_string(),
                value: minecraft_version.to_string(),
                constraint: "supported version required".to_string(),
            });
        }

        // Get the latest stable loader version
        self.get_latest_loader_version().await
    }
}
