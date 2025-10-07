use std::path::{Path, PathBuf};
use std::error::Error;
use std::fmt;

/// Error types for path sanitization
#[derive(Debug, Clone)]
pub enum PathSanitizationError {
    AbsolutePath,
    ParentDirectoryTraversal,
    DisallowedPrefix,
    InvalidPath,
    EmptyPath,
}

impl fmt::Display for PathSanitizationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PathSanitizationError::AbsolutePath => write!(f, "Absolute paths are not allowed"),
            PathSanitizationError::ParentDirectoryTraversal => write!(f, "Parent directory traversal (..) is not allowed"),
            PathSanitizationError::DisallowedPrefix => write!(f, "Path contains disallowed prefix"),
            PathSanitizationError::InvalidPath => write!(f, "Invalid path characters"),
            PathSanitizationError::EmptyPath => write!(f, "Empty path not allowed"),
        }
    }
}

impl Error for PathSanitizationError {}

/// Path sanitization service for secure file extraction
pub struct PathSanitizer {
    /// Base directory for all extractions
    base_dir: PathBuf,
    /// Allowed prefixes for file paths
    allowed_prefixes: Vec<String>,
}

impl PathSanitizer {
    /// Create a new path sanitizer
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            base_dir,
            allowed_prefixes: vec![
                "mods/".to_string(),
                "config/".to_string(),
                "world/".to_string(),
                "logs/".to_string(),
                "scripts/".to_string(),
            ],
        }
    }

    /// Sanitize a file path for safe extraction
    pub fn sanitize_path(&self, file_path: &str) -> Result<PathBuf, PathSanitizationError> {
        // Check for empty path
        if file_path.is_empty() {
            return Err(PathSanitizationError::EmptyPath);
        }

        // Normalize path separators for Windows compatibility
        let normalized_path = file_path.replace('\\', "/");
        
        // Check for absolute paths
        if normalized_path.starts_with('/') || normalized_path.contains(':') {
            return Err(PathSanitizationError::AbsolutePath);
        }

        // Check for parent directory traversal
        if normalized_path.contains("../") || normalized_path.contains("..\\") || 
           normalized_path.starts_with("..") {
            return Err(PathSanitizationError::ParentDirectoryTraversal);
        }

        // Check for disallowed prefixes
        let has_allowed_prefix = self.allowed_prefixes.iter()
            .any(|prefix| normalized_path.starts_with(prefix));
        
        if !has_allowed_prefix {
            return Err(PathSanitizationError::DisallowedPrefix);
        }

        // Create the full path
        let sanitized_path = self.base_dir.join(&normalized_path);
        
        // Ensure the path is within the base directory
        if !sanitized_path.starts_with(&self.base_dir) {
            return Err(PathSanitizationError::ParentDirectoryTraversal);
        }

        // Canonicalize the path to resolve any remaining issues
        match sanitized_path.canonicalize() {
            Ok(canonical_path) => {
                if canonical_path.starts_with(&self.base_dir) {
                    Ok(canonical_path)
                } else {
                    Err(PathSanitizationError::ParentDirectoryTraversal)
                }
            }
            Err(_) => {
                // If canonicalization fails, use the sanitized path as-is
                // This can happen if the file doesn't exist yet
                Ok(sanitized_path)
            }
        }
    }

    /// Check if a path is safe for extraction
    pub fn is_safe_path(&self, file_path: &str) -> bool {
        self.sanitize_path(file_path).is_ok()
    }

    /// Get the base directory
    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }

    /// Add an allowed prefix
    pub fn add_allowed_prefix(&mut self, prefix: String) {
        self.allowed_prefixes.push(prefix);
    }

    /// Remove an allowed prefix
    pub fn remove_allowed_prefix(&mut self, prefix: &str) {
        self.allowed_prefixes.retain(|p| p != prefix);
    }
}

/// Secure file extraction utilities
pub struct SecureExtractor {
    sanitizer: PathSanitizer,
}

impl SecureExtractor {
    /// Create a new secure extractor
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            sanitizer: PathSanitizer::new(base_dir),
        }
    }

    /// Extract a file safely to the target directory
    pub async fn extract_file(
        &self,
        source_path: &str,
        content: &[u8],
    ) -> Result<PathBuf, Box<dyn Error>> {
        // Sanitize the target path
        let target_path = self.sanitizer.sanitize_path(source_path)?;
        
        // Create parent directories if they don't exist
        if let Some(parent) = target_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // Write the file
        tokio::fs::write(&target_path, content).await?;

        Ok(target_path)
    }

    /// Extract multiple files safely
    pub async fn extract_files(
        &self,
        files: Vec<(String, Vec<u8>)>,
    ) -> Result<Vec<(String, PathBuf)>, Box<dyn Error>> {
        let mut extracted_files = Vec::new();
        let mut skipped_files = Vec::new();

        for (source_path, content) in files {
            match self.extract_file(&source_path, &content).await {
                Ok(target_path) => {
                    extracted_files.push((source_path, target_path));
                }
                Err(e) => {
                    skipped_files.push((source_path, e.to_string()));
                }
            }
        }

        // Log skipped files for debugging
        if !skipped_files.is_empty() {
            tracing::warn!("Skipped {} files due to path sanitization", skipped_files.len());
            for (path, reason) in skipped_files {
                tracing::debug!("Skipped {}: {}", path, reason);
            }
        }

        Ok(extracted_files)
    }

    /// Get the sanitizer
    pub fn sanitizer(&self) -> &PathSanitizer {
        &self.sanitizer
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_path_sanitization() {
        let temp_dir = TempDir::new().unwrap();
        let sanitizer = PathSanitizer::new(temp_dir.path().to_path_buf());

        // Valid paths
        assert!(sanitizer.is_safe_path("mods/test.jar"));
        assert!(sanitizer.is_safe_path("config/test.json"));
        assert!(sanitizer.is_safe_path("world/region/r.0.0.mca"));

        // Invalid paths
        assert!(!sanitizer.is_safe_path("/absolute/path"));
        assert!(!sanitizer.is_safe_path("../parent"));
        assert!(!sanitizer.is_safe_path("..\\parent"));
        assert!(!sanitizer.is_safe_path("random/file.txt"));
        assert!(!sanitizer.is_safe_path(""));
    }

    #[test]
    fn test_sanitize_path() {
        let temp_dir = TempDir::new().unwrap();
        let sanitizer = PathSanitizer::new(temp_dir.path().to_path_buf());

        // Valid paths
        assert!(sanitizer.sanitize_path("mods/test.jar").is_ok());
        assert!(sanitizer.sanitize_path("config/test.json").is_ok());

        // Invalid paths
        assert!(sanitizer.sanitize_path("/absolute/path").is_err());
        assert!(sanitizer.sanitize_path("../parent").is_err());
        assert!(sanitizer.sanitize_path("random/file.txt").is_err());
    }

    #[tokio::test]
    async fn test_secure_extraction() {
        let temp_dir = TempDir::new().unwrap();
        let extractor = SecureExtractor::new(temp_dir.path().to_path_buf());

        let content = b"test content";
        let result = extractor.extract_file("mods/test.jar", content).await;
        assert!(result.is_ok());

        let target_path = result.unwrap();
        assert!(target_path.exists());
        assert_eq!(tokio::fs::read(&target_path).await.unwrap(), content);
    }
}
