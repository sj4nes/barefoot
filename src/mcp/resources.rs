//! MCP resources for barefoot runner
//
// TODO: Add MCP resource for weather dashboard (summarize job health, system health)
// TODO: Add MCP resource for alerts and notifications (failing jobs, stuck jobs, degraded service)
// TODO: Add MCP resource for dependency graph visualization
// TODO: Add MCP resource for sparkline and cycle time analytics (minutes/hours/days, avg/p99/last)

use super::*;
use crate::core::RunnerCore;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::Utc;
use crate::error::Result;

/// Resource provider enum to avoid async trait objects
#[derive(Clone)]
pub enum ResourceProvider {
    Health(HealthResource),
    ActiveJobs(ActiveJobsResource),
    JobHistory(JobHistoryResource),
    Config(ConfigResource),
}

impl ResourceProvider {
    /// Get resource identifier
    pub fn resource_id(&self) -> ResourceId {
        match self {
            ResourceProvider::Health(r) => r.resource_id(),
            ResourceProvider::ActiveJobs(r) => r.resource_id(),
            ResourceProvider::JobHistory(r) => r.resource_id(),
            ResourceProvider::Config(r) => r.resource_id(),
        }
    }
    
    /// Get resource content
    pub async fn get_content(&self) -> Result<ResourceContent> {
        match self {
            ResourceProvider::Health(r) => r.get_content().await,
            ResourceProvider::ActiveJobs(r) => r.get_content().await,
            ResourceProvider::JobHistory(r) => r.get_content().await,
            ResourceProvider::Config(r) => r.get_content().await,
        }
    }
    
    /// Check if resource is available
    pub async fn is_available(&self) -> bool {
        match self {
            ResourceProvider::Health(r) => r.is_available().await,
            ResourceProvider::ActiveJobs(r) => r.is_available().await,
            ResourceProvider::JobHistory(r) => r.is_available().await,
            ResourceProvider::Config(r) => r.is_available().await,
        }
    }
    
    /// Get resource expiration time
    pub fn expires_at(&self) -> Option<chrono::DateTime<Utc>> {
        match self {
            ResourceProvider::Health(r) => r.expires_at(),
            ResourceProvider::ActiveJobs(r) => r.expires_at(),
            ResourceProvider::JobHistory(r) => r.expires_at(),
            ResourceProvider::Config(r) => r.expires_at(),
        }
    }
}

/// Health resource provider
#[derive(Clone)]
pub struct HealthResource {
    runner_core: Arc<RwLock<RunnerCore>>,
    start_time: Option<chrono::DateTime<Utc>>,
    last_error: Arc<RwLock<Option<String>>>,
}

impl HealthResource {
    pub fn new(
        runner_core: Arc<RwLock<RunnerCore>>,
        start_time: Option<chrono::DateTime<Utc>>,
        last_error: Arc<RwLock<Option<String>>>,
    ) -> Self {
        Self {
            runner_core,
            start_time,
            last_error,
        }
    }
    
    pub fn resource_id(&self) -> ResourceId {
        ResourceId {
            uri: "barefoot://runner/health".to_string(),
            mime_type: "application/json".to_string(),
            title: "Runner Health".to_string(),
            description: "Overall runner system health and status".to_string(),
        }
    }
    
    pub async fn get_content(&self) -> Result<ResourceContent> {
        let runner_core = self.runner_core.read().await;
        let status = runner_core.status().await;
        let last_error = self.last_error.read().await.clone();
        
        let content = serde_json::json!({
            "status": status.to_string(),
            "active_jobs": runner_core.current_jobs().await.len(),
            "queue_size": runner_core.queue_size().await,
            "capabilities": runner_core.capabilities(),
            "uptime": self.start_time.map(|t| Utc::now().signed_duration_since(t).num_seconds()),
            "last_error": last_error,
            "can_accept_jobs": runner_core.can_accept_jobs().await,
            "health_score": self.calculate_health_score(&runner_core).await,
        });
        
        Ok(ResourceContent {
            id: self.resource_id(),
            content,
            hash: None,
            expires_at: Some(Utc::now() + chrono::Duration::seconds(30)),
        })
    }
    
    pub async fn is_available(&self) -> bool {
        true // Health resource is always available
    }
    
    pub fn expires_at(&self) -> Option<chrono::DateTime<Utc>> {
        Some(Utc::now() + chrono::Duration::seconds(30))
    }
    
    async fn calculate_health_score(&self, runner_core: &RunnerCore) -> f64 {
        let mut score = 100.0;
        
        // Deduct points for errors
        if let Some(_error) = &*self.last_error.read().await {
            score -= 20.0;
        }
        
        // Deduct points for high queue size
        let queue_size = runner_core.queue_size().await;
        if queue_size > 10 {
            score -= (queue_size as f64 - 10.0) * 2.0;
        }
        
        // Deduct points for many active jobs (potential overload)
        let active_jobs = runner_core.current_jobs().await.len();
        if active_jobs > 5 {
            score -= (active_jobs as f64 - 5.0) * 3.0;
        }
        
        // Deduct points if runner can't accept jobs
        if !runner_core.can_accept_jobs().await {
            score -= 30.0;
        }
        
        score.max(0.0)
    }
}

/// Active jobs resource provider
#[derive(Clone)]
pub struct ActiveJobsResource {
    runner_core: Arc<RwLock<RunnerCore>>,
}

impl ActiveJobsResource {
    pub fn new(runner_core: Arc<RwLock<RunnerCore>>) -> Self {
        Self { runner_core }
    }
    
    pub fn resource_id(&self) -> ResourceId {
        ResourceId {
            uri: "barefoot://jobs/active".to_string(),
            mime_type: "application/json".to_string(),
            title: "Active Jobs".to_string(),
            description: "Currently running and queued jobs".to_string(),
        }
    }
    
    pub async fn get_content(&self) -> Result<ResourceContent> {
        let runner_core = self.runner_core.read().await;
        let active_jobs = runner_core.current_jobs().await;
        let queue = runner_core.job_queue().await;
        
        let content = serde_json::json!({
            "active_jobs": active_jobs,
            "queued_jobs": queue,
            "total_count": active_jobs.len() + queue.len(),
            "summary": {
                "running": active_jobs.len(),
                "queued": queue.len(),
                "total": active_jobs.len() + queue.len(),
            },
            "last_updated": Utc::now().to_rfc3339(),
        });
        
        Ok(ResourceContent {
            id: self.resource_id(),
            content,
            hash: None,
            expires_at: Some(Utc::now() + chrono::Duration::seconds(10)),
        })
    }
    
    pub async fn is_available(&self) -> bool {
        true // Active jobs resource is always available
    }
    
    pub fn expires_at(&self) -> Option<chrono::DateTime<Utc>> {
        Some(Utc::now() + chrono::Duration::seconds(10))
    }
}

/// Job history resource provider
#[derive(Clone)]
pub struct JobHistoryResource {
    runner_core: Arc<RwLock<RunnerCore>>,
}

impl JobHistoryResource {
    pub fn new(runner_core: Arc<RwLock<RunnerCore>>) -> Self {
        Self { runner_core }
    }
    
    pub fn resource_id(&self) -> ResourceId {
        ResourceId {
            uri: "barefoot://jobs/history".to_string(),
            mime_type: "application/json".to_string(),
            title: "Job History".to_string(),
            description: "Historical job execution data".to_string(),
        }
    }
    
    pub async fn get_content(&self) -> Result<ResourceContent> {
        let runner_core = self.runner_core.read().await;
        let job_runs = runner_core.get_all_job_runs().await;
        
        let total_jobs = job_runs.len();
        let successful_jobs = job_runs.iter().filter(|run| run.status == crate::types::JobStatus::Completed).count();
        let success_rate = if total_jobs > 0 {
            (successful_jobs as f64 / total_jobs as f64) * 100.0
        } else {
            0.0
        };
        
        let average_duration = if total_jobs > 0 {
            let total_duration: u128 = job_runs.iter().map(|run| run.duration_ms).sum();
            total_duration / total_jobs as u128
        } else {
            0
        };
        
        let content = serde_json::json!({
            "recent_jobs": job_runs.iter().take(10).map(|run| {
                serde_json::json!({
                    "id": run.job_id,
                    "name": run.job_name,
                    "status": run.status,
                    "started_at": run.started_at,
                    "completed_at": run.completed_at,
                    "duration_ms": run.duration_ms,
                })
            }).collect::<Vec<_>>(),
            "success_rate": success_rate,
            "average_duration_ms": average_duration,
            "total_jobs": total_jobs,
            "last_updated": Utc::now().to_rfc3339(),
        });
        
        Ok(ResourceContent {
            id: self.resource_id(),
            content,
            hash: None,
            expires_at: Some(Utc::now() + chrono::Duration::seconds(60)),
        })
    }
    
    pub async fn is_available(&self) -> bool {
        true // Job history resource is always available
    }
    
    pub fn expires_at(&self) -> Option<chrono::DateTime<Utc>> {
        Some(Utc::now() + chrono::Duration::seconds(60))
    }
}

/// Configuration resource provider
#[derive(Clone)]
pub struct ConfigResource {
    config: crate::config::BarefootConfig,
}

impl ConfigResource {
    pub fn new(config: crate::config::BarefootConfig) -> Self {
        Self { config }
    }
    
    pub fn resource_id(&self) -> ResourceId {
        ResourceId {
            uri: "barefoot://config/runner".to_string(),
            mime_type: "application/json".to_string(),
            title: "Runner Configuration".to_string(),
            description: "Current runner configuration settings".to_string(),
        }
    }
    
    pub async fn get_content(&self) -> Result<ResourceContent> {
        let content = serde_json::json!({
            "service_type": self.config.service.service_type,
            "service_url": self.config.service.url,
            "runner_name": self.config.runner.name,
            "max_concurrent_jobs": self.config.runner.max_concurrent_jobs,
            "log_level": self.config.logging.level,
            "differential_logging": self.config.logging.differential_logging,
        });
        
        Ok(ResourceContent {
            id: self.resource_id(),
            content,
            hash: None,
            expires_at: Some(Utc::now() + chrono::Duration::seconds(300)),
        })
    }
    
    pub async fn is_available(&self) -> bool {
        true // Config resource is always available
    }
    
    pub fn expires_at(&self) -> Option<chrono::DateTime<Utc>> {
        Some(Utc::now() + chrono::Duration::seconds(300))
    }
}

/// Resource manager using enum-based providers
#[derive(Clone)]
pub struct ResourceManager {
    resources: std::collections::HashMap<String, ResourceProvider>,
}

impl ResourceManager {
    pub fn new() -> Self {
        Self {
            resources: std::collections::HashMap::new(),
        }
    }
    
    pub fn register_resource(&mut self, resource: ResourceProvider) {
        let uri = resource.resource_id().uri.clone();
        self.resources.insert(uri, resource);
    }
    
    pub async fn get_resource(&self, uri: &str) -> Result<ResourceContent> {
        if let Some(resource) = self.resources.get(uri) {
            resource.get_content().await
        } else {
            Err(BarefootError::Mcp(format!("Resource not found: {}", uri)))
        }
    }
    
    pub fn list_resources(&self) -> Vec<ResourceId> {
        self.resources
            .values()
            .map(|r| r.resource_id())
            .collect()
    }
    
    pub fn has_resource(&self, uri: &str) -> bool {
        self.resources.contains_key(uri)
    }
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::BarefootConfig;
    
    #[tokio::test]
    async fn test_health_resource() {
        let runner_core = Arc::new(RwLock::new(RunnerCore::new(BarefootConfig::default())));
        let last_error = Arc::new(RwLock::new(None));
        let health_resource = HealthResource::new(runner_core, Some(Utc::now()), last_error);
        
        let content = health_resource.get_content().await.unwrap();
        assert_eq!(content.id.uri, "barefoot://runner/health");
        assert!(content.content.is_object());
    }
    
    #[tokio::test]
    async fn test_active_jobs_resource() {
        let runner_core = Arc::new(RwLock::new(RunnerCore::new(BarefootConfig::default())));
        let active_jobs_resource = ActiveJobsResource::new(runner_core);
        
        let content = active_jobs_resource.get_content().await.unwrap();
        assert_eq!(content.id.uri, "barefoot://jobs/active");
        assert!(content.content.is_object());
    }
    
    #[tokio::test]
    async fn test_config_resource() {
        let config = BarefootConfig::default();
        let config_resource = ConfigResource::new(config);
        
        let content = config_resource.get_content().await.unwrap();
        assert_eq!(content.id.uri, "barefoot://config/runner");
        assert!(content.content.is_object());
    }
    
    #[tokio::test]
    async fn test_resource_manager() {
        let mut manager = ResourceManager::new();
        let config = BarefootConfig::default();
        let config_resource = ResourceProvider::Config(ConfigResource::new(config));
        
        manager.register_resource(config_resource);
        assert!(manager.has_resource("barefoot://config/runner"));
        
        let content = manager.get_resource("barefoot://config/runner").await.unwrap();
        assert_eq!(content.id.uri, "barefoot://config/runner");
    }
} 