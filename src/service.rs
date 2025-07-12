use crate::{
    config::BarefootConfig,
    error::Result,
    types::{Job, ServiceType},
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Service client trait
pub trait ServiceClient {
    /// Get available jobs
    async fn get_jobs(&self) -> Result<Vec<Job>>;
    
    /// Update job status
    async fn update_job_status(&self, job_id: &str, status: &str) -> Result<()>;
    
    /// Send job logs
    async fn send_job_logs(&self, job_id: &str, logs: &str) -> Result<()>;
    
    /// Register runner
    async fn register_runner(&self, capabilities: &crate::types::RunnerCapabilities) -> Result<()>;
    
    /// Deregister runner
    async fn deregister_runner(&self) -> Result<()>;
}

/// GitHub service client
pub struct GitHubClient {
    client: Client,
    config: BarefootConfig,
}

impl GitHubClient {
    pub fn new(config: BarefootConfig) -> Self {
        let mut headers = HashMap::new();
        headers.insert(
            "Authorization".to_string(),
            format!("token {}", config.service.token),
        );
        headers.insert("User-Agent".to_string(), "barefoot-runner".to_string());
        headers.insert("Accept".to_string(), "application/vnd.github.v3+json".to_string());

        let client = Client::builder()
            .default_headers(headers.into_iter().collect())
            .build()
            .expect("Failed to create HTTP client");

        Self { client, config }
    }
}

#[async_trait::async_trait]
impl ServiceClient for GitHubClient {
    async fn get_jobs(&self) -> Result<Vec<Job>> {
        let url = format!("{}/api/v3/actions/runners/jobs", self.config.service.url);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| crate::error::BarefootError::Network(e))?;

        if !response.status().is_success() {
            return Err(crate::error::BarefootError::Network(
                reqwest::Error::status_code(response.status())
            ));
        }

        let jobs: Vec<Job> = response.json().await
            .map_err(|e| crate::error::BarefootError::Serialization(e))?;

        Ok(jobs)
    }

    async fn update_job_status(&self, job_id: &str, status: &str) -> Result<()> {
        let url = format!("{}/api/v3/actions/runners/jobs/{}/status", self.config.service.url, job_id);
        
        let payload = serde_json::json!({
            "status": status
        });

        let response = self.client
            .patch(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| crate::error::BarefootError::Network(e))?;

        if !response.status().is_success() {
            return Err(crate::error::BarefootError::Network(
                reqwest::Error::status_code(response.status())
            ));
        }

        Ok(())
    }

    async fn send_job_logs(&self, job_id: &str, logs: &str) -> Result<()> {
        let url = format!("{}/api/v3/actions/runners/jobs/{}/logs", self.config.service.url, job_id);
        
        let payload = serde_json::json!({
            "logs": logs
        });

        let response = self.client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| crate::error::BarefootError::Network(e))?;

        if !response.status().is_success() {
            return Err(crate::error::BarefootError::Network(
                reqwest::Error::status_code(response.status())
            ));
        }

        Ok(())
    }

    async fn register_runner(&self, capabilities: &crate::types::RunnerCapabilities) -> Result<()> {
        let url = format!("{}/api/v3/actions/runners/registration-token", self.config.service.url);
        
        let payload = serde_json::json!({
            "name": self.config.runner.name,
            "labels": capabilities.labels,
            "runs_on": capabilities.os
        });

        let response = self.client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| crate::error::BarefootError::Network(e))?;

        if !response.status().is_success() {
            return Err(crate::error::BarefootError::Network(
                reqwest::Error::status_code(response.status())
            ));
        }

        Ok(())
    }

    async fn deregister_runner(&self) -> Result<()> {
        let url = format!("{}/api/v3/actions/runners/remove-token", self.config.service.url);
        
        let response = self.client
            .post(&url)
            .send()
            .await
            .map_err(|e| crate::error::BarefootError::Network(e))?;

        if !response.status().is_success() {
            return Err(crate::error::BarefootError::Network(
                reqwest::Error::status_code(response.status())
            ));
        }

        Ok(())
    }
}

/// Jujutsu service client
pub struct JujutsuClient {
    client: Client,
    config: BarefootConfig,
}

impl JujutsuClient {
    pub fn new(config: BarefootConfig) -> Self {
        let mut headers = HashMap::new();
        headers.insert(
            "Authorization".to_string(),
            format!("Bearer {}", config.service.token),
        );
        headers.insert("User-Agent".to_string(), "barefoot-runner".to_string());

        let client = Client::builder()
            .default_headers(headers.into_iter().collect())
            .build()
            .expect("Failed to create HTTP client");

        Self { client, config }
    }
}

#[async_trait::async_trait]
impl ServiceClient for JujutsuClient {
    async fn get_jobs(&self) -> Result<Vec<Job>> {
        let url = format!("{}/api/v1/jobs", self.config.service.url);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| crate::error::BarefootError::Network(e))?;

        if !response.status().is_success() {
            return Err(crate::error::BarefootError::Network(
                reqwest::Error::status_code(response.status())
            ));
        }

        let jobs: Vec<Job> = response.json().await
            .map_err(|e| crate::error::BarefootError::Serialization(e))?;

        Ok(jobs)
    }

    async fn update_job_status(&self, job_id: &str, status: &str) -> Result<()> {
        let url = format!("{}/api/v1/jobs/{}/status", self.config.service.url, job_id);
        
        let payload = serde_json::json!({
            "status": status
        });

        let response = self.client
            .patch(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| crate::error::BarefootError::Network(e))?;

        if !response.status().is_success() {
            return Err(crate::error::BarefootError::Network(
                reqwest::Error::status_code(response.status())
            ));
        }

        Ok(())
    }

    async fn send_job_logs(&self, job_id: &str, logs: &str) -> Result<()> {
        let url = format!("{}/api/v1/jobs/{}/logs", self.config.service.url, job_id);
        
        let payload = serde_json::json!({
            "logs": logs
        });

        let response = self.client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| crate::error::BarefootError::Network(e))?;

        if !response.status().is_success() {
            return Err(crate::error::BarefootError::Network(
                reqwest::Error::status_code(response.status())
            ));
        }

        Ok(())
    }

    async fn register_runner(&self, capabilities: &crate::types::RunnerCapabilities) -> Result<()> {
        let url = format!("{}/api/v1/runners/register", self.config.service.url);
        
        let payload = serde_json::json!({
            "name": self.config.runner.name,
            "capabilities": capabilities
        });

        let response = self.client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| crate::error::BarefootError::Network(e))?;

        if !response.status().is_success() {
            return Err(crate::error::BarefootError::Network(
                reqwest::Error::status_code(response.status())
            ));
        }

        Ok(())
    }

    async fn deregister_runner(&self) -> Result<()> {
        let url = format!("{}/api/v1/runners/deregister", self.config.service.url);
        
        let response = self.client
            .post(&url)
            .send()
            .await
            .map_err(|e| crate::error::BarefootError::Network(e))?;

        if !response.status().is_success() {
            return Err(crate::error::BarefootError::Network(
                reqwest::Error::status_code(response.status())
            ));
        }

        Ok(())
    }
}

/// Service factory
pub struct ServiceFactory;

impl ServiceFactory {
    /// Create a service client based on configuration
    pub fn create_client(config: BarefootConfig) -> Result<Box<dyn ServiceClient + Send + Sync>> {
        match config.service.service_type {
            ServiceType::GitHub => {
                Ok(Box::new(GitHubClient::new(config)))
            }
            ServiceType::Jujutsu => {
                Ok(Box::new(JujutsuClient::new(config)))
            }
            _ => {
                Err(crate::error::BarefootError::ServiceNotFound(
                    format!("Service type {:?} not yet implemented", config.service.service_type)
                ))
            }
        }
    }
} 