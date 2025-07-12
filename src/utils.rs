use crate::error::Result;
use std::path::Path;
use walkdir::WalkDir;

/// File system utilities
pub struct FileUtils;

impl FileUtils {
    /// Create directory if it doesn't exist
    pub fn ensure_dir<P: AsRef<Path>>(path: P) -> Result<()> {
        let path = path.as_ref();
        if !path.exists() {
            std::fs::create_dir_all(path)
                .map_err(|e| crate::error::BarefootError::Io(e))?;
        }
        Ok(())
    }

    /// Clean directory contents
    pub fn clean_dir<P: AsRef<Path>>(path: P) -> Result<()> {
        let path = path.as_ref();
        if path.exists() {
            for entry in WalkDir::new(path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path() != path)
            {
                if entry.file_type().is_dir() {
                    std::fs::remove_dir_all(entry.path())
                        .map_err(|e| crate::error::BarefootError::Io(e))?;
                } else {
                    std::fs::remove_file(entry.path())
                        .map_err(|e| crate::error::BarefootError::Io(e))?;
                }
            }
        }
        Ok(())
    }

    /// Get file size in bytes
    pub fn file_size<P: AsRef<Path>>(path: P) -> Result<u64> {
        let metadata = std::fs::metadata(path)
            .map_err(|e| crate::error::BarefootError::Io(e))?;
        Ok(metadata.len())
    }

    /// Check if path is a directory
    pub fn is_dir<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().is_dir()
    }

    /// Check if path is a file
    pub fn is_file<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().is_file()
    }
}

/// Process utilities
pub struct ProcessUtils;

impl ProcessUtils {
    /// Check if a command exists in PATH
    pub fn command_exists(command: &str) -> bool {
        which::which(command).is_ok()
    }

    /// Get the current working directory
    pub fn current_dir() -> Result<String> {
        std::env::current_dir()
            .map_err(|e| crate::error::BarefootError::Io(e))
            .and_then(|path| {
                path.to_str()
                    .map(|s| s.to_string())
                    .ok_or_else(|| {
                        crate::error::BarefootError::Io(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "Invalid path encoding",
                        ))
                    })
            })
    }

    /// Get environment variable with default
    pub fn env_var(key: &str, default: &str) -> String {
        std::env::var(key).unwrap_or_else(|_| default.to_string())
    }

    /// Set environment variable
    pub fn set_env_var(key: &str, value: &str) -> Result<()> {
        std::env::set_var(key, value);
        Ok(())
    }
}

/// Network utilities
pub struct NetworkUtils;

impl NetworkUtils {
    /// Check if a URL is reachable
    pub async fn check_url(url: &str) -> Result<bool> {
        let client = reqwest::Client::new();
        match client.get(url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    /// Get hostname from URL
    pub fn get_hostname(url: &str) -> Option<String> {
        url::Url::parse(url)
            .ok()
            .and_then(|u| u.host_str().map(|h| h.to_string()))
    }

    /// Validate URL format
    pub fn is_valid_url(url: &str) -> bool {
        url::Url::parse(url).is_ok()
    }
}

/// Crypto utilities
pub struct CryptoUtils;

impl CryptoUtils {
    /// Generate HMAC-SHA256 signature
    pub fn hmac_sha256(key: &[u8], data: &[u8]) -> String {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        
        let mut mac = Hmac::<Sha256>::new_from_slice(key)
            .expect("HMAC can take key of any size");
        mac.update(data);
        hex::encode(mac.finalize().into_bytes())
    }

    /// Generate SHA256 hash
    pub fn sha256(data: &[u8]) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(data);
        hex::encode(hasher.finalize())
    }

    /// Generate random token
    pub fn random_token() -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let bytes: [u8; 32] = rng.gen();
        hex::encode(bytes)
    }
}

/// Time utilities
pub struct TimeUtils;

impl TimeUtils {
    /// Get current timestamp
    pub fn now() -> chrono::DateTime<chrono::Utc> {
        chrono::Utc::now()
    }

    /// Format duration as human readable string
    pub fn format_duration(duration: chrono::Duration) -> String {
        let seconds = duration.num_seconds();
        let minutes = seconds / 60;
        let hours = minutes / 60;
        
        if hours > 0 {
            format!("{}h {}m {}s", hours, minutes % 60, seconds % 60)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, seconds % 60)
        } else {
            format!("{}s", seconds)
        }
    }

    /// Parse duration from string
    pub fn parse_duration(s: &str) -> Result<chrono::Duration> {
        // Simple duration parser for formats like "1h30m", "45s", etc.
        let mut duration = chrono::Duration::zero();
        let mut current_num = String::new();
        
        for ch in s.chars() {
            if ch.is_ascii_digit() {
                current_num.push(ch);
            } else {
                if let Ok(num) = current_num.parse::<i64>() {
                    match ch {
                        'h' => duration = duration + chrono::Duration::hours(num),
                        'm' => duration = duration + chrono::Duration::minutes(num),
                        's' => duration = duration + chrono::Duration::seconds(num),
                        _ => return Err(crate::error::BarefootError::Validation(
                            format!("Invalid duration format: {}", s)
                        )),
                    }
                }
                current_num.clear();
            }
        }
        
        Ok(duration)
    }
} 