use crate::core::error_handler::AppError;
use crate::core::error_handler::Result;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::process::Command;
use tracing::{info, warn, error};

#[derive(Debug, Clone)]
pub struct LoaderInstaller {
    java_path: PathBuf,
}

impl LoaderInstaller {
    pub fn new(java_path: PathBuf) -> Self {
        Self { java_path }
    }

    /// Install Fabric server for the given Minecraft version and Fabric version
    pub async fn install_fabric_server(
        &self,
        minecraft_version: &str,
        fabric_version: &str,
        server_dir: &Path,
    ) -> Result<PathBuf> {
        info!("Installing Fabric server {} for Minecraft {}", fabric_version, minecraft_version);
        
        // Create server directory if it doesn't exist
        fs::create_dir_all(server_dir).await.map_err(|e| AppError::FileSystemError {
            message: format!("Failed to create server directory: {}", e),
            path: server_dir.to_string_lossy().to_string(),
            operation: "create_dir_all".to_string(),
        })?;

        // Download Fabric installer
        let installer_jar = self.download_fabric_installer(fabric_version, server_dir).await?;
        
        // Run Fabric installer
        let server_jar = self.run_fabric_installer(
            &installer_jar,
            minecraft_version,
            fabric_version,
            server_dir,
        ).await?;

        info!("Fabric server installed successfully at: {}", server_jar.display());
        Ok(server_jar)
    }

    /// Install Quilt server for the given Minecraft version and Quilt version
    pub async fn install_quilt_server(
        &self,
        minecraft_version: &str,
        quilt_version: &str,
        server_dir: &Path,
    ) -> Result<PathBuf> {
        info!("Installing Quilt server {} for Minecraft {}", quilt_version, minecraft_version);
        
        // Create server directory if it doesn't exist
        fs::create_dir_all(server_dir).await.map_err(|e| AppError::FileSystemError {
            message: format!("Failed to create server directory: {}", e),
            path: server_dir.to_string_lossy().to_string(),
            operation: "create_dir_all".to_string(),
        })?;

        // Download Quilt installer
        let installer_jar = self.download_quilt_installer(quilt_version, server_dir).await?;
        
        // Run Quilt installer
        let server_jar = self.run_quilt_installer(
            &installer_jar,
            minecraft_version,
            quilt_version,
            server_dir,
        ).await?;

        info!("Quilt server installed successfully at: {}", server_jar.display());
        Ok(server_jar)
    }

    /// Install Forge server for the given Minecraft version and Forge version
    pub async fn install_forge_server(
        &self,
        minecraft_version: &str,
        forge_version: &str,
        server_dir: &Path,
    ) -> Result<PathBuf> {
        info!("Installing Forge server {} for Minecraft {}", forge_version, minecraft_version);
        
        // Create server directory if it doesn't exist
        fs::create_dir_all(server_dir).await.map_err(|e| AppError::FileSystemError {
            message: format!("Failed to create server directory: {}", e),
            path: server_dir.to_string_lossy().to_string(),
            operation: "create_dir_all".to_string(),
        })?;

        // Download Forge installer
        let installer_jar = self.download_forge_installer(minecraft_version, forge_version, server_dir).await?;
        
        // Run Forge installer
        let server_jar = self.run_forge_installer(
            &installer_jar,
            minecraft_version,
            forge_version,
            server_dir,
        ).await?;

        info!("Forge server installed successfully at: {}", server_jar.display());
        Ok(server_jar)
    }

    /// Download Fabric installer JAR
    async fn download_fabric_installer(&self, fabric_version: &str, server_dir: &Path) -> Result<PathBuf> {
        let installer_url = format!(
            "https://maven.fabricmc.net/net/fabricmc/fabric-installer/{}/fabric-installer-{}.jar",
            fabric_version, fabric_version
        );
        
        let installer_path = server_dir.join("fabric-installer.jar");
        
        info!("Downloading Fabric installer from: {}", installer_url);
        
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .map_err(|e| AppError::NetworkError {
                message: format!("Failed to create HTTP client: {}", e),
                endpoint: "create_client".to_string(),
                status_code: None,
            })?;

        let response = client.get(&installer_url).send().await.map_err(|e| AppError::NetworkError {
            message: format!("Failed to download Fabric installer: {}", e),
            endpoint: "download_installer".to_string(),
            status_code: None,
        })?;

        if !response.status().is_success() {
            return Err(AppError::NetworkError {
                message: format!("Failed to download Fabric installer: HTTP {}", response.status()),
                endpoint: "download_installer".to_string(),
            status_code: None,
            });
        }

        let bytes = response.bytes().await.map_err(|e| AppError::NetworkError {
            message: format!("Failed to read installer response: {}", e),
                endpoint: "read_response".to_string(),
                status_code: None,
        })?;

        fs::write(&installer_path, bytes).await.map_err(|e| AppError::FileSystemError {
            message: format!("Failed to write installer JAR: {}", e),
            path: installer_path.to_string_lossy().to_string(),
            operation: "write".to_string(),
        })?;

        info!("Fabric installer downloaded to: {}", installer_path.display());
        Ok(installer_path)
    }

    /// Download Quilt installer JAR
    async fn download_quilt_installer(&self, quilt_version: &str, server_dir: &Path) -> Result<PathBuf> {
        let installer_url = format!(
            "https://maven.quiltmc.org/repository/release/org/quiltmc/quilt-installer/{}/quilt-installer-{}.jar",
            quilt_version, quilt_version
        );
        
        let installer_path = server_dir.join("quilt-installer.jar");
        
        info!("Downloading Quilt installer from: {}", installer_url);
        
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .map_err(|e| AppError::NetworkError {
                message: format!("Failed to create HTTP client: {}", e),
                endpoint: "create_client".to_string(),
                status_code: None,
            })?;

        let response = client.get(&installer_url).send().await.map_err(|e| AppError::NetworkError {
            message: format!("Failed to download Quilt installer: {}", e),
            endpoint: "download_installer".to_string(),
            status_code: None,
        })?;

        if !response.status().is_success() {
            return Err(AppError::NetworkError {
                message: format!("Failed to download Quilt installer: HTTP {}", response.status()),
                endpoint: "download_installer".to_string(),
            status_code: None,
            });
        }

        let bytes = response.bytes().await.map_err(|e| AppError::NetworkError {
            message: format!("Failed to read installer response: {}", e),
                endpoint: "read_response".to_string(),
                status_code: None,
        })?;

        fs::write(&installer_path, bytes).await.map_err(|e| AppError::FileSystemError {
            message: format!("Failed to write installer JAR: {}", e),
            path: installer_path.to_string_lossy().to_string(),
            operation: "write".to_string(),
        })?;

        info!("Quilt installer downloaded to: {}", installer_path.display());
        Ok(installer_path)
    }

    /// Run Fabric installer to generate server JAR
    async fn run_fabric_installer(
        &self,
        installer_path: &Path,
        minecraft_version: &str,
        fabric_version: &str,
        server_dir: &Path,
    ) -> Result<PathBuf> {
        info!("Running Fabric installer for MC {} Fabric {}", minecraft_version, fabric_version);
        
        let output = Command::new(&self.java_path)
            .arg("-jar")
            .arg(installer_path)
            .arg("server")
            .arg("-mcversion")
            .arg(minecraft_version)
            .arg("-loader")
            .arg(fabric_version)
            .arg("-downloadMinecraft")
            .current_dir(server_dir)
            .output()
            .await
            .map_err(|e| AppError::ProcessError {
                message: format!("Failed to run Fabric installer: {}", e),
                process_id: None,
                operation: format!("java -jar {} server -mcversion {} -loader {} -downloadMinecraft", 
                    installer_path.display(), minecraft_version, fabric_version),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("Fabric installer failed: {}", stderr);
            return Err(AppError::ProcessError {
                message: format!("Fabric installer failed: {}", stderr),
                process_id: None,
                operation: format!("java -jar {} server -mcversion {} -loader {} -downloadMinecraft", 
                    installer_path.display(), minecraft_version, fabric_version),
            });
        }

        // Look for the generated server JAR
        let server_jar = server_dir.join("server.jar");
        if !server_jar.exists() {
            return Err(AppError::FileSystemError {
                message: "Fabric installer did not generate server.jar".to_string(),
                path: server_jar.to_string_lossy().to_string(),
                operation: "verify_output".to_string(),
            });
        }

        info!("Fabric installer completed successfully");
        Ok(server_jar)
    }

    /// Run Quilt installer to generate server JAR
    async fn run_quilt_installer(
        &self,
        installer_path: &Path,
        minecraft_version: &str,
        quilt_version: &str,
        server_dir: &Path,
    ) -> Result<PathBuf> {
        info!("Running Quilt installer for MC {} Quilt {}", minecraft_version, quilt_version);
        
        let output = Command::new(&self.java_path)
            .arg("-jar")
            .arg(installer_path)
            .arg("server")
            .arg("--mc-version")
            .arg(minecraft_version)
            .arg("--installer-version")
            .arg(quilt_version)
            .arg("--download-server")
            .current_dir(server_dir)
            .output()
            .await
            .map_err(|e| AppError::ProcessError {
                message: format!("Failed to run Quilt installer: {}", e),
                process_id: None,
                operation: format!("java -jar {} server --mc-version {} --installer-version {} --download-server", 
                    installer_path.display(), minecraft_version, quilt_version),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("Quilt installer failed: {}", stderr);
            return Err(AppError::ProcessError {
                message: format!("Quilt installer failed: {}", stderr),
                process_id: None,
                operation: format!("java -jar {} server --mc-version {} --installer-version {} --download-server", 
                    installer_path.display(), minecraft_version, quilt_version),
            });
        }

        // Look for the generated server JAR
        let server_jar = server_dir.join("server.jar");
        if !server_jar.exists() {
            return Err(AppError::FileSystemError {
                message: "Quilt installer did not generate server.jar".to_string(),
                path: server_jar.to_string_lossy().to_string(),
                operation: "verify_output".to_string(),
            });
        }

        info!("Quilt installer completed successfully");
        Ok(server_jar)
    }

    /// Download Forge installer JAR
    async fn download_forge_installer(&self, minecraft_version: &str, forge_version: &str, server_dir: &Path) -> Result<PathBuf> {
        let installer_url = format!(
            "https://maven.minecraftforge.net/net/minecraftforge/forge/{}-{}/forge-{}-{}-installer.jar",
            minecraft_version, forge_version, minecraft_version, forge_version
        );
        
        let installer_path = server_dir.join("forge-installer.jar");
        
        info!("Downloading Forge installer from: {}", installer_url);
        
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .map_err(|e| AppError::NetworkError {
                message: format!("Failed to create HTTP client: {}", e),
                endpoint: "create_client".to_string(),
                status_code: None,
            })?;

        let response = client.get(&installer_url).send().await.map_err(|e| AppError::NetworkError {
            message: format!("Failed to download Forge installer: {}", e),
            endpoint: "download_installer".to_string(),
            status_code: None,
        })?;

        if !response.status().is_success() {
            return Err(AppError::NetworkError {
                message: format!("Failed to download Forge installer: HTTP {}", response.status()),
                endpoint: "download_installer".to_string(),
            status_code: None,
            });
        }

        let bytes = response.bytes().await.map_err(|e| AppError::NetworkError {
            message: format!("Failed to read installer response: {}", e),
                endpoint: "read_response".to_string(),
                status_code: None,
        })?;

        fs::write(&installer_path, bytes).await.map_err(|e| AppError::FileSystemError {
            message: format!("Failed to write installer JAR: {}", e),
            path: installer_path.to_string_lossy().to_string(),
            operation: "write".to_string(),
        })?;

        info!("Forge installer downloaded to: {}", installer_path.display());
        Ok(installer_path)
    }

    /// Run Forge installer to generate server JAR
    async fn run_forge_installer(
        &self,
        installer_path: &Path,
        minecraft_version: &str,
        forge_version: &str,
        server_dir: &Path,
    ) -> Result<PathBuf> {
        info!("Running Forge installer for MC {} Forge {}", minecraft_version, forge_version);
        
        let output = Command::new(&self.java_path)
            .arg("-jar")
            .arg(installer_path)
            .arg("--installServer")
            .arg("--minecraft")
            .arg(minecraft_version)
            .arg("--version")
            .arg(forge_version)
            .current_dir(server_dir)
            .output()
            .await
            .map_err(|e| AppError::ProcessError {
                message: format!("Failed to run Forge installer: {}", e),
                process_id: None,
                operation: format!("java -jar {} --installServer --minecraft {} --version {}", 
                    installer_path.display(), minecraft_version, forge_version),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("Forge installer failed: {}", stderr);
            return Err(AppError::ProcessError {
                message: format!("Forge installer failed: {}", stderr),
                process_id: None,
                operation: format!("java -jar {} --installServer --minecraft {} --version {}", 
                    installer_path.display(), minecraft_version, forge_version),
            });
        }

        // Look for the generated server JAR
        let server_jar = server_dir.join("server.jar");
        if !server_jar.exists() {
            return Err(AppError::FileSystemError {
                message: "Forge installer did not generate server.jar".to_string(),
                path: server_jar.to_string_lossy().to_string(),
                operation: "verify_output".to_string(),
            });
        }

        info!("Forge installer completed successfully");
        Ok(server_jar)
    }

    /// Detect Java installation on the system
    pub async fn detect_java() -> Result<PathBuf> {
        // Try common Java paths on Windows
        let common_paths = [
            r"C:\Program Files\Java\jdk-*\bin\java.exe",
            r"C:\Program Files\Java\jre-*\bin\java.exe",
            r"C:\Program Files\Eclipse Adoptium\jdk-*\bin\java.exe",
            r"C:\Program Files\Eclipse Adoptium\jre-*\bin\java.exe",
            r"C:\Program Files\Microsoft\jdk-*\bin\java.exe",
            r"C:\Program Files\Microsoft\jre-*\bin\java.exe",
        ];

        // First try to find java in PATH
        if let Ok(output) = Command::new("java").arg("-version").output().await {
            if output.status.success() {
                if let Ok(which_output) = Command::new("where").arg("java").output().await {
                    if let Ok(path) = String::from_utf8(which_output.stdout) {
                        let java_path = path.trim().lines().next().unwrap_or("");
                        if !java_path.is_empty() {
                            return Ok(PathBuf::from(java_path));
                        }
                    }
                }
            }
        }

        // Try common installation paths
        for pattern in &common_paths {
            if let Ok(entries) = glob::glob(pattern) {
                for entry in entries.flatten() {
                    if entry.exists() {
                        return Ok(entry);
                    }
                }
            }
        }

        Err(AppError::ValidationError {
            message: "Java installation not found. Please install Java 17 or higher.".to_string(),
            field: "java_path".to_string(),
            value: "".to_string(),
            constraint: "Java 17+ required".to_string(),
        })
    }
}
