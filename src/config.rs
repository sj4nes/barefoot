use crate::error::Result;
use crate::types::{RunnerConfig, ServiceConfig, ServiceType, RunnerCapabilities, ContainerCleanupConfig};
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

/// Differential logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifferentialLoggingConfig {
    pub enabled: bool,
    pub max_job_runs: usize,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub file: Option<String>,
    pub differential_logging: DifferentialLoggingConfig,
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
                container_backend: "native".to_string(),
                container_backend_opts: None,
                container_cleanup: ContainerCleanupConfig {
                    enabled: true,
                    interval_minutes: 60,
                    max_usage_bytes: 50_000_000_000, // 50GB
                },
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
                differential_logging: DifferentialLoggingConfig {
                    enabled: true,
                    max_job_runs: 25,
                },
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
        let config = config::Config::builder()
            .add_source(config::File::from(path.as_ref()))
            .add_source(config::Environment::with_prefix("BAREFOOT"))
            .build()
            .map_err(|e| crate::error::BarefootError::Configuration(e.to_string()))?;

        config
            .try_deserialize()
            .map_err(|e| crate::error::BarefootError::Configuration(e.to_string()))
    }

    /// Load configuration from multiple sources
    pub fn from_sources(sources: Vec<&str>) -> Result<Self> {
        let mut builder = config::Config::builder();

        for source in sources {
            builder = builder.add_source(config::File::with_name(source));
        }

        let config = builder
            .add_source(config::Environment::with_prefix("BAREFOOT"))
            .build()
            .map_err(|e| crate::error::BarefootError::Configuration(e.to_string()))?;

        config
            .try_deserialize()
            .map_err(|e| crate::error::BarefootError::Configuration(e.to_string()))
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        // Allow empty runner token for Jujutsu service type (local jobs)
        if self.runner.token.is_empty() && self.service.service_type != ServiceType::Jujutsu {
            return Err(crate::error::BarefootError::Configuration(
                "Runner token is required".to_string(),
            ));
        }

        // Allow empty service token for Jujutsu service type (local jobs)
        if self.service.token.is_empty() && self.service.service_type != ServiceType::Jujutsu {
            return Err(crate::error::BarefootError::Configuration(
                "Service token is required".to_string(),
            ));
        }

        if self.runner.max_concurrent_jobs == 0 {
            return Err(crate::error::BarefootError::Configuration(
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