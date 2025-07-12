use crate::{
    config::BarefootConfig,
    error::Result,
    types::{Job, JobStatus, RunnerStatus},
};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use std::collections::VecDeque;
use chrono::{DateTime, Utc};

/// Job run record for differential logging
#[derive(Debug, Clone)]
pub struct JobRunRecord {
    pub job_id: Uuid,
    pub job_name: String,
    pub status: JobStatus,
    pub logs: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub duration_ms: u128,
}

/// Differential logging service
#[derive(Debug)]
pub struct DifferentialLogger {
    job_runs: Arc<RwLock<VecDeque<JobRunRecord>>>,
    max_runs: usize,
}

impl DifferentialLogger {
    pub fn new(max_runs: usize) -> Self {
        Self {
            job_runs: Arc::new(RwLock::new(VecDeque::new())),
            max_runs,
        }
    }

    /// Store a job run record
    pub async fn store_job_run(&self, record: JobRunRecord) {
        let mut runs = self.job_runs.write().await;
        
        // Add new record
        runs.push_back(record);
        
        // Keep only the last max_runs records
        while runs.len() > self.max_runs {
            runs.pop_front();
        }
        
        tracing::debug!("Stored job run, total records: {}", runs.len());
    }

    /// Get differential logs for a job (comparing with previous runs)
    pub async fn get_differential_logs(&self, job_name: &str) -> String {
        let runs = self.job_runs.read().await;
        
        // Find the most recent run of this job type
        let current_run = runs.iter().rev().find(|r| r.job_name == job_name);
        
        if let Some(current) = current_run {
            // Find the previous run of this job type
            let mut iter = runs.iter().rev();
            iter.next(); // Skip the current run
            let previous_run = iter.find(|r| r.job_name == job_name);
            
            if let Some(previous) = previous_run {
                // Generate differential log
                self.generate_differential_log(current, previous)
            } else {
                // First run of this job type
                format!("=== First run of job '{}' ===\n{}", job_name, current.logs)
            }
        } else {
            // No runs found
            format!("=== No previous runs found for job '{}' ===", job_name)
        }
    }

    /// Generate differential log between two runs
    fn generate_differential_log(&self, current: &JobRunRecord, previous: &JobRunRecord) -> String {
        let mut diff_log = String::new();
        
        diff_log.push_str(&format!("=== Differential Log for '{}' ===\n", current.job_name));
        diff_log.push_str(&format!("Current Run: {} ({}ms)\n", current.completed_at, current.duration_ms));
        diff_log.push_str(&format!("Previous Run: {} ({}ms)\n", previous.completed_at, previous.duration_ms));
        diff_log.push_str(&format!("Status Change: {:?} -> {:?}\n", previous.status, current.status));
        
        // Compare logs (simple diff for now)
        if current.logs != previous.logs {
            diff_log.push_str("\n=== Log Changes ===\n");
            
            // Simple line-by-line comparison
            let current_lines: Vec<&str> = current.logs.lines().collect();
            let previous_lines: Vec<&str> = previous.logs.lines().collect();
            
            let max_lines = current_lines.len().max(previous_lines.len());
            
            for i in 0..max_lines {
                let current_line = current_lines.get(i).unwrap_or(&"");
                let previous_line = previous_lines.get(i).unwrap_or(&"");
                
                if current_line != previous_line {
                    diff_log.push_str(&format!("Line {}: -{}\n", i + 1, previous_line));
                    diff_log.push_str(&format!("Line {}: +{}\n", i + 1, current_line));
                }
            }
        } else {
            diff_log.push_str("\n=== No log changes detected ===\n");
        }
        
        diff_log
    }

    /// Get all stored job runs
    pub async fn get_all_runs(&self) -> Vec<JobRunRecord> {
        let runs = self.job_runs.read().await;
        runs.iter().cloned().collect()
    }

    /// Clear all stored runs
    pub async fn clear_runs(&self) {
        let mut runs = self.job_runs.write().await;
        runs.clear();
        tracing::info!("Cleared all differential logging records");
    }
}

/// Core runner state
#[derive(Debug)]
pub struct RunnerCore {
    config: Arc<BarefootConfig>,
    status: Arc<RwLock<RunnerStatus>>,
    current_jobs: Arc<RwLock<Vec<Job>>>,
    job_queue: Arc<RwLock<Vec<Job>>>,
    differential_logger: Arc<DifferentialLogger>,
}

impl RunnerCore {
    /// Create a new runner core
    pub fn new(config: BarefootConfig) -> Self {
        let max_runs = config.logging.differential_logging.max_job_runs;
        let differential_logger = Arc::new(DifferentialLogger::new(max_runs));
        
        Self {
            config: Arc::new(config),
            status: Arc::new(RwLock::new(RunnerStatus::Idle)),
            current_jobs: Arc::new(RwLock::new(Vec::new())),
            job_queue: Arc::new(RwLock::new(Vec::new())),
            differential_logger,
        }
    }

    /// Get the current status
    pub async fn status(&self) -> RunnerStatus {
        *self.status.read().await
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

    /// Store a job run for differential logging
    pub async fn store_job_run(&self, job: &Job, logs: &str, duration_ms: u128) {
        if self.config.logging.differential_logging.enabled {
            let record = JobRunRecord {
                job_id: job.id,
                job_name: job.name.clone(),
                status: job.status,
                logs: logs.to_string(),
                started_at: job.started_at.unwrap_or_else(Utc::now),
                completed_at: job.completed_at.unwrap_or_else(Utc::now),
                duration_ms,
            };
            
            self.differential_logger.store_job_run(record).await;
        }
    }

    /// Get differential logs for a job
    pub async fn get_differential_logs(&self, job_name: &str) -> String {
        if self.config.logging.differential_logging.enabled {
            self.differential_logger.get_differential_logs(job_name).await
        } else {
            "Differential logging is disabled".to_string()
        }
    }

    /// Get all stored job runs
    pub async fn get_all_job_runs(&self) -> Vec<JobRunRecord> {
        if self.config.logging.differential_logging.enabled {
            self.differential_logger.get_all_runs().await
        } else {
            Vec::new()
        }
    }

    /// Clear all stored job runs
    pub async fn clear_job_runs(&self) {
        if self.config.logging.differential_logging.enabled {
            self.differential_logger.clear_runs().await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::BarefootConfig;

    #[tokio::test]
    async fn test_differential_logger_creation() {
        let logger = DifferentialLogger::new(5);
        let runs = logger.get_all_runs().await;
        assert_eq!(runs.len(), 0);
    }

    #[tokio::test]
    async fn test_differential_logger_store_and_retrieve() {
        let logger = DifferentialLogger::new(3);
        
        let record1 = JobRunRecord {
            job_id: Uuid::new_v4(),
            job_name: "test-job".to_string(),
            status: JobStatus::Completed,
            logs: "test logs 1".to_string(),
            started_at: Utc::now(),
            completed_at: Utc::now(),
            duration_ms: 1000,
        };
        
        logger.store_job_run(record1.clone()).await;
        
        let runs = logger.get_all_runs().await;
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].job_name, "test-job");
    }

    #[tokio::test]
    async fn test_differential_logger_max_runs_limit() {
        let logger = DifferentialLogger::new(2);
        
        for i in 0..4 {
            let record = JobRunRecord {
                job_id: Uuid::new_v4(),
                job_name: format!("test-job-{}", i),
                status: JobStatus::Completed,
                logs: format!("test logs {}", i),
                started_at: Utc::now(),
                completed_at: Utc::now(),
                duration_ms: 1000,
            };
            
            logger.store_job_run(record).await;
        }
        
        let runs = logger.get_all_runs().await;
        assert_eq!(runs.len(), 2); // Should only keep last 2
        assert_eq!(runs[0].job_name, "test-job-2");
        assert_eq!(runs[1].job_name, "test-job-3");
    }

    #[tokio::test]
    async fn test_differential_logger_get_logs() {
        let logger = DifferentialLogger::new(5);
        
        // First run
        let record1 = JobRunRecord {
            job_id: Uuid::new_v4(),
            job_name: "test-job".to_string(),
            status: JobStatus::Completed,
            logs: "first run logs".to_string(),
            started_at: Utc::now(),
            completed_at: Utc::now(),
            duration_ms: 1000,
        };
        
        logger.store_job_run(record1).await;
        
        // Second run
        let record2 = JobRunRecord {
            job_id: Uuid::new_v4(),
            job_name: "test-job".to_string(),
            status: JobStatus::Failed,
            logs: "second run logs".to_string(),
            started_at: Utc::now(),
            completed_at: Utc::now(),
            duration_ms: 2000,
        };
        
        logger.store_job_run(record2).await;
        
        let diff_logs = logger.get_differential_logs("test-job").await;
        assert!(diff_logs.contains("Differential Log for 'test-job'"));
        assert!(diff_logs.contains("Status Change: Completed -> Failed"));
    }

    #[tokio::test]
    async fn test_runner_core_differential_logging() {
        let config = BarefootConfig::default();
        let core = RunnerCore::new(config);
        
        // Test that differential logging is enabled by default
        let runs = core.get_all_job_runs().await;
        assert_eq!(runs.len(), 0);
        
        // Test clearing runs
        core.clear_job_runs().await;
        let runs = core.get_all_job_runs().await;
        assert_eq!(runs.len(), 0);
    }
} 