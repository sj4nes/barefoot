//! MCP tools for barefoot runner
//
// TODO: Add MCP tool for weather dashboard (summarize job health, system health)
// TODO: Add MCP tool for alerts and notifications (failing jobs, stuck jobs, degraded service)
// TODO: Add MCP tool for dependency graph visualization
// TODO: Add MCP tool for sparkline and cycle time analytics (minutes/hours/days, avg/p99/last)

use super::*;
use crate::core::RunnerCore;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::Utc;
use crate::error::Result;

/// Tool handler trait for MCP tools
pub trait ToolHandler: Send + Sync {
    /// Get tool definition
    fn definition(&self) -> ToolDefinition;
    
    /// Validate tool arguments
    fn validate_args(&self, args: &serde_json::Value) -> Result<()>;
}

/// Async tool execution enum to avoid async trait objects
#[derive(Clone)]
pub enum AsyncToolHandler {
    StartJob(StartJobTool),
    StopJob(StopJobTool),
    HealthCheck(HealthCheckTool),
}

impl AsyncToolHandler {
    /// Execute the tool
    pub async fn execute(&self, args: serde_json::Value) -> Result<ToolResult> {
        match self {
            AsyncToolHandler::StartJob(t) => t.execute(args).await,
            AsyncToolHandler::StopJob(t) => t.execute(args).await,
            AsyncToolHandler::HealthCheck(t) => t.execute(args).await,
        }
    }
    
    /// Get tool definition
    pub fn definition(&self) -> ToolDefinition {
        match self {
            AsyncToolHandler::StartJob(t) => t.definition(),
            AsyncToolHandler::StopJob(t) => t.definition(),
            AsyncToolHandler::HealthCheck(t) => t.definition(),
        }
    }
}

/// Start job tool handler
#[derive(Clone)]
pub struct StartJobTool {
    runner_core: Arc<RwLock<RunnerCore>>,
}

impl StartJobTool {
    pub fn new(runner_core: Arc<RwLock<RunnerCore>>) -> Self {
        Self { runner_core }
    }
    
    pub fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "start_job".to_string(),
            description: "Start execution of a specific job".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "job_id": {"type": "string"},
                    "priority": {"type": "integer", "minimum": 1, "maximum": 10}
                },
                "required": ["job_id"]
            }),
            permissions: vec!["job:execute".to_string()],
        }
    }
    
    pub fn validate_args(&self, args: &serde_json::Value) -> Result<()> {
        if !args.is_object() {
            return Err(BarefootError::Mcp("Arguments must be an object".to_string()));
        }
        
        if let Some(job_id) = args.get("job_id") {
            if !job_id.is_string() {
                return Err(BarefootError::Mcp("job_id must be a string".to_string()));
            }
        } else {
            return Err(BarefootError::Mcp("job_id is required".to_string()));
        }
        
        if let Some(priority) = args.get("priority") {
            if !priority.is_number() {
                return Err(BarefootError::Mcp("priority must be a number".to_string()));
            }
            if let Some(priority_val) = priority.as_u64() {
                if priority_val < 1 || priority_val > 10 {
                    return Err(BarefootError::Mcp("priority must be between 1 and 10".to_string()));
                }
            }
        }
        
        Ok(())
    }
    
    pub async fn execute(&self, args: serde_json::Value) -> Result<ToolResult> {
        self.validate_args(&args)?;
        
        let job_id = args["job_id"].as_str().unwrap();
        let priority = args.get("priority").and_then(|p| p.as_u64()).unwrap_or(5);
        
        // Parse job ID
        let job_uuid = uuid::Uuid::parse_str(job_id)
            .map_err(|_| BarefootError::Mcp("Invalid job ID format".to_string()))?;
        
        let runner_core = self.runner_core.read().await;
        
        // Check if runner can accept jobs
        if !runner_core.can_accept_jobs().await {
            return Ok(ToolResult {
                success: false,
                content: serde_json::json!({
                    "error": "Runner cannot accept more jobs",
                    "job_id": job_id,
                    "priority": priority,
                }),
                error: Some("Runner cannot accept more jobs".to_string()),
                duration: None,
            });
        }
        
        // Create a mock job for demonstration
        let job = crate::types::Job {
            id: job_uuid,
            name: format!("job-{}", job_id),
            status: crate::types::JobStatus::Queued,
            workflow: "mcp-triggered".to_string(),
            repository: "mcp".to_string(),
            started_at: None,
            completed_at: None,
            steps: vec![],
        };
        
        // Queue the job
        match runner_core.queue_job(job.clone()).await {
            Ok(_) => {
                let content = serde_json::json!({
                    "success": true,
                    "job_id": job_id,
                    "priority": priority,
                    "message": "Job queued successfully",
                    "queue_size": runner_core.queue_size().await,
                    "timestamp": Utc::now().to_rfc3339(),
                });
                
                Ok(ToolResult {
                    success: true,
                    content,
                    error: None,
                    duration: None,
                })
            }
            Err(e) => {
                Ok(ToolResult {
                    success: false,
                    content: serde_json::json!({
                        "error": e.to_string(),
                        "job_id": job_id,
                        "priority": priority,
                    }),
                    error: Some(e.to_string()),
                    duration: None,
                })
            }
        }
    }
}

/// Stop job tool handler
#[derive(Clone)]
pub struct StopJobTool {
    runner_core: Arc<RwLock<RunnerCore>>,
}

impl StopJobTool {
    pub fn new(runner_core: Arc<RwLock<RunnerCore>>) -> Self {
        Self { runner_core }
    }
    
    pub fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "stop_job".to_string(),
            description: "Stop/cancel a running job".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "job_id": {"type": "string"},
                    "force": {"type": "boolean"}
                },
                "required": ["job_id"]
            }),
            permissions: vec!["job:control".to_string()],
        }
    }
    
    pub fn validate_args(&self, args: &serde_json::Value) -> Result<()> {
        if !args.is_object() {
            return Err(BarefootError::Mcp("Arguments must be an object".to_string()));
        }
        
        if let Some(job_id) = args.get("job_id") {
            if !job_id.is_string() {
                return Err(BarefootError::Mcp("job_id must be a string".to_string()));
            }
        } else {
            return Err(BarefootError::Mcp("job_id is required".to_string()));
        }
        
        Ok(())
    }
    
    pub async fn execute(&self, args: serde_json::Value) -> Result<ToolResult> {
        self.validate_args(&args)?;
        
        let job_id = args["job_id"].as_str().unwrap();
        let force = args.get("force").and_then(|f| f.as_bool()).unwrap_or(false);
        
        // Parse job ID
        let job_uuid = uuid::Uuid::parse_str(job_id)
            .map_err(|_| BarefootError::Mcp("Invalid job ID format".to_string()))?;
        
        let runner_core = self.runner_core.read().await;
        let current_jobs = runner_core.current_jobs().await;
        
        // Check if job is currently running
        let job_found = current_jobs.iter().find(|j| j.id == job_uuid);
        
        if let Some(_job) = job_found {
            // Complete the job with cancelled status
            match runner_core.complete_job(job_uuid, crate::types::JobStatus::Cancelled).await {
                Ok(_) => {
                    let content = serde_json::json!({
                        "success": true,
                        "job_id": job_id,
                        "force": force,
                        "message": "Job stopped successfully",
                        "job_status": "cancelled",
                        "timestamp": Utc::now().to_rfc3339(),
                    });
                    
                    Ok(ToolResult {
                        success: true,
                        content,
                        error: None,
                        duration: None,
                    })
                }
                Err(e) => {
                    Ok(ToolResult {
                        success: false,
                        content: serde_json::json!({
                            "error": e.to_string(),
                            "job_id": job_id,
                            "force": force,
                        }),
                        error: Some(e.to_string()),
                        duration: None,
                    })
                }
            }
        } else {
            // Job not found or not running
            let content = serde_json::json!({
                "success": false,
                "job_id": job_id,
                "force": force,
                "message": "Job not found or not running",
                "available_jobs": current_jobs.iter().map(|j| j.id.to_string()).collect::<Vec<_>>(),
                "timestamp": Utc::now().to_rfc3339(),
            });
            
            Ok(ToolResult {
                success: false,
                content,
                error: Some("Job not found or not running".to_string()),
                duration: None,
            })
        }
    }
}

/// Health check tool handler
#[derive(Clone)]
pub struct HealthCheckTool {
    runner_core: Arc<RwLock<RunnerCore>>,
}

impl HealthCheckTool {
    pub fn new(runner_core: Arc<RwLock<RunnerCore>>) -> Self {
        Self { runner_core }
    }
    
    pub fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "health_check".to_string(),
            description: "Perform a health check on the runner".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "detailed": {"type": "boolean"}
                }
            }),
            permissions: vec!["runner:read".to_string()],
        }
    }
    
    pub fn validate_args(&self, _args: &serde_json::Value) -> Result<()> {
        Ok(()) // No validation needed for health check
    }
    
    pub async fn execute(&self, args: serde_json::Value) -> Result<ToolResult> {
        self.validate_args(&args)?;
        
        let detailed = args.get("detailed").and_then(|d| d.as_bool()).unwrap_or(false);
        let runner_core = self.runner_core.read().await;
        let status = runner_core.status().await;
        
        let mut content = serde_json::json!({
            "status": status.to_string(),
            "active_jobs": runner_core.current_jobs().await.len(),
            "queue_size": runner_core.queue_size().await,
            "can_accept_jobs": runner_core.can_accept_jobs().await,
            "health_score": self.calculate_health_score(&runner_core).await,
            "timestamp": Utc::now().to_rfc3339(),
        });
        
        if detailed {
            content["detailed"] = serde_json::json!({
                "capabilities": runner_core.capabilities(),
                "queue": runner_core.job_queue().await,
                "active_jobs": runner_core.current_jobs().await,
            });
        }
        
        Ok(ToolResult {
            success: true,
            content,
            error: None,
            duration: None,
        })
    }
    
    async fn calculate_health_score(&self, runner_core: &RunnerCore) -> f64 {
        let mut score = 100.0;
        
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

/// Tool manager using enum-based handlers
pub struct ToolManager {
    tools: std::collections::HashMap<String, Box<dyn ToolHandler>>,
    async_tools: std::collections::HashMap<String, AsyncToolHandler>,
}

impl ToolManager {
    pub fn new() -> Self {
        Self {
            tools: std::collections::HashMap::new(),
            async_tools: std::collections::HashMap::new(),
        }
    }
    
    pub fn register_tool(&mut self, tool: Box<dyn ToolHandler>) {
        let name = tool.definition().name.clone();
        self.tools.insert(name, tool);
    }
    
    pub fn register_async_tool(&mut self, tool: AsyncToolHandler) {
        let name = tool.definition().name.clone();
        self.async_tools.insert(name, tool);
    }
    
    pub async fn execute_tool(&self, name: &str, args: serde_json::Value) -> Result<ToolResult> {
        if let Some(tool) = self.async_tools.get(name) {
            tool.execute(args).await
        } else {
            Err(BarefootError::Mcp(format!("Tool not found: {}", name)))
        }
    }
    
    pub fn list_tools(&self) -> Vec<ToolDefinition> {
        let mut definitions = Vec::new();
        
        // Add sync tools
        for tool in self.tools.values() {
            definitions.push(tool.definition());
        }
        
        // Add async tools
        for tool in self.async_tools.values() {
            definitions.push(tool.definition());
        }
        
        definitions
    }
    
    pub fn has_tool(&self, name: &str) -> bool {
        self.tools.contains_key(name) || self.async_tools.contains_key(name)
    }
    
    pub fn get_tool_definition(&self, name: &str) -> Option<ToolDefinition> {
        if let Some(tool) = self.tools.get(name) {
            Some(tool.definition())
        } else if let Some(tool) = self.async_tools.get(name) {
            Some(tool.definition())
        } else {
            None
        }
    }
}

impl Default for ToolManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::BarefootConfig;
    
    #[tokio::test]
    async fn test_start_job_tool() {
        let runner_core = Arc::new(RwLock::new(RunnerCore::new(BarefootConfig::default())));
        let tool = StartJobTool::new(runner_core);
        let valid_uuid = uuid::Uuid::new_v4().to_string();
        let args = serde_json::json!({"job_id": valid_uuid, "priority": 5});
        let result = tool.execute(args).await.unwrap();
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_stop_job_tool() {
        let runner_core = Arc::new(RwLock::new(RunnerCore::new(BarefootConfig::default())));
        let tool = StopJobTool::new(runner_core);
        let valid_uuid = uuid::Uuid::new_v4().to_string();
        let args = serde_json::json!({"job_id": valid_uuid, "force": true});
        let result = tool.execute(args).await.unwrap();
        assert!(!result.success); // Job won't be found, but should not panic
    }

    #[tokio::test]
    async fn test_health_check_tool() {
        let runner_core = Arc::new(RwLock::new(RunnerCore::new(BarefootConfig::default())));
        let health_tool = HealthCheckTool::new(runner_core.clone());
        
        let args = serde_json::json!({
            "detailed": true
        });
        
        let result = health_tool.execute(args).await.unwrap();
        assert!(result.success);
        assert!(result.content.is_object());
    }
    
    #[tokio::test]
    async fn test_tool_manager() {
        let runner_core = Arc::new(RwLock::new(RunnerCore::new(BarefootConfig::default())));
        let mut manager = ToolManager::new();
        manager.register_async_tool(AsyncToolHandler::StartJob(StartJobTool::new(runner_core.clone())));
        manager.register_async_tool(AsyncToolHandler::StopJob(StopJobTool::new(runner_core.clone())));
        manager.register_async_tool(AsyncToolHandler::HealthCheck(HealthCheckTool::new(runner_core.clone())));
        let valid_uuid = uuid::Uuid::new_v4().to_string();
        let result = manager.execute_tool("start_job", serde_json::json!({"job_id": valid_uuid, "priority": 5})).await.unwrap();
        assert!(result.success);
    }
} 