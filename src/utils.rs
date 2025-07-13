use crate::error::Result;
use crate::types::{DiskUsageReport, ExecutionContext, Job, JobStep, StepResult, Workflow};
use async_trait::async_trait;
use reqwest::Url;
use std::collections::HashMap;
use std::path::Path;
use walkdir::WalkDir;

/// File system utilities
pub struct FileUtils;

impl FileUtils {
    /// Create directory if it doesn't exist
    pub fn ensure_dir<P: AsRef<Path>>(path: P) -> Result<()> {
        let path = path.as_ref();
        if !path.exists() {
            std::fs::create_dir_all(path).map_err(crate::error::BarefootError::Io)?;
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
                        .map_err(crate::error::BarefootError::Io)?;
                } else {
                    std::fs::remove_file(entry.path()).map_err(crate::error::BarefootError::Io)?;
                }
            }
        }
        Ok(())
    }

    /// Get file size in bytes
    pub fn file_size<P: AsRef<Path>>(path: P) -> Result<u64> {
        let metadata = std::fs::metadata(path).map_err(crate::error::BarefootError::Io)?;
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
            .map_err(crate::error::BarefootError::Io)
            .and_then(|path| {
                path.to_str().map(|s| s.to_string()).ok_or_else(|| {
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
        Url::parse(url)
            .ok()
            .and_then(|u| u.host_str().map(|h| h.to_string()))
    }

    /// Validate URL format
    pub fn is_valid_url(url: &str) -> bool {
        Url::parse(url).is_ok()
    }
}

/// Crypto utilities
pub struct CryptoUtils;
// TODO: Implement cryptographic utilities as needed

impl CryptoUtils {
    /// Generate HMAC-SHA256 signature
    pub fn hmac_sha256(key: &[u8], data: &[u8]) -> String {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        let mut mac = Hmac::<Sha256>::new_from_slice(key).expect("HMAC can take key of any size");
        mac.update(data);
        hex::encode(mac.finalize().into_bytes())
    }

    /// Generate SHA256 hash
    pub fn sha256(data: &[u8]) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(data);
        hex::encode(hasher.finalize())
    }

    /// Generate random token
    pub fn random_token() -> String {
        use rand::Rng;
        let mut rng = rand::rng();
        let bytes: [u8; 32] = rng.random();
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
            format!("{seconds}s")
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
                        'h' => duration += chrono::Duration::hours(num),
                        'm' => duration += chrono::Duration::minutes(num),
                        's' => duration += chrono::Duration::seconds(num),
                        _ => {
                            return Err(crate::error::BarefootError::Validation(format!(
                                "Invalid duration format: {s}"
                            )))
                        }
                    }
                }
                current_num.clear();
            }
        }

        Ok(duration)
    }
}

#[async_trait]
pub trait ContainerBackend: Send + Sync {
    async fn run_step(&self, step: &JobStep, context: &ExecutionContext) -> Result<StepResult>;
    async fn prepare_environment(&self, job: &Job, workflow: &Workflow) -> Result<()>;
    async fn disk_usage(&self) -> Result<DiskUsageReport>;
    async fn cleanup(&self) -> Result<()>;
}

/// Generate a random token
pub fn generate_token() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    use rand::Rng;
    let mut rng = rand::rng();
    let token: String = (0..32)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();
    token
}

/// Truncate logs to show first N lines, last M lines, with omitted marker
pub fn truncate_logs(logs: &str, first_lines: usize, last_lines: usize) -> String {
    let lines: Vec<&str> = logs.lines().collect();
    let total_lines = lines.len();
    let omitted = total_lines.saturating_sub(first_lines + last_lines);

    if omitted <= 15 {
        // If omitted lines is 15 or fewer, return the full log as-is, but ensure trailing newline
        let mut result = logs.to_string();
        if !result.ends_with('\n') {
            result.push('\n');
        }
        return result;
    }

    let mut truncated = String::new();

    // Add first N lines
    for line in lines.iter().take(first_lines) {
        truncated.push_str(line);
        truncated.push('\n');
    }

    // Add omitted marker
    truncated.push_str(&format!("... {} lines omitted ...\n", omitted));

    // Add last M lines
    for (i, line) in lines.iter().skip(total_lines - last_lines).enumerate() {
        truncated.push_str(line);
        if i < last_lines - 1 {
            truncated.push('\n');
        }
    }
    // Always add a trailing newline after truncation
    truncated.push('\n');

    truncated
}

/// Parse environment variables into a HashMap
pub fn parse_env_vars(env_str: &str) -> Result<HashMap<String, String>> {
    let mut env_vars = HashMap::new();

    for line in env_str.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some(equal_pos) = line.find('=') {
            let key = line[..equal_pos].trim();
            let value = line[equal_pos + 1..].trim();

            if !key.is_empty() {
                env_vars.insert(key.to_string(), value.to_string());
            }
        }
    }

    Ok(env_vars)
}

/// Validate a URL
pub fn validate_url(url: &str) -> Result<()> {
    if url.is_empty() {
        return Err(crate::error::BarefootError::Configuration(
            "URL cannot be empty".to_string(),
        ));
    }

    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(crate::error::BarefootError::Configuration(
            "URL must start with http:// or https://".to_string(),
        ));
    }

    Ok(())
}

/// Validate a token
pub fn validate_token(token: &str) -> Result<()> {
    if token.is_empty() {
        return Err(crate::error::BarefootError::Configuration(
            "Token cannot be empty".to_string(),
        ));
    }

    if token.len() < 8 {
        return Err(crate::error::BarefootError::Configuration(
            "Token must be at least 8 characters long".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_token() {
        let token1 = generate_token();
        let token2 = generate_token();

        assert_eq!(token1.len(), 32);
        assert_eq!(token2.len(), 32);
        assert_ne!(token1, token2);
    }

    #[test]
    fn test_truncate_logs_short() {
        let short_logs = "line1\nline2\nline3";
        let truncated = truncate_logs(short_logs, 1, 1);
        let expected = "line1\nline2\nline3\n";
        assert_eq!(truncated, expected);
    }

    #[test]
    fn test_truncate_logs_long() {
        // 25 lines, first 3, last 5, omitted = 17 (should truncate)
        let long_logs = (1..=25)
            .map(|i| format!("line{}", i))
            .collect::<Vec<_>>()
            .join("\n");
        let truncated = truncate_logs(&long_logs, 3, 5);
        let expected = "line1\nline2\nline3\n... 17 lines omitted ...\nline21\nline22\nline23\nline24\nline25\n";
        assert_eq!(truncated, expected);
    }

    #[test]
    fn test_truncate_logs_exact() {
        // 11 lines, first 3, last 5, omitted = 3 (should not truncate)
        let exact_logs = (1..=11)
            .map(|i| format!("line{}", i))
            .collect::<Vec<_>>()
            .join("\n");
        let truncated = truncate_logs(&exact_logs, 3, 5);
        let expected = (1..=11).map(|i| format!("line{}\n", i)).collect::<String>();
        assert_eq!(truncated, expected);
    }

    #[test]
    fn test_truncate_logs_just_below_threshold() {
        // 23 lines, first 3, last 5, omitted = 15 (should not truncate)
        let logs = (1..=23)
            .map(|i| format!("line{}", i))
            .collect::<Vec<_>>()
            .join("\n");
        let truncated = truncate_logs(&logs, 3, 5);
        let expected = (1..=23).map(|i| format!("line{}\n", i)).collect::<String>();
        assert_eq!(truncated, expected);
    }

    #[test]
    fn test_parse_env_vars() {
        let env_str = "KEY1=value1\nKEY2=value2\n# comment\n\nKEY3=value3";
        let result = parse_env_vars(env_str).unwrap();

        assert_eq!(result.get("KEY1"), Some(&"value1".to_string()));
        assert_eq!(result.get("KEY2"), Some(&"value2".to_string()));
        assert_eq!(result.get("KEY3"), Some(&"value3".to_string()));
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_validate_url() {
        assert!(validate_url("https://api.github.com").is_ok());
        assert!(validate_url("http://localhost:8080").is_ok());
        assert!(validate_url("").is_err());
        assert!(validate_url("ftp://example.com").is_err());
    }

    #[test]
    fn test_validate_token() {
        assert!(validate_token("valid_token_123").is_ok());
        assert!(validate_token("").is_err());
        assert!(validate_token("short").is_err());
    }
}
