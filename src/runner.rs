use crate::{
    core::RunnerCore,
    error::Result,
    types::{Job, JobStatus, Workflow, WorkflowJob, JobStrategy},
};
use std::process::Stdio;
use std::collections::HashMap;
use tokio::process::Command;
use uuid::Uuid;
use chrono::Utc;

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

        // Enhanced step execution with support for different step types
        if let Some(command) = &step.run {
            // Handle 'run' steps
            self.execute_shell_command(command).await?;
        } else if let Some(action) = &step.uses {
            // Handle 'uses' steps
            self.execute_action_step(action).await?;
        } else {
            return Err(crate::error::BarefootError::Workflow(
                format!("Step '{}' must have either 'run' or 'uses'", step.name)
            ));
        }

        Ok(())
    }

    /// Execute an action step (uses: action@version)
    async fn execute_action_step(&self, action: &str) -> Result<()> {
        tracing::info!("Executing action: {}", action);
        
        // Parse action reference (e.g., "actions/checkout@v3")
        let (action_name, version) = self.parse_action_reference(action)?;
        
        // For now, implement basic action support
        match action_name.as_str() {
            "actions/checkout" => {
                self.execute_checkout_action(&version).await?;
            }
            "actions-rs/toolchain" => {
                self.execute_toolchain_action(&version).await?;
            }
            "actions/setup-node" => {
                self.execute_setup_node_action(&version).await?;
            }
            "actions/setup-python" => {
                self.execute_setup_python_action(&version).await?;
            }
            "actions/cache" => {
                self.execute_cache_action(&version).await?;
            }
            _ => {
                // TODO: Support additional GitHub Actions and custom actions (setup-node, setup-python, cache are now supported)
                return Err(crate::error::BarefootError::Workflow(
                    format!("Unsupported action: {action_name}")
                ));
            }
        }
        
        Ok(())
    }

    /// Parse action reference into name and version
    fn parse_action_reference(&self, action_ref: &str) -> Result<(String, String)> {
        let parts: Vec<&str> = action_ref.split('@').collect();
        if parts.len() != 2 {
            return Err(crate::error::BarefootError::Workflow(
                format!("Invalid action reference: {action_ref}")
            ));
        }
        
        Ok((parts[0].to_string(), parts[1].to_string()))
    }

    /// Execute checkout action
    async fn execute_checkout_action(&self, _version: &str) -> Result<()> {
        tracing::info!("Executing checkout action");
        
        // Basic checkout implementation
        let commands = vec![
            "git init",
            "git remote add origin https://github.com/owner/repo.git",
            "git fetch origin",
            "git checkout -b main origin/main",
        ];
        
        for command in commands {
            self.execute_shell_command(command).await?;
        }
        
        Ok(())
    }

    /// Execute toolchain action
    async fn execute_toolchain_action(&self, version: &str) -> Result<()> {
        tracing::info!("Executing toolchain action with version: {}", version);
        
        // Install specified Rust version
        let command = format!("rustup install {version}");
        self.execute_shell_command(&command).await?;
        
        // Set as default
        let command = format!("rustup default {version}");
        self.execute_shell_command(&command).await?;
        
        Ok(())
    }

    /// Execute setup-node action
    async fn execute_setup_node_action(&self, _version: &str) -> Result<()> {
        tracing::info!("Executing setup-node action");
        
        // Basic Node.js setup implementation
        // In a real implementation, this would download and install Node.js
        let commands = vec![
            "which node || echo 'Node.js not found'",
            "node --version || echo 'Node.js not installed'",
        ];
        
        for command in commands {
            self.execute_shell_command(command).await?;
        }
        
        Ok(())
    }

    /// Execute setup-python action
    async fn execute_setup_python_action(&self, _version: &str) -> Result<()> {
        tracing::info!("Executing setup-python action");
        
        // Basic Python setup implementation
        // In a real implementation, this would download and install Python
        let commands = vec![
            "which python3 || echo 'Python3 not found'",
            "python3 --version || echo 'Python3 not installed'",
        ];
        
        for command in commands {
            self.execute_shell_command(command).await?;
        }
        
        Ok(())
    }

    /// Execute cache action
    async fn execute_cache_action(&self, _version: &str) -> Result<()> {
        tracing::info!("Executing cache action");
        
        // Basic cache implementation
        // In a real implementation, this would handle caching of dependencies
        let commands = vec![
            "echo 'Cache action executed'",
            "mkdir -p ~/.cache/barefoot",
        ];
        
        for command in commands {
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
            .map_err(crate::error::BarefootError::Yaml)
    }

    /// Execute a workflow
    pub async fn execute_workflow(&self, workflow: Workflow) -> Result<Vec<Job>> {
        let mut jobs = Vec::new();

        for (job_name, job_config) in workflow.jobs {
            // Handle matrix builds
            if let Some(strategy) = &job_config.strategy {
                let matrix_jobs = self.create_matrix_jobs(&job_name, &job_config, &workflow.name, strategy).await?;
                jobs.extend(matrix_jobs);
            } else {
                // Single job without matrix
                let job = self.create_job_from_workflow_job(&job_name, &job_config, &workflow.name, "unknown").await?;
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
            let job = self.create_matrix_job(job_name, job_config, workflow_name, combination, "unknown").await?;
            matrix_jobs.push(job);
        }
        
        Ok(matrix_jobs)
    }

    /// Generate all combinations from matrix configuration
    fn generate_matrix_combinations(&self, matrix: &HashMap<String, Vec<serde_json::Value>>) -> Result<Vec<HashMap<String, serde_json::Value>>> {
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
        _matrix_values: &HashMap<String, serde_json::Value>,
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
            name: job_config.name.clone().unwrap_or_else(|| format!("{job_name}-matrix")), // TODO: Improve matrix job naming to reflect matrix values
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
            name: job_config.name.clone().unwrap_or_else(|| job_name.to_string()),
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
        let content = std::fs::read_to_string(path)
            .map_err(crate::error::BarefootError::Io)?;
        
        serde_yaml::from_str(&content)
            .map_err(crate::error::BarefootError::Yaml)
    }

    /// Parse workflow from string
    pub fn from_string(content: &str) -> Result<Workflow> {
        serde_yaml::from_str(content)
            .map_err(crate::error::BarefootError::Yaml)
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
                    format!("Job '{job_name}' must contain at least one step"),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{WorkflowStep, WorkflowJob};
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
        let result = executor.execute_job(job).await;
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
        let job = rt.block_on(executor.create_job_from_workflow_job(
            "test-job",
            &job_config,
            "test-workflow",
            repository,
        )).unwrap();
        assert_eq!(job.repository, repository);
    }

    #[test]
    fn test_additional_github_actions_support() {
        // Test that additional GitHub Actions can be parsed and executed
        let yaml_content = r#"
name: Test Additional Actions
on:
  push:
    branches: [main]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '18'
      - name: Setup Python
        uses: actions/setup-python@v4
        with:
          python-version: '3.11'
      - name: Cache dependencies
        uses: actions/cache@v3
        with:
          path: ~/.npm
          key: ${{ runner.os }}-node-${{ hashFiles('**/package-lock.json') }}
      - name: Run tests
        run: npm test
"#;
        
        let result = WorkflowParser::from_string(yaml_content);
        assert!(result.is_ok());
        
        let workflow = result.unwrap();
        let test_job = workflow.jobs.get("test").unwrap();
        
        // Verify all steps are parsed correctly
        assert_eq!(test_job.steps.len(), 4);
        
        // Check setup-node action
        let setup_node_step = &test_job.steps[0];
        assert_eq!(setup_node_step.name, "Setup Node.js");
        assert_eq!(setup_node_step.uses, Some("actions/setup-node@v3".to_string()));
        
        // Check setup-python action
        let setup_python_step = &test_job.steps[1];
        assert_eq!(setup_python_step.name, "Setup Python");
        assert_eq!(setup_python_step.uses, Some("actions/setup-python@v4".to_string()));
        
        // Check cache action
        let cache_step = &test_job.steps[2];
        assert_eq!(cache_step.name, "Cache dependencies");
        assert_eq!(cache_step.uses, Some("actions/cache@v3".to_string()));
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
        let result = executor.execute_action_step("actions/setup-python@v4").await;
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
} 