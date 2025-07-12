use crate::{
    core::RunnerCore,
    error::Result,
    types::{Job, JobStatus, Workflow, WorkflowJob, WorkflowStep},
};
use std::process::Stdio;
use tokio::process::Command;
use uuid::Uuid;
use chrono::Utc;

/// Job executor
pub struct JobExecutor {
    core: RunnerCore,
}

impl JobExecutor {
    /// Create a new job executor
    pub fn new(core: RunnerCore) -> Self {
        Self { core }
    }

    /// Execute a job
    pub async fn execute_job(&self, job: Job) -> Result<JobStatus> {
        tracing::info!("Starting job execution: {}", job.id);
        
        // Start the job
        self.core.start_job(job.clone()).await?;

        let start_time = Utc::now();
        let mut final_status = JobStatus::Completed;

        // Execute each step in the job
        for step in &job.steps {
            match self.execute_step(step).await {
                Ok(_) => {
                    tracing::info!("Step completed: {}", step.name);
                }
                Err(e) => {
                    tracing::error!("Step failed: {} - {}", step.name, e);
                    final_status = JobStatus::Failed;
                    break;
                }
            }
        }

        // Complete the job
        self.core.complete_job(job.id, final_status).await?;

        let duration = Utc::now() - start_time;
        tracing::info!(
            "Job completed: {} with status: {:?} (duration: {:?})",
            job.id,
            final_status,
            duration
        );

        Ok(final_status)
    }

    /// Execute a single step
    async fn execute_step(&self, step: &crate::types::JobStep) -> Result<()> {
        tracing::info!("Executing step: {}", step.name);

        // For now, we'll implement basic shell command execution
        // In a full implementation, this would handle different step types
        // like `uses`, `run`, etc.
        
        if let Some(command) = &step.output {
            self.execute_shell_command(command).await?;
        }

        Ok(())
    }

    /// Execute a shell command
    async fn execute_shell_command(&self, command: &str) -> Result<()> {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| {
            if cfg!(windows) {
                "cmd".to_string()
            } else {
                "/bin/bash".to_string()
            }
        });

        let args = if cfg!(windows) {
            vec!["/C", command]
        } else {
            vec!["-c", command]
        };

        let output = Command::new(&shell)
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| crate::error::BarefootError::Process(e.to_string()))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(crate::error::BarefootError::Process(error.to_string()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        tracing::debug!("Command output: {}", stdout);

        Ok(())
    }

    /// Parse workflow from YAML
    pub fn parse_workflow(&self, yaml_content: &str) -> Result<Workflow> {
        serde_yaml::from_str(yaml_content)
            .map_err(|e| crate::error::BarefootError::Yaml(e))
    }

    /// Execute a workflow
    pub async fn execute_workflow(&self, workflow: Workflow) -> Result<Vec<Job>> {
        let mut jobs = Vec::new();

        for (job_name, job_config) in workflow.jobs {
            let job = self.create_job_from_workflow_job(&job_name, &job_config, &workflow.name).await?;
            jobs.push(job);
        }

        Ok(jobs)
    }

    /// Create a job from a workflow job configuration
    async fn create_job_from_workflow_job(
        &self,
        job_name: &str,
        job_config: &WorkflowJob,
        workflow_name: &str,
    ) -> Result<Job> {
        let mut steps = Vec::new();

        for step_config in &job_config.steps {
            let step = crate::types::JobStep {
                name: step_config.name.clone(),
                status: JobStatus::Queued,
                output: step_config.run.clone(),
                error: None,
                duration: None,
            };
            steps.push(step);
        }

        let job = Job {
            id: Uuid::new_v4(),
            name: job_name.to_string(),
            status: JobStatus::Queued,
            workflow: workflow_name.to_string(),
            repository: "unknown".to_string(), // This would come from the context
            started_at: None,
            completed_at: None,
            steps,
        };

        Ok(job)
    }
}

/// Workflow parser
pub struct WorkflowParser;

impl WorkflowParser {
    /// Parse workflow from file
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Workflow> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| crate::error::BarefootError::Io(e))?;
        
        serde_yaml::from_str(&content)
            .map_err(|e| crate::error::BarefootError::Yaml(e))
    }

    /// Parse workflow from string
    pub fn from_string(content: &str) -> Result<Workflow> {
        serde_yaml::from_str(content)
            .map_err(|e| crate::error::BarefootError::Yaml(e))
    }

    /// Validate workflow
    pub fn validate(workflow: &Workflow) -> Result<()> {
        if workflow.jobs.is_empty() {
            return Err(crate::error::BarefootError::Workflow(
                "Workflow must contain at least one job".to_string(),
            ));
        }

        for (job_name, job) in &workflow.jobs {
            if job.steps.is_empty() {
                return Err(crate::error::BarefootError::Workflow(
                    format!("Job '{}' must contain at least one step", job_name),
                ));
            }

            for step in &job.steps {
                if step.run.is_none() && step.uses.is_none() {
                    return Err(crate::error::BarefootError::Workflow(
                        format!("Step '{}' must have either 'run' or 'uses'", step.name),
                    ));
                }
            }
        }

        Ok(())
    }
} 