use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Runner status
///
/// # Examples
/// ```rust
/// use barefoot::types::RunnerStatus;
/// let status = RunnerStatus::Idle;
/// assert_eq!(status, RunnerStatus::Idle);
/// ```
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
pub enum RunnerStatus {
    #[default]
    Idle,
    Busy,
    Offline,
    Maintenance,
}

/// Runner capabilities
///
/// # Examples
/// ```rust
/// use barefoot::types::RunnerCapabilities;
/// use std::collections::HashMap;
/// let caps = RunnerCapabilities {
///     os: "linux".to_string(),
///     architecture: "x86_64".to_string(),
///     labels: vec!["self-hosted".to_string()],
///     features: HashMap::new(),
/// };
/// assert_eq!(caps.os, "linux");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RunnerCapabilities {
    pub os: String,
    pub architecture: String,
    pub labels: Vec<String>,
    pub features: HashMap<String, String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct DiskUsageReport {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub details: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub duration_ms: Option<u128>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub work_dir: String,
    pub env: std::collections::HashMap<String, String>,
    // Add more as needed
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerCleanupConfig {
    pub enabled: bool,
    pub interval_minutes: u64,
    pub max_usage_bytes: u64,
}

/// Runner configuration
///
/// # Examples
/// ```rust
/// use barefoot::types::{RunnerConfig, RunnerCapabilities};
/// use std::collections::HashMap;
/// let config = RunnerConfig {
///     name: "runner".to_string(),
///     url: "http://localhost".to_string(),
///     token: "token".to_string(),
///     labels: vec!["self-hosted".to_string()],
///     capabilities: RunnerCapabilities {
///         os: "linux".to_string(),
///         architecture: "x86_64".to_string(),
///         labels: vec![],
///         features: HashMap::new(),
///     },
///     max_concurrent_jobs: 1,
///     work_dir: ".".to_string(),
/// };
/// assert_eq!(config.name, "runner");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunnerConfig {
    pub name: String,
    pub url: String,
    pub token: String,
    pub labels: Vec<String>,
    pub capabilities: RunnerCapabilities,
    pub max_concurrent_jobs: usize,
    pub work_dir: String,
    pub container_backend: String, // "docker", "podman", "nix", "native"
    pub container_backend_opts: Option<std::collections::HashMap<String, String>>,
    pub container_cleanup: ContainerCleanupConfig,
}

/// Job status
///
/// # Examples
/// ```rust
/// use barefoot::types::JobStatus;
/// let status = JobStatus::Queued;
/// assert_eq!(status, JobStatus::Queued);
/// ```
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
pub enum JobStatus {
    #[default]
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
    Skipped,
}

/// Job information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Job {
    pub id: Uuid,
    pub name: String,
    pub status: JobStatus,
    pub workflow: String,
    pub repository: String,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub steps: Vec<JobStep>,
}

/// Job step
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JobStep {
    pub name: String,
    pub status: JobStatus,
    pub output: Option<String>,
    pub error: Option<String>,
    pub duration: Option<chrono::Duration>,
}

/// Workflow step
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkflowStep {
    pub name: String,
    pub run: Option<String>,
    pub uses: Option<String>,
    pub with: HashMap<String, serde_json::Value>,
    pub env: HashMap<String, String>,
    pub shell: Option<String>,
    pub working_directory: Option<String>,
}

/// Workflow job
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkflowJob {
    pub name: String,
    pub runs_on: String,
    pub steps: Vec<WorkflowStep>,
    pub env: HashMap<String, String>,
    pub timeout_minutes: Option<u32>,
    pub strategy: Option<JobStrategy>,
}

/// Job strategy for matrix builds
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JobStrategy {
    pub matrix: HashMap<String, Vec<serde_json::Value>>,
    pub fail_fast: Option<bool>,
    pub max_parallel: Option<usize>,
}

/// Workflow
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Workflow {
    pub name: String,
    pub on: WorkflowTriggers,
    pub jobs: HashMap<String, WorkflowJob>,
    pub env: HashMap<String, String>,
}

/// Workflow triggers
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkflowTriggers {
    pub push: Option<PushTrigger>,
    pub pull_request: Option<PullRequestTrigger>,
    pub schedule: Option<Vec<ScheduleTrigger>>,
    pub workflow_dispatch: Option<bool>,
}

/// Push trigger
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PushTrigger {
    pub branches: Option<Vec<String>>,
    pub paths: Option<Vec<String>>,
}

/// Pull request trigger
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PullRequestTrigger {
    pub branches: Option<Vec<String>>,
    pub paths: Option<Vec<String>>,
}

/// Schedule trigger
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScheduleTrigger {
    pub cron: String,
}

/// Service type for different Git hosting platforms
///
/// # Examples
/// ```rust
/// use barefoot::types::ServiceType;
/// let t = ServiceType::GitHub;
/// assert_eq!(t, ServiceType::GitHub);
/// ```
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
pub enum ServiceType {
    #[default]
    GitHub,
    GitLab,
    Gitea,
    Jujutsu,
    Custom,
}

/// Service configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServiceConfig {
    pub service_type: ServiceType,
    pub url: String,
    pub token: String,
    pub api_version: Option<String>,
    pub custom_headers: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    use uuid::Uuid;
    use chrono::Utc;

    #[test]
    fn test_runner_status_serde() {
        let status = RunnerStatus::Busy;
        let s = serde_json::to_string(&status).unwrap();
        let d: RunnerStatus = serde_json::from_str(&s).unwrap();
        assert_eq!(status, d);
    }

    #[test]
    fn test_job_status_serde() {
        let status = JobStatus::Completed;
        let s = serde_json::to_string(&status).unwrap();
        let d: JobStatus = serde_json::from_str(&s).unwrap();
        assert_eq!(status, d);
    }

    #[test]
    fn test_service_type_serde() {
        let t = ServiceType::Jujutsu;
        let s = serde_json::to_string(&t).unwrap();
        let d: ServiceType = serde_json::from_str(&s).unwrap();
        assert_eq!(t, d);
    }

    #[test]
    fn test_job_serialization() {
        let job = Job {
            id: Uuid::new_v4(),
            name: "test-job".to_string(),
            status: JobStatus::Queued,
            workflow: "wf".to_string(),
            repository: "repo".to_string(),
            started_at: Some(Utc::now()),
            completed_at: None,
            steps: vec![],
        };
        let s = serde_json::to_string(&job).unwrap();
        let d: Job = serde_json::from_str(&s).unwrap();
        assert_eq!(job.name, d.name);
        assert_eq!(job.status, d.status);
    }
} 