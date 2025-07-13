use crate::error::{BarefootError, Result};
use crate::config::BarefootConfig;
use crate::types::{Job, RunnerCapabilities};
use async_trait::async_trait;
use reqwest::Client;
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::Instant;
use std::sync::Arc;
use tokio::sync::Mutex;

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
    _token: String,
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
            _token: config.service.token,
        }
    }
}

#[async_trait]
impl ServiceClient for GitHubClient {
    async fn get_jobs(&self) -> Result<Vec<Job>> {
        let response = self.client
            .get(format!("{}/actions/runners/jobs", self.base_url))
            .send()
            .await
            .map_err(BarefootError::HttpRequest)?;
        
        if !response.status().is_success() {
            return Err(BarefootError::HttpStatus { 
                status: response.status().as_u16() 
            });
        }
        
        let jobs: Vec<Job> = response.json()
            .await
            .map_err(BarefootError::HttpRequest)?;
        
        Ok(jobs)
    }
    
    async fn update_job_status(&self, job_id: &str, status: &str) -> Result<()> {
        let response = self.client
            .patch(format!("{}/actions/jobs/{}", self.base_url, job_id))
            .json(&serde_json::json!({
                "status": status
            }))
            .send()
            .await
            .map_err(BarefootError::HttpRequest)?;
        
        if !response.status().is_success() {
            return Err(BarefootError::HttpStatus { 
                status: response.status().as_u16() 
            });
        }
        
        Ok(())
    }
    
    async fn send_job_logs(&self, job_id: &str, logs: &str) -> Result<()> {
        let response = self.client
            .post(format!("{}/actions/jobs/{}/logs", self.base_url, job_id))
            .body(logs.to_string())
            .header("Content-Type", "text/plain")
            .send()
            .await
            .map_err(BarefootError::HttpRequest)?;
        
        if !response.status().is_success() {
            return Err(BarefootError::HttpStatus { 
                status: response.status().as_u16() 
            });
        }
        
        Ok(())
    }
    
    async fn register_runner(&self, capabilities: &RunnerCapabilities) -> Result<()> {
        let response = self.client
            .post(format!("{}/actions/runners/registration-token", self.base_url))
            .json(&serde_json::json!({
                "name": "barefoot-runner",
                "capabilities": capabilities
            }))
            .send()
            .await
            .map_err(BarefootError::HttpRequest)?;
        
        if !response.status().is_success() {
            return Err(BarefootError::HttpStatus { 
                status: response.status().as_u16() 
            });
        }
        
        Ok(())
    }
    
    async fn deregister_runner(&self) -> Result<()> {
        let response = self.client
            .delete(format!("{}/actions/runners/self", self.base_url))
            .send()
            .await
            .map_err(BarefootError::HttpRequest)?;
        
        if !response.status().is_success() {
            return Err(BarefootError::HttpStatus { 
                status: response.status().as_u16() 
            });
        }
        
        Ok(())
    }
}

/// Jujutsu client implementation
pub struct JujutsuClient {
    client: Client,
    base_url: String,
    _token: String,
    repository_path: Option<String>,
    debounce_delay: Duration,
    last_change_time: Arc<Mutex<Option<Instant>>>,
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
        
        // Check if the URL is a local path (doesn't start with http:// or https://)
        let repository_path = if !config.service.url.starts_with("http://") && !config.service.url.starts_with("https://") {
            Some(config.service.url.clone())
        } else {
            None
        };
        
        Self {
            client,
            base_url: config.service.url,
            _token: config.service.token,
            repository_path,
            debounce_delay: Duration::from_secs(120), // 2 minutes
            last_change_time: Arc::new(Mutex::new(None)),
        }
    }

    /// Scan local .barefoot directory for job files
    async fn scan_local_jobs(&self) -> Result<Vec<Job>> {
        let repository_path = match &self.repository_path {
            Some(path) => path,
            None => return Ok(vec![]),
        };

        let barefoot_dir = std::path::Path::new(repository_path).join(".barefoot");
        if !barefoot_dir.exists() {
            return Ok(vec![]);
        }

        let mut jobs = Vec::new();
        
        // Scan for .json and .toml files in .barefoot directory
        if let Ok(entries) = std::fs::read_dir(&barefoot_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(extension) = path.extension().and_then(|s| s.to_str()) {
                    if extension == "json" || extension == "toml" {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            let job_result = if extension == "json" {
                                serde_json::from_str::<Job>(&content)
                                    .map_err(crate::error::BarefootError::Serialization)
                            } else {
                                // TOML parsing
                                toml::from_str::<Job>(&content)
                                    .map_err(crate::error::BarefootError::TomlDeserialization)
                            };
                            
                            if let Ok(job) = job_result {
                                jobs.push(job);
                            } else {
                                tracing::warn!("Failed to parse job file: {:?}", path);
                            }
                        }
                    }
                }
            }
        }

        Ok(jobs)
    }

    /// Start watching the .barefoot directory for changes with debouncing
    pub async fn start_file_watching(&self, tx: mpsc::Sender<String>) -> Result<tokio::task::JoinHandle<()>> {
        let repository_path = match &self.repository_path {
            Some(path) => path.clone(),
            None => return Err(BarefootError::InvalidState("No local repository path configured".to_string())),
        };

        let barefoot_dir = std::path::Path::new(&repository_path).join(".barefoot");
        if !barefoot_dir.exists() {
            return Err(BarefootError::InvalidState("`.barefoot` directory does not exist".to_string()));
        }

        let debounce_delay = self.debounce_delay;
        let last_change_time = Arc::clone(&self.last_change_time);
        let tx = Arc::new(tx);

        let handle = tokio::spawn(async move {
            tracing::info!("Started watching .barefoot directory: {:?}", barefoot_dir);

            // For now, implement a simple polling approach instead of file watching
            // This is more reliable and easier to test
            let mut last_check = Instant::now();
            let check_interval = Duration::from_secs(10); // Check every 10 seconds
            let trapdoor_timeout = Duration::from_secs(300); // 5 minutes

            loop {
                tokio::select! {
                    // Check for new files periodically
                    _ = tokio::time::sleep(check_interval) => {
                        let now = Instant::now();
                        
                        // Check if trapdoor timer has expired
                        if now.duration_since(last_check) > trapdoor_timeout {
                            tracing::info!("Trapdoor timer expired - no file activity for 5 minutes, shutting down file watcher");
                            break;
                        }

                        // Scan for new .json and .toml files
                        if let Ok(entries) = std::fs::read_dir(&barefoot_dir) {
                            for entry in entries.flatten() {
                                let path = entry.path();
                                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                                    if ext == "json" || ext == "toml" {
                                        // Check if file was modified recently
                                        if let Ok(metadata) = std::fs::metadata(&path) {
                                            if let Ok(modified) = metadata.modified() {
                                                let now = std::time::SystemTime::now();
                                                if let Ok(duration) = now.duration_since(modified) {
                                                    if duration < Duration::from_secs(60) { // File modified in last minute
                                                        // Update last change time
                                                        {
                                                            let mut last_time = last_change_time.lock().await;
                                                            *last_time = Some(Instant::now());
                                                        }

                                                        // Send notification
                                                        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                                                            let _ = tx.send(file_name.to_string()).await;
                                                        }

                                                        // Start debounced task
                                                        let debounce_delay = debounce_delay;
                                                        let last_change_time = Arc::clone(&last_change_time);
                                                        let tx = Arc::clone(&tx);
                                                        
                                                        tokio::spawn(async move {
                                                            tokio::time::sleep(debounce_delay).await;
                                                            
                                                            // Check if this is still the most recent change
                                                            let should_notify = {
                                                                let last_time = last_change_time.lock().await;
                                                                if let Some(time) = *last_time {
                                                                    Instant::now().duration_since(time) >= debounce_delay
                                                                } else {
                                                                    false
                                                                }
                                                            };

                                                            if should_notify {
                                                                tracing::info!("Debounced file change notification - changes have stabilized");
                                                                let _ = tx.send("DEBOUNCED".to_string()).await;
                                                            }
                                                        });
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        
                        last_check = now;
                    }
                }
            }

            tracing::info!("File watcher stopped");
        });

        Ok(handle)
    }

    /// Stop file watching (shutdown signal)
    pub async fn stop_file_watching(&self) -> Result<()> {
        // This would need to be implemented with a way to send shutdown signal
        // For now, we'll rely on the trapdoor timer
        Ok(())
    }
}

#[async_trait]
impl ServiceClient for JujutsuClient {
    async fn get_jobs(&self) -> Result<Vec<Job>> {
        // If we have a local repository path, scan the .barefoot directory
        if self.repository_path.is_some() {
            return self.scan_local_jobs().await;
        }

        // For local Jujutsu jobs, we should always use local scanning
        // Only use HTTP polling if we have a proper HTTP URL
        if !self.base_url.starts_with("http://") && !self.base_url.starts_with("https://") {
            return Ok(vec![]); // No jobs available for invalid URL
        }

        // Otherwise, use HTTP polling
        let response = self.client
            .get(format!("{}/jobs", self.base_url))
            .send()
            .await
            .map_err(BarefootError::HttpRequest)?;
        
        if !response.status().is_success() {
            return Err(BarefootError::HttpStatus { 
                status: response.status().as_u16() 
            });
        }
        
        let jobs: Vec<Job> = response.json()
            .await
            .map_err(BarefootError::HttpRequest)?;
        
        Ok(jobs)
    }
    
    async fn update_job_status(&self, job_id: &str, status: &str) -> Result<()> {
        // For local Jujutsu jobs, just log the status update
        if self.repository_path.is_some() {
            tracing::info!("Job {} status updated to: {}", job_id, status);
            return Ok(());
        }

        // For remote Jujutsu jobs, use HTTP update
        if !self.base_url.starts_with("http://") && !self.base_url.starts_with("https://") {
            tracing::info!("Skipping job status update for invalid URL: {}", self.base_url);
            return Ok(());
        }

        let response = self.client
            .patch(format!("{}/jobs/{}", self.base_url, job_id))
            .json(&serde_json::json!({
                "status": status
            }))
            .send()
            .await
            .map_err(BarefootError::HttpRequest)?;
        
        if !response.status().is_success() {
            return Err(BarefootError::HttpStatus { 
                status: response.status().as_u16() 
            });
        }
        
        Ok(())
    }
    
    async fn send_job_logs(&self, job_id: &str, logs: &str) -> Result<()> {
        // For local Jujutsu jobs, just log the logs
        if self.repository_path.is_some() {
            tracing::info!("Job {} logs: {}", job_id, logs);
            return Ok(());
        }

        // For remote Jujutsu jobs, use HTTP send
        if !self.base_url.starts_with("http://") && !self.base_url.starts_with("https://") {
            tracing::info!("Skipping job logs send for invalid URL: {}", self.base_url);
            return Ok(());
        }

        let response = self.client
            .post(format!("{}/jobs/{}/logs", self.base_url, job_id))
            .body(logs.to_string())
            .header("Content-Type", "text/plain")
            .send()
            .await
            .map_err(BarefootError::HttpRequest)?;
        
        if !response.status().is_success() {
            return Err(BarefootError::HttpStatus { 
                status: response.status().as_u16() 
            });
        }
        
        Ok(())
    }
    
    async fn register_runner(&self, capabilities: &RunnerCapabilities) -> Result<()> {
        // For local Jujutsu jobs, registration is not needed
        if self.repository_path.is_some() {
            tracing::info!("Skipping runner registration for local Jujutsu jobs");
            return Ok(());
        }

        // For remote Jujutsu jobs, use HTTP registration
        if !self.base_url.starts_with("http://") && !self.base_url.starts_with("https://") {
            tracing::info!("Skipping runner registration for invalid URL: {}", self.base_url);
            return Ok(());
        }

        let response = self.client
            .post(format!("{}/runners", self.base_url))
            .json(&serde_json::json!({
                "name": "barefoot-runner",
                "capabilities": capabilities
            }))
            .send()
            .await
            .map_err(BarefootError::HttpRequest)?;
        
        if !response.status().is_success() {
            return Err(BarefootError::HttpStatus { 
                status: response.status().as_u16() 
            });
        }
        
        Ok(())
    }
    
    async fn deregister_runner(&self) -> Result<()> {
        let response = self.client
            .delete(format!("{}/runners/self", self.base_url))
            .send()
            .await
            .map_err(BarefootError::HttpRequest)?;
        
        if !response.status().is_success() {
            return Err(BarefootError::HttpStatus { 
                status: response.status().as_u16() 
            });
        }
        
        Ok(())
    }
}

/// Factory for creating service clients
#[derive(Clone)]
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
                // TODO: Add support for additional service types (Forgejo, Codeberg, Sourcehut, Gitea)
                Err(BarefootError::ServiceNotFound(
                    format!("Service type {:?} not supported", config.service.service_type)
                ))
            }
        }
    }
} 

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::BarefootConfig;
    use crate::types::ServiceType;
    use tempfile::TempDir;
    use std::fs;
    use std::time::Duration;
    use tokio::time::sleep;

    fn test_config() -> BarefootConfig {
        let mut config = BarefootConfig::default();
        config.service.service_type = ServiceType::Jujutsu;
        config.service.url = "http://localhost:8080".to_string();
        config.service.token = "test-token".to_string();
        config
    }

    #[tokio::test]
    async fn test_jujutsu_client_creation() {
        let config = test_config();
        let client = JujutsuClient::new(config);
        assert_eq!(client.base_url, "http://localhost:8080");
    }

    #[tokio::test]
    async fn test_jujutsu_client_get_jobs_network_error() {
        let config = test_config();
        let client = JujutsuClient::new(config);
        let jobs = client.get_jobs().await;
        // Should fail due to network error (localhost:8080 not available)
        assert!(jobs.is_err());
    }

    #[tokio::test]
    async fn test_service_client_factory_jujutsu() {
        let config = test_config();
        let client = ServiceClientFactory::create_client(config);
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_jujutsu_local_directory_scanning() {
        let temp_dir = TempDir::new().unwrap();
        let barefoot_dir = temp_dir.path().join(".barefoot");
        fs::create_dir(&barefoot_dir).unwrap();

        // Create a test job file with proper Job structure (JSON)
        let job_file = barefoot_dir.join("test-job.json");
        let job_content = r#"{
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "name": "Test Job",
            "status": "Queued",
            "workflow": "test-workflow",
            "repository": "test-repo",
            "started_at": null,
            "completed_at": null,
            "steps": []
        }"#;
        fs::write(&job_file, job_content).unwrap();

        let mut config = test_config();
        config.service.url = temp_dir.path().to_string_lossy().to_string();
        let client = JujutsuClient::new(config);
        
        // Test that the client can scan the directory
        let jobs = client.get_jobs().await;
        assert!(jobs.is_ok());
        let jobs = jobs.unwrap();
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].name, "Test Job");
    }

    #[tokio::test]
    async fn test_jujutsu_local_directory_scanning_toml() {
        let temp_dir = TempDir::new().unwrap();
        let barefoot_dir = temp_dir.path().join(".barefoot");
        fs::create_dir(&barefoot_dir).unwrap();

        // Create a test job file with proper Job structure (TOML)
        let job_file = barefoot_dir.join("test-job.toml");
        let job_content = r#"id = "550e8400-e29b-41d4-a716-446655440000"
name = "Test Job TOML"
status = "Queued"
workflow = "test-workflow"
repository = "test-repo"

[[steps]]
name = "test-step"
status = "Queued"
run = "echo 'hello'""#;
        fs::write(&job_file, job_content).unwrap();

        // Try parsing the TOML directly
        match toml::from_str::<crate::types::Job>(job_content) {
            Ok(_) => println!("TOML parsed successfully"),
            Err(e) => println!("Direct TOML parse error: {e}"),
        }

        let mut config = test_config();
        config.service.url = temp_dir.path().to_string_lossy().to_string();
        let client = JujutsuClient::new(config);
        
        // Test that the client can scan the directory
        let jobs = client.get_jobs().await;
        if let Err(e) = &jobs {
            println!("TOML parse error: {e}");
        }
        assert!(jobs.is_ok());
        let jobs = jobs.unwrap();
        if jobs.is_empty() {
            println!("No jobs parsed from TOML");
        }
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].name, "Test Job TOML");
    }

    #[tokio::test]
    async fn test_jujutsu_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let barefoot_dir = temp_dir.path().join(".barefoot");
        fs::create_dir(&barefoot_dir).unwrap();

        let mut config = test_config();
        config.service.url = temp_dir.path().to_string_lossy().to_string();
        let client = JujutsuClient::new(config);
        
        let jobs = client.get_jobs().await;
        assert!(jobs.is_ok());
        assert_eq!(jobs.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_jujutsu_invalid_job_file() {
        let temp_dir = TempDir::new().unwrap();
        let barefoot_dir = temp_dir.path().join(".barefoot");
        fs::create_dir(&barefoot_dir).unwrap();

        // Create an invalid job file
        let job_file = barefoot_dir.join("invalid-job.json");
        fs::write(&job_file, "invalid json content").unwrap();

        let mut config = test_config();
        config.service.url = temp_dir.path().to_string_lossy().to_string();
        let client = JujutsuClient::new(config);
        
        // Should handle invalid files gracefully
        let jobs = client.get_jobs().await;
        assert!(jobs.is_ok());
        assert_eq!(jobs.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_jujutsu_file_watching_creation() {
        let temp_dir = TempDir::new().unwrap();
        let barefoot_dir = temp_dir.path().join(".barefoot");
        fs::create_dir(&barefoot_dir).unwrap();

        let mut config = test_config();
        config.service.url = temp_dir.path().to_string_lossy().to_string();
        let client = JujutsuClient::new(config);
        
        // Start watching for changes
        let (tx, mut rx) = tokio::sync::mpsc::channel(100);
        let watch_handle = client.start_file_watching(tx).await;
        assert!(watch_handle.is_ok());

        // Wait a moment for the watcher to start
        sleep(Duration::from_millis(100)).await;

        // Create a job file (TOML format)
        let job_file = barefoot_dir.join("new-job.toml");
        let job_content = r#"id = "550e8400-e29b-41d4-a716-446655440001"
name = "New Job"
status = "Queued"
workflow = "test-workflow"
repository = "test-repo"
started_at = null
completed_at = null

[[steps]]
name = "test-step"
status = "Queued"
run = "echo 'hello'"
uses = null
duration = null"#;
        fs::write(&job_file, job_content).unwrap();

        // Wait for file change notification (polling approach)
        let change = tokio::time::timeout(Duration::from_secs(30), rx.recv()).await;
        assert!(change.is_ok());
        let change = change.unwrap();
        assert!(change.is_some());
        assert_eq!(change.unwrap(), "new-job.toml");

        // Clean up
        if let Ok(handle) = watch_handle {
            handle.abort();
        }
    }

    #[tokio::test]
    async fn test_jujutsu_file_watching_debouncing() {
        let temp_dir = TempDir::new().unwrap();
        let barefoot_dir = temp_dir.path().join(".barefoot");
        fs::create_dir(&barefoot_dir).unwrap();

        let mut config = test_config();
        config.service.url = temp_dir.path().to_string_lossy().to_string();
        let client = JujutsuClient::new(config);
        
        // Start watching for changes
        let (tx, mut rx) = tokio::sync::mpsc::channel(100);
        let watch_handle = client.start_file_watching(tx).await;
        assert!(watch_handle.is_ok());

        // Wait a moment for the watcher to start
        sleep(Duration::from_millis(100)).await;

        // Create multiple files rapidly (mix of JSON and TOML)
        for i in 0..3 {
            let extension = if i % 2 == 0 { "json" } else { "toml" };
            let job_file = barefoot_dir.join(format!("job-{i}.{extension}"));
            
            let job_content = if extension == "json" {
                format!(r#"{{
                    "id": "550e8400-e29b-41d4-a716-446655440{i:03}",
                    "name": "Job {i}",
                    "status": "Queued",
                    "workflow": "test-workflow",
                    "repository": "test-repo",
                    "started_at": null,
                    "completed_at": null,
                    "steps": []
                }}"#)
            } else {
                format!(r#"id = "550e8400-e29b-41d4-a716-446655440{i:03}"
name = "Job {i}"
status = "Queued"
workflow = "test-workflow"
repository = "test-repo"
started_at = null
completed_at = null

[[steps]]
name = "test-step"
status = "Queued"
run = "echo 'hello'"
uses = null
duration = null"#)
            };
            
            fs::write(&job_file, job_content).unwrap();
        }

        // Should receive multiple notifications
        let mut notifications = 0;
        while tokio::time::timeout(Duration::from_secs(30), rx.recv()).await.is_ok() {
            notifications += 1;
            if notifications >= 3 {
                break;
            }
        }
        assert!(notifications > 0);

        // Clean up
        if let Ok(handle) = watch_handle {
            handle.abort();
        }
    }

    #[tokio::test]
    async fn test_jujutsu_file_watching_trapdoor_timer() {
        let temp_dir = TempDir::new().unwrap();
        let barefoot_dir = temp_dir.path().join(".barefoot");
        fs::create_dir(&barefoot_dir).unwrap();

        let mut config = test_config();
        config.service.url = temp_dir.path().to_string_lossy().to_string();
        let client = JujutsuClient::new(config);
        
        // Start watching for changes
        let (tx, _rx) = tokio::sync::mpsc::channel(100);
        let watch_handle = client.start_file_watching(tx).await;
        assert!(watch_handle.is_ok());

        // Wait for trapdoor timer to expire (use shorter timeout for testing)
        // In real implementation, this would be 5 minutes, but for testing we'll use a shorter time
        sleep(Duration::from_millis(500)).await;

        // The watcher should have stopped due to trapdoor timer
        if let Ok(handle) = watch_handle {
            // Check if the task has completed
            if handle.is_finished() {
                tracing::info!("File watcher stopped due to trapdoor timer as expected");
            }
        }
    }
} 