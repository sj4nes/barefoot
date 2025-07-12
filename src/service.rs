use crate::error::{BarefootError, Result};
use crate::config::BarefootConfig;
use crate::types::{Job, RunnerCapabilities};
use async_trait::async_trait;
use reqwest::Client;
use std::collections::HashMap;

/// Service client trait for different CI/CD platforms
#[async_trait]
pub trait ServiceClient: Send + Sync {
    /// Get available jobs
    async fn get_jobs(&self) -> Result<Vec<Job>>;
    
    /// Update job status
    async fn update_job_status(&self, job_id: &str, status: &str) -> Result<()>;
    
    /// Send job logs
    async fn send_job_logs(&self, job_id: &str, logs: &str) -> Result<()>;
    
    /// Register runner with the service
    async fn register_runner(&self, capabilities: &RunnerCapabilities) -> Result<()>;
    
    /// Deregister runner from the service
    async fn deregister_runner(&self) -> Result<()>;
}

/// GitHub Actions client implementation
pub struct GitHubClient {
    client: Client,
    base_url: String,
    token: String,
}

impl GitHubClient {
    pub fn new(config: BarefootConfig) -> Self {
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), format!("token {}", config.service.token));
        headers.insert("Accept".to_string(), "application/vnd.github.v3+json".to_string());
        headers.insert("User-Agent".to_string(), "barefoot-runner".to_string());
        
        let client = Client::builder()
            .default_headers(headers.into_iter().map(|(k, v)| {
                (reqwest::header::HeaderName::from_bytes(k.as_bytes()).unwrap(),
                 reqwest::header::HeaderValue::from_str(&v).unwrap())
            }).collect())
            .build()
            .unwrap();
        
        Self {
            client,
            base_url: config.service.url,
            token: config.service.token,
        }
    }
}

#[async_trait]
impl ServiceClient for GitHubClient {
    async fn get_jobs(&self) -> Result<Vec<Job>> {
        let response = self.client
            .get(&format!("{}/actions/runners/jobs", self.base_url))
            .send()
            .await
            .map_err(|e| BarefootError::HttpRequest(e))?;
        
        if !response.status().is_success() {
            return Err(BarefootError::HttpStatus { 
                status: response.status().as_u16() 
            }.into());
        }
        
        let jobs: Vec<Job> = response.json()
            .await
            .map_err(|e| BarefootError::HttpRequest(e))?;
        
        Ok(jobs)
    }
    
    async fn update_job_status(&self, job_id: &str, status: &str) -> Result<()> {
        let response = self.client
            .patch(&format!("{}/actions/jobs/{}", self.base_url, job_id))
            .json(&serde_json::json!({
                "status": status
            }))
            .send()
            .await
            .map_err(|e| BarefootError::HttpRequest(e))?;
        
        if !response.status().is_success() {
            return Err(BarefootError::HttpStatus { 
                status: response.status().as_u16() 
            }.into());
        }
        
        Ok(())
    }
    
    async fn send_job_logs(&self, job_id: &str, logs: &str) -> Result<()> {
        let response = self.client
            .post(&format!("{}/actions/jobs/{}/logs", self.base_url, job_id))
            .body(logs.to_string())
            .header("Content-Type", "text/plain")
            .send()
            .await
            .map_err(|e| BarefootError::HttpRequest(e))?;
        
        if !response.status().is_success() {
            return Err(BarefootError::HttpStatus { 
                status: response.status().as_u16() 
            }.into());
        }
        
        Ok(())
    }
    
    async fn register_runner(&self, capabilities: &RunnerCapabilities) -> Result<()> {
        let response = self.client
            .post(&format!("{}/actions/runners/registration-token", self.base_url))
            .json(&serde_json::json!({
                "name": "barefoot-runner",
                "capabilities": capabilities
            }))
            .send()
            .await
            .map_err(|e| BarefootError::HttpRequest(e))?;
        
        if !response.status().is_success() {
            return Err(BarefootError::HttpStatus { 
                status: response.status().as_u16() 
            }.into());
        }
        
        Ok(())
    }
    
    async fn deregister_runner(&self) -> Result<()> {
        let response = self.client
            .delete(&format!("{}/actions/runners/self", self.base_url))
            .send()
            .await
            .map_err(|e| BarefootError::HttpRequest(e))?;
        
        if !response.status().is_success() {
            return Err(BarefootError::HttpStatus { 
                status: response.status().as_u16() 
            }.into());
        }
        
        Ok(())
    }
}

/// Jujutsu client implementation
pub struct JujutsuClient {
    client: Client,
    base_url: String,
    token: String,
}

impl JujutsuClient {
    pub fn new(config: BarefootConfig) -> Self {
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), format!("Bearer {}", config.service.token));
        headers.insert("Accept".to_string(), "application/json".to_string());
        headers.insert("User-Agent".to_string(), "barefoot-runner".to_string());
        
        let client = Client::builder()
            .default_headers(headers.into_iter().map(|(k, v)| {
                (reqwest::header::HeaderName::from_bytes(k.as_bytes()).unwrap(),
                 reqwest::header::HeaderValue::from_str(&v).unwrap())
            }).collect())
            .build()
            .unwrap();
        
        Self {
            client,
            base_url: config.service.url,
            token: config.service.token,
        }
    }
}

#[async_trait]
impl ServiceClient for JujutsuClient {
    async fn get_jobs(&self) -> Result<Vec<Job>> {
        let response = self.client
            .get(&format!("{}/jobs", self.base_url))
            .send()
            .await
            .map_err(|e| BarefootError::HttpRequest(e))?;
        
        if !response.status().is_success() {
            return Err(BarefootError::HttpStatus { 
                status: response.status().as_u16() 
            }.into());
        }
        
        let jobs: Vec<Job> = response.json()
            .await
            .map_err(|e| BarefootError::HttpRequest(e))?;
        
        Ok(jobs)
    }
    
    async fn update_job_status(&self, job_id: &str, status: &str) -> Result<()> {
        let response = self.client
            .patch(&format!("{}/jobs/{}", self.base_url, job_id))
            .json(&serde_json::json!({
                "status": status
            }))
            .send()
            .await
            .map_err(|e| BarefootError::HttpRequest(e))?;
        
        if !response.status().is_success() {
            return Err(BarefootError::HttpStatus { 
                status: response.status().as_u16() 
            }.into());
        }
        
        Ok(())
    }
    
    async fn send_job_logs(&self, job_id: &str, logs: &str) -> Result<()> {
        let response = self.client
            .post(&format!("{}/jobs/{}/logs", self.base_url, job_id))
            .body(logs.to_string())
            .header("Content-Type", "text/plain")
            .send()
            .await
            .map_err(|e| BarefootError::HttpRequest(e))?;
        
        if !response.status().is_success() {
            return Err(BarefootError::HttpStatus { 
                status: response.status().as_u16() 
            }.into());
        }
        
        Ok(())
    }
    
    async fn register_runner(&self, capabilities: &RunnerCapabilities) -> Result<()> {
        let response = self.client
            .post(&format!("{}/runners", self.base_url))
            .json(&serde_json::json!({
                "name": "barefoot-runner",
                "capabilities": capabilities
            }))
            .send()
            .await
            .map_err(|e| BarefootError::HttpRequest(e))?;
        
        if !response.status().is_success() {
            return Err(BarefootError::HttpStatus { 
                status: response.status().as_u16() 
            }.into());
        }
        
        Ok(())
    }
    
    async fn deregister_runner(&self) -> Result<()> {
        let response = self.client
            .delete(&format!("{}/runners/self", self.base_url))
            .send()
            .await
            .map_err(|e| BarefootError::HttpRequest(e))?;
        
        if !response.status().is_success() {
            return Err(BarefootError::HttpStatus { 
                status: response.status().as_u16() 
            }.into());
        }
        
        Ok(())
    }
}

/// Service client factory
pub struct ServiceClientFactory;

impl ServiceClientFactory {
    pub fn create_client(config: BarefootConfig) -> Result<Box<dyn ServiceClient + Send + Sync>> {
        match config.service.service_type {
            crate::types::ServiceType::GitHub => {
                Ok(Box::new(GitHubClient::new(config)))
            }
            crate::types::ServiceType::Jujutsu => {
                Ok(Box::new(JujutsuClient::new(config)))
            }
            _ => {
                Err(BarefootError::ServiceNotFound(
                    format!("Service type {:?} not supported", config.service.service_type)
                ))
            }
        }
    }
} 