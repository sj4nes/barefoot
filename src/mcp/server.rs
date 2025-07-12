//! MCP server implementation for barefoot runner

use super::*;
use crate::{core::RunnerCore, service::ServiceClientFactory};
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::Utc;
use crate::error::Result;

/// MCP server for barefoot runner
pub struct BarefootMcpServer {
    /// Server configuration
    config: McpConfig,
    /// Runner core reference
    runner_core: Arc<RwLock<RunnerCore>>,
    /// Service client factory
    service_factory: ServiceClientFactory,
    /// Server start time
    start_time: Option<chrono::DateTime<Utc>>,
    /// Active connections
    connections: Arc<RwLock<usize>>,
    /// Last error
    last_error: Arc<RwLock<Option<String>>>,
    /// Resource manager
    resource_manager: resources::ResourceManager,
    /// Tool manager
    tool_manager: tools::ToolManager,
}

impl BarefootMcpServer {
    /// Create a new MCP server
    pub fn new(config: McpConfig) -> Self {
        let runner_core = Arc::new(RwLock::new(RunnerCore::new(crate::config::BarefootConfig::default())));
        let service_factory = ServiceClientFactory;
        let start_time = Some(Utc::now());
        let connections = Arc::new(RwLock::new(0));
        let last_error = Arc::new(RwLock::new(None));
        
        let mut resource_manager = resources::ResourceManager::new();
        let mut tool_manager = tools::ToolManager::new();
        
        // Register default resources
        let health_resource = resources::ResourceProvider::Health(
            resources::HealthResource::new(runner_core.clone(), start_time, last_error.clone())
        );
        let active_jobs_resource = resources::ResourceProvider::ActiveJobs(
            resources::ActiveJobsResource::new(runner_core.clone())
        );
        let job_history_resource = resources::ResourceProvider::JobHistory(
            resources::JobHistoryResource::new(runner_core.clone())
        );
        
        resource_manager.register_resource(health_resource);
        resource_manager.register_resource(active_jobs_resource);
        resource_manager.register_resource(job_history_resource);
        
        // Register default tools
        let start_tool = tools::AsyncToolHandler::StartJob(
            tools::StartJobTool::new(runner_core.clone())
        );
        let stop_tool = tools::AsyncToolHandler::StopJob(
            tools::StopJobTool::new(runner_core.clone())
        );
        let health_tool = tools::AsyncToolHandler::HealthCheck(
            tools::HealthCheckTool::new(runner_core.clone())
        );
        
        tool_manager.register_async_tool(start_tool);
        tool_manager.register_async_tool(stop_tool);
        tool_manager.register_async_tool(health_tool);
        
        Self {
            config,
            runner_core,
            service_factory,
            start_time,
            connections,
            last_error,
            resource_manager,
            tool_manager,
        }
    }
    
    /// Get runner core reference
    pub fn runner_core(&self) -> Arc<RwLock<RunnerCore>> {
        self.runner_core.clone()
    }
    
    /// Get service factory reference
    pub fn service_factory(&self) -> &ServiceClientFactory {
        &self.service_factory
    }
    
    /// Update last error
    async fn update_last_error(&self, error: String) {
        let mut last_error = self.last_error.write().await;
        *last_error = Some(error);
    }
    
    /// Increment connection count
    pub async fn increment_connections(&self) {
        let mut connections = self.connections.write().await;
        *connections += 1;
    }
    
    /// Decrement connection count
    pub async fn decrement_connections(&self) {
        let mut connections = self.connections.write().await;
        if *connections > 0 {
            *connections -= 1;
        }
    }
    
    /// Initialize the server
    pub async fn initialize(&mut self, config: McpConfig) -> Result<()> {
        self.config = config;
        self.start_time = Some(Utc::now());
        Ok(())
    }
    
    /// Start the server
    pub async fn start(&mut self) -> Result<()> {
        tracing::info!("Starting MCP server");
        self.start_time = Some(Utc::now());
        Ok(())
    }
    
    /// Stop the server
    pub async fn stop(&mut self) -> Result<()> {
        tracing::info!("Stopping MCP server");
        Ok(())
    }
    
    /// Get server status
    pub async fn status(&self) -> Result<ServerStatus> {
        let connections = *self.connections.read().await;
        let last_error = self.last_error.read().await.clone();
        
        let uptime = self.start_time
            .map(|start| Utc::now().signed_duration_since(start))
            .unwrap_or_default();
        
        Ok(ServerStatus {
            running: true,
            connections,
            uptime,
            resource_count: self.resource_manager.list_resources().len(),
            tool_count: self.tool_manager.list_tools().len(),
            last_error,
        })
    }
    
    /// List available resources
    pub async fn list_resources(&self) -> Result<Vec<ResourceId>> {
        Ok(self.resource_manager.list_resources())
    }
    
    /// Get resource content
    pub async fn get_resource(&self, uri: &str) -> Result<ResourceContent> {
        self.resource_manager.get_resource(uri).await
    }
    
    /// List available tools
    pub async fn list_tools(&self) -> Result<Vec<ToolDefinition>> {
        Ok(self.tool_manager.list_tools())
    }
    
    /// Execute a tool
    pub async fn execute_tool(&self, name: &str, args: serde_json::Value) -> Result<ToolResult> {
        let start_time = std::time::Instant::now();
        
        let result = self.tool_manager.execute_tool(name, args).await;
        
        let duration = start_time.elapsed();
        
        match result {
            Ok(mut tool_result) => {
                tool_result.duration = Some(chrono::Duration::from_std(duration).unwrap_or_default());
                Ok(tool_result)
            }
            Err(e) => {
                self.update_last_error(e.to_string()).await;
                Ok(ToolResult {
                    success: false,
                    content: serde_json::json!({}),
                    error: Some(e.to_string()),
                    duration: Some(chrono::Duration::from_std(duration).unwrap_or_default()),
                })
            }
        }
    }
    
    /// List available prompts
    pub async fn list_prompts(&self) -> Result<Vec<PromptTemplate>> {
        let prompts = vec![
            PromptTemplate {
                name: "troubleshooting".to_string(),
                description: "Troubleshoot job execution issues".to_string(),
                content: "Analyze the job logs and identify potential issues with the execution. Consider common problems like missing dependencies, configuration errors, or resource constraints.".to_string(),
                variables: vec!["job_id".to_string(), "error_message".to_string()],
                category: "debugging".to_string(),
            },
            PromptTemplate {
                name: "optimization".to_string(),
                description: "Optimize job performance".to_string(),
                content: "Review the job configuration and execution patterns to suggest optimizations for better performance and resource utilization.".to_string(),
                variables: vec!["job_id".to_string(), "execution_time".to_string()],
                category: "performance".to_string(),
            },
            PromptTemplate {
                name: "scheduling".to_string(),
                description: "Schedule job execution".to_string(),
                content: "Help schedule jobs based on dependencies, resource availability, and priority requirements.".to_string(),
                variables: vec!["job_list".to_string(), "constraints".to_string()],
                category: "planning".to_string(),
            },
        ];
        
        Ok(prompts)
    }
    
    /// Get prompt template
    pub async fn get_prompt(&self, name: &str) -> Result<PromptTemplate> {
        let prompts = self.list_prompts().await?;
        
        prompts
            .into_iter()
            .find(|p| p.name == name)
            .ok_or_else(|| BarefootError::Mcp(format!("Prompt not found: {}", name)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_mcp_server_creation() {
        let config = McpConfig::default();
        let server = BarefootMcpServer::new(config);
        
        assert_eq!(server.config.transport, TransportType::Stdio);
    }
    
    #[tokio::test]
    async fn test_mcp_server_status() {
        let config = McpConfig::default();
        let server = BarefootMcpServer::new(config);
        
        let status = server.status().await.unwrap();
        assert!(status.running);
        assert_eq!(status.connections, 0);
        assert_eq!(status.resource_count, 3);
        assert_eq!(status.tool_count, 3);
    }
    
    #[tokio::test]
    async fn test_mcp_server_resources() {
        let config = McpConfig::default();
        let server = BarefootMcpServer::new(config);
        
        let resources = server.list_resources().await.unwrap();
        assert_eq!(resources.len(), 3);
        
        let health_resource = server.get_resource("barefoot://runner/health").await.unwrap();
        assert_eq!(health_resource.id.uri, "barefoot://runner/health");
    }
    
    #[tokio::test]
    async fn test_mcp_server_tools() {
        let config = McpConfig::default();
        let server = BarefootMcpServer::new(config);
        
        let tools = server.list_tools().await.unwrap();
        assert_eq!(tools.len(), 3);
        
        let result = server.execute_tool("health_check", serde_json::json!({})).await.unwrap();
        assert!(result.success);
    }
} 