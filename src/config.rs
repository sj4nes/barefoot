use crate::{error::Result, types::*};
use config::{Config, Environment, File};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Main configuration for the barefoot runner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarefootConfig {
    pub runner: RunnerConfig,
    pub service: ServiceConfig,
    pub logging: LoggingConfig,
    pub security: SecurityConfig,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub file: Option<String>,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub enable_ssl_verification: bool,
    pub allowed_origins: Vec<String>,
    pub max_upload_size: usize,
}

impl Default for BarefootConfig {
    fn default() -> Self {
        Self {
            runner: RunnerConfig {
                name: "barefoot-runner".to_string(),
                url: "http://localhost:8080".to_string(),
                token: "".to_string(),
                labels: vec!["self-hosted".to_string()],
                capabilities: RunnerCapabilities {
                    os: std::env::consts::OS.to_string(),
                    architecture: std::env::consts::ARCH.to_string(),
                    labels: vec!["self-hosted".to_string()],
                    features: HashMap::new(),
                },
                max_concurrent_jobs: 1,
                work_dir: "./work".to_string(),
            },
            service: ServiceConfig {
                service_type: ServiceType::GitHub,
                url: "https://api.github.com".to_string(),
                token: "".to_string(),
                api_version: None,
                custom_headers: HashMap::new(),
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "json".to_string(),
                file: None,
            },
            security: SecurityConfig {
                enable_ssl_verification: true,
                allowed_origins: vec!["*".to_string()],
                max_upload_size: 100 * 1024 * 1024, // 100MB
            },
        }
    }
}

impl BarefootConfig {
    /// Load configuration from file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let config = Config::builder()
            .add_source(File::from(path.as_ref()))
            .add_source(Environment::with_prefix("BAREFOOT"))
            .build()
            .map_err(|e| crate::error::BarefootError::Config(e.to_string()))?;

        config
            .try_deserialize()
            .map_err(|e| crate::error::BarefootError::Config(e.to_string()))
    }

    /// Load configuration from multiple sources
    pub fn from_sources(sources: Vec<&str>) -> Result<Self> {
        let mut builder = Config::builder();

        for source in sources {
            builder = builder.add_source(File::with_name(source));
        }

        let config = builder
            .add_source(Environment::with_prefix("BAREFOOT"))
            .build()
            .map_err(|e| crate::error::BarefootError::Config(e.to_string()))?;

        config
            .try_deserialize()
            .map_err(|e| crate::error::BarefootError::Config(e.to_string()))
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        if self.runner.token.is_empty() {
            return Err(crate::error::BarefootError::Config(
                "Runner token is required".to_string(),
            ));
        }

        if self.service.token.is_empty() {
            return Err(crate::error::BarefootError::Config(
                "Service token is required".to_string(),
            ));
        }

        if self.runner.max_concurrent_jobs == 0 {
            return Err(crate::error::BarefootError::Config(
                "Max concurrent jobs must be greater than 0".to_string(),
            ));
        }

        Ok(())
    }

    /// Get the work directory path
    pub fn work_dir(&self) -> &str {
        &self.runner.work_dir
    }

    /// Get the runner name
    pub fn runner_name(&self) -> &str {
        &self.runner.name
    }

    /// Get the service URL
    pub fn service_url(&self) -> &str {
        &self.service.url
    }
} 