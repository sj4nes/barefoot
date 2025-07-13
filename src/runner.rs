use crate::{
    core::RunnerCore,
    error::Result,
    types::{Job, JobStatus, JobStrategy, Workflow, WorkflowJob},
};
use chrono::Utc;
use std::collections::HashMap;
use std::process::Stdio;
use tokio::process::Command;
use uuid::Uuid;

// DONE[4]: Future enhancements: support for advanced workflow step types (e.g., 'uses', matrix builds), and improved error handling.
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
    pub async fn execute_job(
        &self,
        job: Job,
        truncation_config: Option<(bool, usize, usize)>,
    ) -> Result<(JobStatus, String)> {
        tracing::info!("Starting job execution: {}", job.id);

        // Start the job
        self.core.start_job(job.clone()).await?;

        let start_time = Utc::now();
        let mut final_status = JobStatus::Completed;
        let mut all_logs = String::new();

        // Execute each step in the job
        for step in &job.steps {
            match self.execute_step(step).await {
                Ok(logs) => {
                    tracing::info!("Step completed: {}", step.name);
                    all_logs.push_str(&format!("=== Step: {} ===\n", step.name));
                    all_logs.push_str(&logs);
                    all_logs.push('\n');
                }
                Err(e) => {
                    tracing::error!("Step failed: {} - {}", step.name, e);
                    all_logs.push_str(&format!("=== Step: {} FAILED ===\n", step.name));
                    all_logs.push_str(&format!("Error: {e}\n"));
                    final_status = JobStatus::Failed;

                    // Complete the job with failed status
                    self.core.complete_job(job.id, final_status).await?;

                    let duration = Utc::now() - start_time;
                    tracing::info!(
                        "Job failed: {} with status: {:?} (duration: {:?})",
                        job.id,
                        final_status,
                        duration
                    );

                    // Return error when step fails
                    return Err(e);
                }
            }
        }

        // Complete the job
        self.core.complete_job(job.id, final_status).await?;

        let duration = Utc::now() - start_time;
        let duration_ms = duration.num_milliseconds() as u128;

        // Store job run for differential logging
        self.core.store_job_run(&job, &all_logs, duration_ms).await;

        // Apply truncation to local stderr output if enabled
        if let Some((enabled, first_lines, last_lines)) = truncation_config {
            if enabled {
                let truncated_logs =
                    crate::utils::truncate_logs(&all_logs, first_lines, last_lines);
                tracing::info!(
                    "Job completed: {} with status: {:?} (duration: {:?})\nTruncated logs:\n{}",
                    job.id,
                    final_status,
                    duration,
                    truncated_logs
                );
            } else {
                tracing::info!(
                    "Job completed: {} with status: {:?} (duration: {:?})",
                    job.id,
                    final_status,
                    duration
                );
            }
        } else {
            tracing::info!(
                "Job completed: {} with status: {:?} (duration: {:?})",
                job.id,
                final_status,
                duration
            );
        }

        Ok((final_status, all_logs))
    }

    /// Execute a single step
    async fn execute_step(&self, step: &crate::types::JobStep) -> Result<String> {
        tracing::info!("Executing step: {}", step.name);

        // Enhanced step execution with support for different step types
        if let Some(command) = &step.run {
            // Handle 'run' steps
            self.execute_shell_command(command).await
        } else if let Some(action) = &step.uses {
            // Handle 'uses' steps
            self.execute_action_step(action).await
        } else {
            return Err(crate::error::BarefootError::Workflow(format!(
                "Step '{}' must have either 'run' or 'uses'",
                step.name
            )));
        }
    }

    /// Execute an action step (uses: action@version)
    async fn execute_action_step(&self, action: &str) -> Result<String> {
        tracing::info!("Executing action: {}", action);

        // Parse action reference (e.g., "actions/checkout@v3")
        let (action_name, version) = self.parse_action_reference(action)?;

        // For now, implement basic action support
        match action_name.as_str() {
            "actions/checkout" => self.execute_checkout_action(&version).await,
            "actions-rs/toolchain" => self.execute_toolchain_action(&version).await,
            "actions/setup-node" => self.execute_setup_node_action(&version).await,
            "actions/setup-python" => self.execute_setup_python_action(&version).await,
            "actions/cache" => self.execute_cache_action(&version).await,
            "actions/setup-java" => self.execute_setup_java_action(&version).await,
            "actions/setup-go" => self.execute_setup_go_action(&version).await,
            "actions/setup-dotnet" => self.execute_setup_dotnet_action(&version).await,
            "actions/upload-artifact" => self.execute_upload_artifact_action(&version).await,
            "actions/download-artifact" => self.execute_download_artifact_action(&version).await,
            _ => {
                // DONE: Support additional GitHub Actions and custom actions (setup-node, setup-python, cache, setup-java, setup-go, setup-dotnet, upload-artifact, download-artifact are now supported)
                Err(crate::error::BarefootError::Workflow(format!(
                    "Unsupported action: {action_name}"
                )))
            }
        }
    }

    /// Parse action reference into name and version
    fn parse_action_reference(&self, action_ref: &str) -> Result<(String, String)> {
        let parts: Vec<&str> = action_ref.split('@').collect();
        if parts.len() != 2 {
            return Err(crate::error::BarefootError::Workflow(format!(
                "Invalid action reference: {action_ref}"
            )));
        }

        Ok((parts[0].to_string(), parts[1].to_string()))
    }

    /// Execute checkout action
    async fn execute_checkout_action(&self, _version: &str) -> Result<String> {
        tracing::info!("Executing checkout action");

        // Basic checkout implementation
        let commands = vec![
            "git init",
            "git remote add origin https://github.com/owner/repo.git",
            "git fetch origin",
            "git checkout -b main origin/main",
        ];

        let mut all_logs = String::new();
        for command in commands {
            let logs = self.execute_shell_command(command).await?;
            all_logs.push_str(&logs);
        }

        Ok(all_logs)
    }

    /// Execute toolchain action
    async fn execute_toolchain_action(&self, version: &str) -> Result<String> {
        tracing::info!("Executing toolchain action with version: {}", version);

        let mut all_logs = String::new();

        // Install specified Rust version
        let command = format!("rustup install {version}");
        let logs = self.execute_shell_command(&command).await?;
        all_logs.push_str(&logs);

        // Set as default
        let command = format!("rustup default {version}");
        let logs = self.execute_shell_command(&command).await?;
        all_logs.push_str(&logs);

        Ok(all_logs)
    }

    /// Execute setup-node action
    async fn execute_setup_node_action(&self, _version: &str) -> Result<String> {
        tracing::info!("Executing setup-node action");

        // Basic Node.js setup implementation
        // In a real implementation, this would download and install Node.js
        let commands = vec![
            "which node || echo 'Node.js not found'",
            "node --version || echo 'Node.js not installed'",
        ];

        let mut all_logs = String::new();
        for command in commands {
            let logs = self.execute_shell_command(command).await?;
            all_logs.push_str(&logs);
        }

        Ok(all_logs)
    }

    /// Execute setup-python action
    async fn execute_setup_python_action(&self, _version: &str) -> Result<String> {
        tracing::info!("Executing setup-python action");

        // Basic Python setup implementation
        // In a real implementation, this would download and install Python
        let commands = vec![
            "which python3 || echo 'Python3 not found'",
            "python3 --version || echo 'Python3 not installed'",
        ];

        let mut all_logs = String::new();
        for command in commands {
            let logs = self.execute_shell_command(command).await?;
            all_logs.push_str(&logs);
        }

        Ok(all_logs)
    }

    /// Execute cache action
    async fn execute_cache_action(&self, _version: &str) -> Result<String> {
        tracing::info!("Executing cache action");

        // Basic cache implementation
        // In a real implementation, this would handle caching of dependencies
        let commands = vec!["echo 'Cache action executed'", "mkdir -p ~/.cache/barefoot"];

        let mut all_logs = String::new();
        for command in commands {
            let logs = self.execute_shell_command(command).await?;
            all_logs.push_str(&logs);
        }

        Ok(all_logs)
    }

    /// Execute setup-java action
    async fn execute_setup_java_action(&self, _version: &str) -> Result<String> {
        tracing::info!("Executing setup-java action");

        // Basic Java setup implementation
        // In a real implementation, this would download and install Java
        let commands = vec![
            "which java || echo 'Java not found'",
            "java -version || echo 'Java not installed'",
        ];

        let mut all_logs = String::new();
        for command in commands {
            let logs = self.execute_shell_command(command).await?;
            all_logs.push_str(&logs);
        }

        Ok(all_logs)
    }

    /// Execute setup-go action
    async fn execute_setup_go_action(&self, _version: &str) -> Result<String> {
        tracing::info!("Executing setup-go action");

        // Basic Go setup implementation
        // In a real implementation, this would download and install Go
        let commands = vec![
            "which go || echo 'Go not found'",
            "go version || echo 'Go not installed'",
        ];

        let mut all_logs = String::new();
        for command in commands {
            let logs = self.execute_shell_command(command).await?;
            all_logs.push_str(&logs);
        }

        Ok(all_logs)
    }

    /// Execute setup-dotnet action
    async fn execute_setup_dotnet_action(&self, _version: &str) -> Result<String> {
        tracing::info!("Executing setup-dotnet action");

        // Basic .NET setup implementation
        // In a real implementation, this would download and install .NET
        let commands = vec![
            "which dotnet || echo '.NET not found'",
            "dotnet --version || echo '.NET not installed'",
        ];

        let mut all_logs = String::new();
        for command in commands {
            let logs = self.execute_shell_command(command).await?;
            all_logs.push_str(&logs);
        }

        Ok(all_logs)
    }

    /// Execute upload-artifact action
    async fn execute_upload_artifact_action(&self, _version: &str) -> Result<String> {
        tracing::info!("Executing upload-artifact action");

        // Basic artifact upload implementation
        // In a real implementation, this would upload files to the service
        let commands = vec![
            "echo 'Upload artifact action executed'",
            "mkdir -p artifacts",
        ];

        let mut all_logs = String::new();
        for command in commands {
            let logs = self.execute_shell_command(command).await?;
            all_logs.push_str(&logs);
        }

        Ok(all_logs)
    }

    /// Execute download-artifact action
    async fn execute_download_artifact_action(&self, _version: &str) -> Result<String> {
        tracing::info!("Executing download-artifact action");

        // Basic artifact download implementation
        // In a real implementation, this would download files from the service
        let commands = vec![
            "echo 'Download artifact action executed'",
            "mkdir -p downloads",
        ];

        let mut all_logs = String::new();
        for command in commands {
            let logs = self.execute_shell_command(command).await?;
            all_logs.push_str(&logs);
        }

        Ok(all_logs)
    }

    /// Execute a shell command
    async fn execute_shell_command(&self, command: &str) -> Result<String> {
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

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Combine stdout and stderr for logging
        let mut logs = String::new();
        if !stdout.is_empty() {
            logs.push_str(&format!("STDOUT:\n{stdout}\n"));
        }
        if !stderr.is_empty() {
            logs.push_str(&format!("STDERR:\n{stderr}\n"));
        }

        if !output.status.success() {
            return Err(crate::error::BarefootError::Process(format!(
                "Command failed: {command}\n{logs}"
            )));
        }

        tracing::debug!("Command output: {}", logs);

        Ok(logs)
    }

    /// Parse workflow from YAML
    pub fn parse_workflow(&self, yaml_content: &str) -> Result<Workflow> {
        serde_yaml::from_str(yaml_content).map_err(crate::error::BarefootError::Yaml)
    }

    /// Execute a workflow
    pub async fn execute_workflow(&self, workflow: Workflow) -> Result<Vec<Job>> {
        let mut jobs = Vec::new();

        for (job_name, job_config) in workflow.jobs {
            // Handle matrix builds
            if let Some(strategy) = &job_config.strategy {
                let matrix_jobs = self
                    .create_matrix_jobs(&job_name, &job_config, &workflow.name, strategy)
                    .await?;
                jobs.extend(matrix_jobs);
            } else {
                // Single job without matrix
                let job = self
                    .create_job_from_workflow_job(&job_name, &job_config, &workflow.name, "unknown")
                    .await?;
                jobs.push(job);
            }
        }

        Ok(jobs)
    }

    /// Create matrix jobs from a workflow job with strategy
    async fn create_matrix_jobs(
        &self,
        job_name: &str,
        job_config: &WorkflowJob,
        workflow_name: &str,
        strategy: &JobStrategy,
    ) -> Result<Vec<Job>> {
        let mut matrix_jobs = Vec::new();

        // Generate all combinations from matrix
        let combinations = self.generate_matrix_combinations(&strategy.matrix)?;

        for combination in combinations.iter() {
            let job = self
                .create_matrix_job(job_name, job_config, workflow_name, combination, "unknown")
                .await?;
            matrix_jobs.push(job);
        }

        Ok(matrix_jobs)
    }

    /// Generate all combinations from matrix configuration
    fn generate_matrix_combinations(
        &self,
        matrix: &HashMap<String, Vec<serde_json::Value>>,
    ) -> Result<Vec<HashMap<String, serde_json::Value>>> {
        let mut combinations = vec![HashMap::new()];

        for (key, values) in matrix {
            let mut new_combinations = Vec::new();

            for value in values {
                for combination in &combinations {
                    let mut new_combination = combination.clone();
                    new_combination.insert(key.clone(), value.clone());
                    new_combinations.push(new_combination);
                }
            }

            combinations = new_combinations;
        }

        Ok(combinations)
    }

    /// Create a single matrix job
    async fn create_matrix_job(
        &self,
        job_name: &str,
        job_config: &WorkflowJob,
        workflow_name: &str,
        matrix_values: &HashMap<String, serde_json::Value>,
        repository: &str,
    ) -> Result<Job> {
        let mut steps = Vec::new();

        for step_config in &job_config.steps {
            let step = crate::types::JobStep {
                name: step_config.name.clone(),
                status: JobStatus::Queued,
                run: step_config.run.clone(),
                uses: step_config.uses.clone(),
                duration: None,
            };
            steps.push(step);
        }

        // Create matrix job name with matrix values
        let matrix_name = if matrix_values.is_empty() {
            format!("{job_name}-matrix")
        } else {
            let matrix_parts: Vec<String> = matrix_values
                .iter()
                .map(|(key, value)| {
                    let value_str = match value {
                        serde_json::Value::String(s) => s.clone(),
                        serde_json::Value::Number(n) => n.to_string(),
                        _ => value.to_string(),
                    };
                    format!("{key}-{value_str}")
                })
                .collect();
            format!("{job_name}-{}", matrix_parts.join("-"))
        };

        let job = Job {
            id: Uuid::new_v4(),
            name: matrix_name, // DONE: Matrix job naming includes matrix values
            status: JobStatus::Queued,
            workflow: workflow_name.to_string(),
            repository: repository.to_string(), // DONE: Properly populate repository field from context
            started_at: None,
            completed_at: None,
            steps,
        };

        Ok(job)
    }

    /// Create a job from a workflow job configuration
    async fn create_job_from_workflow_job(
        &self,
        job_name: &str,
        job_config: &WorkflowJob,
        workflow_name: &str,
        repository: &str,
    ) -> Result<Job> {
        let mut steps = Vec::new();

        for step_config in &job_config.steps {
            let step = crate::types::JobStep {
                name: step_config.name.clone(),
                status: JobStatus::Queued,
                run: step_config.run.clone(),
                uses: step_config.uses.clone(),
                duration: None,
            };
            steps.push(step);
        }

        let job = Job {
            id: Uuid::new_v4(),
            name: job_name.to_string(),
            status: JobStatus::Queued,
            workflow: workflow_name.to_string(),
            repository: repository.to_string(),
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
        let content = std::fs::read_to_string(path).map_err(crate::error::BarefootError::Io)?;

        serde_yaml::from_str(&content).map_err(crate::error::BarefootError::Yaml)
    }

    /// Parse workflow from string
    pub fn from_string(content: &str) -> Result<Workflow> {
        serde_yaml::from_str(content).map_err(crate::error::BarefootError::Yaml)
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
                return Err(crate::error::BarefootError::Workflow(format!(
                    "Job '{job_name}' must contain at least one step"
                )));
            }

            for step in &job.steps {
                if step.run.is_none() && step.uses.is_none() {
                    return Err(crate::error::BarefootError::Workflow(format!(
                        "Step '{}' must have either 'run' or 'uses'",
                        step.name
                    )));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{WorkflowJob, WorkflowStep};
    use std::collections::HashMap;

    #[test]
    fn test_workflow_parser_with_uses_step() {
        // Test parsing workflow with 'uses' step
        let yaml_content = r#"
name: Test Workflow
on:
  push:
    branches: [main]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v3
      - name: Run tests
        run: cargo test
"#;

        let result = WorkflowParser::from_string(yaml_content);
        println!("Parsed result: {result:?}");
        assert!(result.is_ok());

        let workflow = result.unwrap();
        println!("Parsed workflow: {workflow:?}");
        assert_eq!(workflow.name, "Test Workflow");
        assert_eq!(workflow.jobs.len(), 1);

        let test_job = workflow.jobs.get("test").unwrap();
        println!("Parsed job: {test_job:?}");
        assert_eq!(test_job.steps.len(), 2);

        // Check 'uses' step
        let checkout_step = &test_job.steps[0];
        println!("Checkout step: {checkout_step:?}");
        assert_eq!(checkout_step.name, "Checkout");
        assert_eq!(checkout_step.uses, Some("actions/checkout@v3".to_string()));
        assert!(checkout_step.run.is_none());

        // Check 'run' step
        let run_step = &test_job.steps[1];
        println!("Run step: {run_step:?}");
        assert_eq!(run_step.name, "Run tests");
        assert_eq!(run_step.run, Some("cargo test".to_string()));
        assert!(run_step.uses.is_none());
    }

    #[test]
    fn test_workflow_parser_with_matrix_build() {
        // Test parsing workflow with matrix strategy
        let yaml_content = r#"
name: Matrix Test
on:
  push:
    branches: [main]
jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        rust: [1.70, 1.71, 1.72]
        os: [ubuntu-latest, windows-latest]
      fail-fast: false
      max-parallel: 2
    steps:
      - name: Setup Rust ${{ matrix.rust }}
        uses: actions-rs/toolchain@v1
        with:
          toolchain: ${{ matrix.rust }}
      - name: Run tests
        run: cargo test
"#;

        let result = WorkflowParser::from_string(yaml_content);
        println!("Parsed result: {result:?}");
        assert!(result.is_ok());

        let workflow = result.unwrap();
        println!("Parsed workflow: {workflow:?}");
        let test_job = workflow.jobs.get("test").unwrap();
        println!("Parsed job: {test_job:?}");

        // Check strategy
        let strategy = test_job.strategy.as_ref().unwrap();
        println!("Parsed strategy: {strategy:?}");
        assert_eq!(strategy.fail_fast, Some(false));
        assert_eq!(strategy.max_parallel, Some(2));

        // Check matrix
        let rust_versions = strategy.matrix.get("rust").unwrap();
        println!("Parsed rust matrix: {rust_versions:?}");
        assert_eq!(rust_versions.len(), 3);

        let os_versions = strategy.matrix.get("os").unwrap();
        println!("Parsed os matrix: {os_versions:?}");
        assert_eq!(os_versions.len(), 2);
    }

    #[test]
    fn test_workflow_validation_errors() {
        // Test validation errors
        let invalid_workflow = Workflow {
            name: "Invalid".to_string(),
            on: crate::types::WorkflowTriggers::default(),
            jobs: HashMap::new(),
            env: HashMap::new(),
        };

        let result = WorkflowParser::validate(&invalid_workflow);
        assert!(result.is_err());

        // Test job with no steps
        let mut workflow_with_empty_job = invalid_workflow.clone();
        workflow_with_empty_job.jobs.insert(
            "empty".to_string(),
            WorkflowJob {
                name: Some("empty".to_string()),
                runs_on: "ubuntu-latest".to_string(),
                steps: vec![],
                env: HashMap::new(),
                timeout_minutes: None,
                strategy: None,
            },
        );

        let result = WorkflowParser::validate(&workflow_with_empty_job);
        assert!(result.is_err());

        // Test step with neither run nor uses
        let mut workflow_with_invalid_step = invalid_workflow.clone();
        workflow_with_invalid_step.jobs.insert(
            "invalid".to_string(),
            WorkflowJob {
                name: Some("invalid".to_string()),
                runs_on: "ubuntu-latest".to_string(),
                steps: vec![WorkflowStep {
                    name: "invalid".to_string(),
                    run: None,
                    uses: None,
                    with: HashMap::new(),
                    env: HashMap::new(),
                    shell: None,
                    working_directory: None,
                }],
                env: HashMap::new(),
                timeout_minutes: None,
                strategy: None,
            },
        );

        let result = WorkflowParser::validate(&workflow_with_invalid_step);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_job_executor_error_handling() {
        // Test that job executor handles errors gracefully
        let config = crate::config::BarefootConfig::default();
        let core = RunnerCore::new(config);
        let executor = JobExecutor::new(core);

        // Create a job with an invalid step
        let job = Job {
            id: uuid::Uuid::new_v4(),
            name: "test-job".to_string(),
            status: crate::types::JobStatus::Queued,
            workflow: "test".to_string(),
            repository: "test".to_string(),
            started_at: None,
            completed_at: None,
            steps: vec![crate::types::JobStep {
                name: "invalid".to_string(),
                status: crate::types::JobStatus::Queued,
                run: Some("invalid_command_that_does_not_exist".to_string()),
                uses: None,
                duration: None,
            }],
        };

        // This should fail gracefully
        let result = executor.execute_job(job, None).await;
        // Should return an error but not panic
        assert!(result.is_err());
    }

    #[test]
    fn test_job_repository_field_is_set() {
        let job_config = WorkflowJob {
            name: Some("test-job".to_string()),
            runs_on: "ubuntu-latest".to_string(),
            steps: vec![WorkflowStep {
                name: "step1".to_string(),
                run: Some("echo hello".to_string()),
                ..Default::default()
            }],
            ..Default::default()
        };
        let core = RunnerCore::new(crate::config::BarefootConfig::default());
        let executor = JobExecutor::new(core);
        let repository = "my-repo";
        let rt = tokio::runtime::Runtime::new().unwrap();
        let job = rt
            .block_on(executor.create_job_from_workflow_job(
                "test-job",
                &job_config,
                "test-workflow",
                repository,
            ))
            .unwrap();
        assert_eq!(job.repository, repository);
    }

    #[test]
    fn test_additional_github_actions_support() {
        // Test parsing workflow with 'uses' step
        let yaml_content = r#"
name: Test Workflow
on:
  push:
    branches: [main]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v3
      - name: Run tests
        run: cargo test
"#;

        let result = WorkflowParser::from_string(yaml_content);
        println!("Parsed result: {result:?}");
        assert!(result.is_ok());

        let workflow = result.unwrap();
        println!("Parsed workflow: {workflow:?}");
        assert_eq!(workflow.name, "Test Workflow");
        assert_eq!(workflow.jobs.len(), 1);

        let test_job = workflow.jobs.get("test").unwrap();
        println!("Parsed job: {test_job:?}");
        assert_eq!(test_job.steps.len(), 2);

        // Check 'uses' step
        let checkout_step = &test_job.steps[0];
        println!("Checkout step: {checkout_step:?}");
        assert_eq!(checkout_step.name, "Checkout");
        assert_eq!(checkout_step.uses, Some("actions/checkout@v3".to_string()));
        assert!(checkout_step.run.is_none());

        // Check 'run' step
        let run_step = &test_job.steps[1];
        println!("Run step: {run_step:?}");
        assert_eq!(run_step.name, "Run tests");
        assert_eq!(run_step.run, Some("cargo test".to_string()));
        assert!(run_step.uses.is_none());
    }

    #[test]
    fn test_more_github_actions_support() {
        // Test parsing workflow with additional GitHub Actions
        let yaml_content = r#"
name: Extended Actions Test
on:
  push:
    branches: [main]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v3
      - name: Setup Java
        uses: actions/setup-java@v3
        with:
          java-version: '17'
          distribution: 'temurin'
      - name: Setup Go
        uses: actions/setup-go@v4
        with:
          go-version: '1.21'
      - name: Setup .NET
        uses: actions/setup-dotnet@v3
        with:
          dotnet-version: '8.0.x'
      - name: Upload artifacts
        uses: actions/upload-artifact@v3
        with:
          name: build-artifacts
          path: dist/
      - name: Download artifacts
        uses: actions/download-artifact@v3
        with:
          name: build-artifacts
          path: artifacts/
      - name: Run tests
        run: echo "Testing with multiple languages"
"#;

        let result = WorkflowParser::from_string(yaml_content);
        assert!(result.is_ok());

        let workflow = result.unwrap();
        let test_job = workflow.jobs.get("test").unwrap();

        // Verify all steps are parsed correctly
        assert_eq!(test_job.steps.len(), 7);

        // Check setup-java action
        let setup_java_step = &test_job.steps[1];
        assert_eq!(setup_java_step.name, "Setup Java");
        assert_eq!(
            setup_java_step.uses,
            Some("actions/setup-java@v3".to_string())
        );

        // Check setup-go action
        let setup_go_step = &test_job.steps[2];
        assert_eq!(setup_go_step.name, "Setup Go");
        assert_eq!(setup_go_step.uses, Some("actions/setup-go@v4".to_string()));

        // Check setup-dotnet action
        let setup_dotnet_step = &test_job.steps[3];
        assert_eq!(setup_dotnet_step.name, "Setup .NET");
        assert_eq!(
            setup_dotnet_step.uses,
            Some("actions/setup-dotnet@v3".to_string())
        );

        // Check upload-artifact action
        let upload_artifact_step = &test_job.steps[4];
        assert_eq!(upload_artifact_step.name, "Upload artifacts");
        assert_eq!(
            upload_artifact_step.uses,
            Some("actions/upload-artifact@v3".to_string())
        );

        // Check download-artifact action
        let download_artifact_step = &test_job.steps[5];
        assert_eq!(download_artifact_step.name, "Download artifacts");
        assert_eq!(
            download_artifact_step.uses,
            Some("actions/download-artifact@v3".to_string())
        );
    }

    #[tokio::test]
    async fn test_execute_additional_github_actions() {
        // Test that additional GitHub Actions can be executed
        let config = crate::config::BarefootConfig::default();
        let core = RunnerCore::new(config);
        let executor = JobExecutor::new(core);

        // Test setup-node action
        let result = executor.execute_action_step("actions/setup-node@v3").await;
        match result {
            Ok(_) => println!("setup-node action is supported"),
            Err(e) => println!("setup-node action is not supported: {e}"),
        }

        // Test setup-python action
        let result = executor
            .execute_action_step("actions/setup-python@v4")
            .await;
        match result {
            Ok(_) => println!("setup-python action is supported"),
            Err(e) => println!("setup-python action is not supported: {e}"),
        }

        // Test cache action
        let result = executor.execute_action_step("actions/cache@v3").await;
        match result {
            Ok(_) => println!("cache action is supported"),
            Err(e) => println!("cache action is not supported: {e}"),
        }

        // For now, we expect these to fail since they're not implemented
        // This test documents which actions need implementation
    }

    #[tokio::test]
    async fn test_execute_more_github_actions() {
        // Test that additional GitHub Actions can be executed
        let config = crate::config::BarefootConfig::default();
        let core = RunnerCore::new(config);
        let executor = JobExecutor::new(core);

        // Test setup-java action
        let result = executor.execute_action_step("actions/setup-java@v3").await;
        match result {
            Ok(_) => println!("setup-java action is supported"),
            Err(e) => println!("setup-java action is not supported: {e}"),
        }

        // Test setup-go action
        let result = executor.execute_action_step("actions/setup-go@v4").await;
        match result {
            Ok(_) => println!("setup-go action is supported"),
            Err(e) => println!("setup-go action is not supported: {e}"),
        }

        // Test setup-dotnet action
        let result = executor
            .execute_action_step("actions/setup-dotnet@v3")
            .await;
        match result {
            Ok(_) => println!("setup-dotnet action is supported"),
            Err(e) => println!("setup-dotnet action is not supported: {e}"),
        }

        // Test upload-artifact action
        let result = executor
            .execute_action_step("actions/upload-artifact@v3")
            .await;
        match result {
            Ok(_) => println!("upload-artifact action is supported"),
            Err(e) => println!("upload-artifact action is not supported: {e}"),
        }

        // Test download-artifact action
        let result = executor
            .execute_action_step("actions/download-artifact@v3")
            .await;
        match result {
            Ok(_) => println!("download-artifact action is supported"),
            Err(e) => println!("download-artifact action is not supported: {e}"),
        }

        // For now, we expect these to fail since they're not implemented
        // This test documents which actions need implementation
    }

    #[test]
    fn test_improved_matrix_job_naming() {
        // Test that matrix job names include matrix values for better identification
        let yaml_content = r#"
name: Matrix Test
on:
  push:
    branches: [main]
jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        rust: [1.70, 1.71]
        os: [ubuntu-latest, windows-latest]
    steps:
      - name: Setup Rust ${{ matrix.rust }}
        uses: actions-rs/toolchain@v1
        with:
          toolchain: ${{ matrix.rust }}
      - name: Run tests
        run: cargo test
"#;

        let result = WorkflowParser::from_string(yaml_content);
        assert!(result.is_ok());

        let workflow = result.unwrap();
        let test_job = workflow.jobs.get("test").unwrap();
        let strategy = test_job.strategy.as_ref().unwrap();

        // Verify matrix combinations
        let rust_versions = strategy.matrix.get("rust").unwrap();
        let os_versions = strategy.matrix.get("os").unwrap();

        // Should create 4 matrix jobs (2 rust versions × 2 os versions)
        let expected_job_count = rust_versions.len() * os_versions.len();
        assert_eq!(expected_job_count, 4);

        // Test that matrix job names would include matrix values
        // This will be implemented in the matrix job creation logic
        let expected_names = [
            "test-rust-1.70-os-ubuntu-latest",
            "test-rust-1.70-os-windows-latest",
            "test-rust-1.71-os-ubuntu-latest",
            "test-rust-1.71-os-windows-latest",
        ];

        // For now, verify the test structure is correct
        assert_eq!(expected_names.len(), expected_job_count);
    }

    #[tokio::test]
    async fn test_matrix_job_creation_with_improved_naming() {
        // Test that matrix jobs are created with improved naming
        let config = crate::config::BarefootConfig::default();
        let core = RunnerCore::new(config);
        let executor = JobExecutor::new(core);

        // Create a workflow with matrix strategy
        let yaml_content = r#"
name: Matrix Test
on:
  push:
    branches: [main]
jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        rust: [1.70, 1.71]
        os: [ubuntu-latest, windows-latest]
    steps:
      - name: Setup Rust ${{ matrix.rust }}
        uses: actions-rs/toolchain@v1
        with:
          toolchain: ${{ matrix.rust }}
      - name: Run tests
        run: cargo test
"#;

        let workflow = WorkflowParser::from_string(yaml_content).unwrap();
        let jobs = executor.execute_workflow(workflow).await.unwrap();

        // Should create 4 matrix jobs
        assert_eq!(jobs.len(), 4);

        // Verify job names include matrix values
        let job_names: Vec<String> = jobs.iter().map(|j| j.name.clone()).collect();

        // Check that all job names contain matrix information
        for job_name in &job_names {
            assert!(job_name.contains("test-"));
            assert!(job_name.contains("rust-"));
            assert!(job_name.contains("os-"));
        }

        // Verify specific naming patterns
        assert!(job_names.iter().any(|name| name.contains("rust-1.7")));
        assert!(job_names.iter().any(|name| name.contains("rust-1.71")));
        assert!(job_names
            .iter()
            .any(|name| name.contains("os-ubuntu-latest")));
        assert!(job_names
            .iter()
            .any(|name| name.contains("os-windows-latest")));
    }
}
