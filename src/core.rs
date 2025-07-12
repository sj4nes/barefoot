use crate::{
    config::BarefootConfig,
    error::Result,
    types::{Job, JobStatus, RunnerStatus},
};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Core runner state
#[derive(Debug)]
pub struct RunnerCore {
    config: Arc<BarefootConfig>,
    status: Arc<RwLock<RunnerStatus>>,
    current_jobs: Arc<RwLock<Vec<Job>>>,
    job_queue: Arc<RwLock<Vec<Job>>>,
}

impl RunnerCore {
    /// Create a new runner core
    pub fn new(config: BarefootConfig) -> Self {
        Self {
            config: Arc::new(config),
            status: Arc::new(RwLock::new(RunnerStatus::Idle)),
            current_jobs: Arc::new(RwLock::new(Vec::new())),
            job_queue: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Get the current status
    pub async fn status(&self) -> RunnerStatus {
        self.status.read().await.clone()
    }

    /// Set the status
    pub async fn set_status(&self, status: RunnerStatus) {
        let mut status_guard = self.status.write().await;
        *status_guard = status;
        tracing::info!("Runner status changed to: {:?}", status);
    }

    /// Get current jobs
    pub async fn current_jobs(&self) -> Vec<Job> {
        self.current_jobs.read().await.clone()
    }

    /// Add a job to the queue
    pub async fn queue_job(&self, job: Job) -> Result<()> {
        let mut queue = self.job_queue.write().await;
        queue.push(job);
        tracing::info!("Job queued, queue size: {}", queue.len());
        Ok(())
    }

    /// Get the next job from the queue
    pub async fn next_job(&self) -> Option<Job> {
        let mut queue = self.job_queue.write().await;
        queue.pop()
    }

    /// Start a job
    pub async fn start_job(&self, mut job: Job) -> Result<()> {
        let mut current_jobs = self.current_jobs.write().await;
        
        // Check if we can accept more jobs
        if current_jobs.len() >= self.config.runner.max_concurrent_jobs {
            return Err(crate::error::BarefootError::TooManyJobs);
        }
        
        // Update job status
        job.status = JobStatus::Running;
        job.started_at = Some(chrono::Utc::now());
        
        // Add to current jobs
        current_jobs.push(job.clone());
        
        tracing::info!("Job started: {}", job.id);
        
        Ok(())
    }

    /// Complete a job
    pub async fn complete_job(&self, job_id: Uuid, status: JobStatus) -> Result<()> {
        let mut current_jobs = self.current_jobs.write().await;
        
        if let Some(job) = current_jobs.iter_mut().find(|j| j.id == job_id) {
            job.status = status;
            tracing::info!("Job completed: {} with status: {:?}", job_id, status);
        }

        // Remove completed jobs
        current_jobs.retain(|j| j.status == JobStatus::Running);

        // Update status if no more running jobs
        if current_jobs.is_empty() {
            self.set_status(RunnerStatus::Idle).await;
        }

        Ok(())
    }

    /// Get configuration
    pub fn config(&self) -> &BarefootConfig {
        &self.config
    }

    /// Get runner capabilities
    pub fn capabilities(&self) -> &crate::types::RunnerCapabilities {
        &self.config.runner.capabilities
    }

    /// Check if runner can accept more jobs
    pub async fn can_accept_jobs(&self) -> bool {
        let current_jobs = self.current_jobs.read().await;
        current_jobs.len() < self.config.runner.max_concurrent_jobs
    }

    /// Get queue size
    pub async fn queue_size(&self) -> usize {
        self.job_queue.read().await.len()
    }

    /// Get running jobs count
    pub async fn running_jobs_count(&self) -> usize {
        self.current_jobs.read().await.len()
    }
} 