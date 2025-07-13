//! MCP server implementation for barefoot runner

use super::*;
use crate::core::RunnerCore;
use crate::error::{BarefootError, Result};
use crate::service::ServiceClientFactory;
use chrono::Utc;
use serde_json::Value;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};

/// MCP server for barefoot runner
pub struct BarefootMcpServer {
    /// Server configuration
    config: McpConfig,
    /// Pretty print tool results for debugging
    pretty_print_results: bool,
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
    /// Initialize status
    initialized: Arc<RwLock<bool>>,
    /// Pending messages queue
    pending_messages: Arc<RwLock<VecDeque<Value>>>,
}

impl Clone for BarefootMcpServer {
    fn clone(&self) -> Self {
        // Create a new server with the same configuration but fresh tool manager
        let mut new_server = Self::new(self.config.clone());

        // Copy over the shared state
        new_server.runner_core = self.runner_core.clone();
        new_server.service_factory = self.service_factory.clone();
        new_server.start_time = self.start_time;
        new_server.connections = self.connections.clone();
        new_server.last_error = self.last_error.clone();
        new_server.resource_manager = self.resource_manager.clone();
        new_server.initialized = self.initialized.clone();
        new_server.pending_messages = self.pending_messages.clone();

        // Re-register the async tools (which are cloneable)
        for (_name, tool) in self.tool_manager.async_tools() {
            new_server.tool_manager.register_async_tool(tool.clone());
        }

        new_server
    }
}

impl BarefootMcpServer {
    /// Create a new MCP server
    pub fn new(config: McpConfig) -> Self {
        let pretty_print_results = std::env::var("BAREFOOT_PRETTY_PRINT_RESULTS")
            .ok()
            .map_or(false, |v| v == "1" || v == "true");
        let runner_core = Arc::new(RwLock::new(RunnerCore::new(
            crate::config::BarefootConfig::default(),
        )));
        let service_factory = ServiceClientFactory;
        let start_time = Some(Utc::now());
        let connections = Arc::new(RwLock::new(0));
        let last_error = Arc::new(RwLock::new(None));
        let initialized = Arc::new(RwLock::new(false));
        let pending_messages = Arc::new(RwLock::new(VecDeque::new()));

        let mut resource_manager = resources::ResourceManager::new();
        let mut tool_manager = tools::ToolManager::new();

        // Register default resources
        let health_resource = resources::ResourceProvider::Health(resources::HealthResource::new(
            runner_core.clone(),
            start_time,
            last_error.clone(),
        ));
        let active_jobs_resource = resources::ResourceProvider::ActiveJobs(
            resources::ActiveJobsResource::new(runner_core.clone()),
        );
        let job_history_resource = resources::ResourceProvider::JobHistory(
            resources::JobHistoryResource::new(runner_core.clone()),
        );

        resource_manager.register_resource(health_resource);
        resource_manager.register_resource(active_jobs_resource);
        resource_manager.register_resource(job_history_resource);

        // Register default tools
        let start_tool =
            tools::AsyncToolHandler::StartJob(tools::StartJobTool::new(runner_core.clone()));
        let stop_tool =
            tools::AsyncToolHandler::StopJob(tools::StopJobTool::new(runner_core.clone()));
        let health_tool =
            tools::AsyncToolHandler::HealthCheck(tools::HealthCheckTool::new(runner_core.clone()));
        let weather_tool = tools::AsyncToolHandler::WeatherDashboard(
            tools::WeatherDashboardTool::new(runner_core.clone()),
        );
        let alerts_tool =
            tools::AsyncToolHandler::Alerts(tools::AlertsTool::new(runner_core.clone()));
        let graph_tool = tools::AsyncToolHandler::DependencyGraph(tools::DependencyGraphTool::new(
            runner_core.clone(),
        ));
        let analytics_tool =
            tools::AsyncToolHandler::Analytics(tools::AnalyticsTool::new(runner_core.clone()));

        tool_manager.register_async_tool(start_tool);
        tool_manager.register_async_tool(stop_tool);
        tool_manager.register_async_tool(health_tool);
        tool_manager.register_async_tool(weather_tool);
        tool_manager.register_async_tool(alerts_tool);
        tool_manager.register_async_tool(graph_tool);
        tool_manager.register_async_tool(analytics_tool);
        // Register new job tools
        let list_jobs_tool =
            tools::AsyncToolHandler::ListJobs(tools::ListJobsTool::new(runner_core.clone()));
        let job_history_tool =
            tools::AsyncToolHandler::JobHistory(tools::JobHistoryTool::new(runner_core.clone()));
        let job_logs_tool =
            tools::AsyncToolHandler::JobLogs(tools::JobLogsTool::new(runner_core.clone()));
        tool_manager.register_async_tool(list_jobs_tool);
        tool_manager.register_async_tool(job_history_tool);
        tool_manager.register_async_tool(job_logs_tool);

        Self {
            config,
            pretty_print_results,
            runner_core,
            service_factory,
            start_time,
            connections,
            last_error,
            resource_manager,
            tool_manager,
            initialized,
            pending_messages,
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

    /// Get transport configuration
    pub fn transport(&self) -> &TransportType {
        &self.config.transport
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
        let mut initialized = self.initialized.write().await;
        *initialized = true;
        Ok(())
    }

    /// Process a single message and return the response
    async fn process_message(&self, req: &Value, stdout: &mut tokio::io::Stdout) -> Option<Value> {
        if self.pretty_print_results {
            if let Ok(pretty) = serde_json::to_string_pretty(req) {
                println!("|--> request\n{}\n", pretty);
            }
        }
        let id = req.get("id").cloned();
        let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");

        // Log the method being processed
        let _ = self
            .send_info_log(&format!("Processing method '{}'", method), stdout)
            .await;

        let result = match method {
            "initialized" => {
                // Notification, no response
                let _ = self
                    .send_info_log("Received 'initialized' notification", stdout)
                    .await;
                None
            }
            "server_info" => {
                let _ = self
                    .send_info_log("Handling server_info request", stdout)
                    .await;
                let result = serde_json::json!({
                    "name": "barefoot-runner",
                    "version": "0.1.0",
                    "description": "Barefoot MCP server"
                });
                Some(serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id.clone().unwrap_or(Value::Null),
                    "result": result
                }))
            }
            "list_tools" => {
                let _ = self
                    .send_info_log("Handling list_tools request", stdout)
                    .await;
                let tools = self.tool_manager.list_tools();
                let result = serde_json::json!({
                    "tools": tools,
                    "serverInfo": {
                        "name": "barefoot-runner",
                        "version": "0.1.0",
                        "description": "Barefoot MCP server"
                    }
                });
                Some(serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id.clone().unwrap_or(Value::Null),
                    "result": result
                }))
            }
            "list_offerings" => {
                let _ = self
                    .send_info_log("Handling list_offerings request", stdout)
                    .await;
                let tools = self.tool_manager.list_tools();
                let result = serde_json::json!({
                    "tools": tools,
                    "serverInfo": {
                        "name": "barefoot-runner",
                        "version": "0.1.0",
                        "description": "Barefoot MCP server"
                    }
                });
                Some(serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id.clone().unwrap_or(Value::Null),
                    "result": result
                }))
            }
            "call_tool" | "tools/call" => {
                let _ = self
                    .send_info_log("Handling call_tool/tools/call request", stdout)
                    .await;
                let params = req.get("params").cloned().unwrap_or(Value::Null);
                // Accept both 'tool'/'args' and 'name'/'arguments'
                let tool_name = params
                    .get("tool")
                    .or_else(|| params.get("name"))
                    .and_then(|t| t.as_str());
                let args = params
                    .get("args")
                    .or_else(|| params.get("arguments"))
                    .cloned()
                    .unwrap_or(Value::Null);

                if let Some(tool_name) = tool_name {
                    let _ = self
                        .send_info_log(&format!("Calling tool '{}'", tool_name), stdout)
                        .await;
                    // For demo, only support 'health_check' as a stub
                    if tool_name == "health_check" {
                        let result = serde_json::json!({
                            "success": true,
                            "content": { "status": "ok" },
                            "error": null,
                            "duration": 1
                        });
                        Some(serde_json::json!({
                            "jsonrpc": "2.0",
                            "id": id.clone().unwrap_or(Value::Null),
                            "result": result
                        }))
                    } else {
                        let error_msg = format!("Tool '{}' not found", tool_name);
                        let _ = self.send_error_log(&error_msg, stdout).await;
                        Some(serde_json::json!({
                            "jsonrpc": "2.0",
                            "id": id.clone().unwrap_or(Value::Null),
                            "error": {
                                "code": -32601,
                                "message": error_msg
                            }
                        }))
                    }
                } else {
                    let error_msg = "Missing tool name in call_tool params".to_string();
                    let _ = self.send_error_log(&error_msg, stdout).await;
                    Some(serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": id.clone().unwrap_or(Value::Null),
                        "error": {
                            "code": -32602,
                            "message": error_msg
                        }
                    }))
                }
            }
            "list_resources" => {
                let _ = self
                    .send_info_log("Handling list_resources request", stdout)
                    .await;
                // Stub: return empty list
                let result = serde_json::json!({ "resources": [] });
                Some(serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id.clone().unwrap_or(Value::Null),
                    "result": result
                }))
            }
            "get_resource" => {
                let _ = self
                    .send_info_log("Handling get_resource request", stdout)
                    .await;
                // Stub: return not found
                let error_msg = "Resource not found".to_string();
                let _ = self.send_error_log(&error_msg, stdout).await;
                Some(serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id.clone().unwrap_or(Value::Null),
                    "error": {
                        "code": -32602,
                        "message": error_msg
                    }
                }))
            }
            "list_prompts" => {
                let _ = self
                    .send_info_log("Handling list_prompts request", stdout)
                    .await;
                // Stub: return empty list
                let result = serde_json::json!({ "prompts": [] });
                Some(serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id.clone().unwrap_or(Value::Null),
                    "result": result
                }))
            }
            "get_prompt" => {
                let _ = self
                    .send_info_log("Handling get_prompt request", stdout)
                    .await;
                // Stub: return not found
                let error_msg = "Prompt not found".to_string();
                let _ = self.send_error_log(&error_msg, stdout).await;
                Some(serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id.clone().unwrap_or(Value::Null),
                    "error": {
                        "code": -32602,
                        "message": error_msg
                    }
                }))
            }
            "shutdown" | "exit" => {
                let _ = self
                    .send_info_log("Handling shutdown/exit request", stdout)
                    .await;
                // Respond with success
                let result = serde_json::json!({ "success": true });
                Some(serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id.clone().unwrap_or(Value::Null),
                    "result": result
                }))
            }
            "" if id.is_none() => {
                // Notification (e.g., 'initialized'), no response
                let _ = self
                    .send_info_log("Received notification (no response needed)", stdout)
                    .await;
                None
            }
            _ => {
                let error_msg = format!("Method '{}' not found", method);
                let _ = self.send_error_log(&error_msg, stdout).await;
                Some(serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id.clone().unwrap_or(Value::Null),
                    "error": {
                        "code": -32601,
                        "message": error_msg
                    }
                }))
            }
        };

        // Log the result
        match &result {
            Some(response) => {
                if response.get("error").is_some() {
                    let _ = self
                        .send_error_log("Returning error response", stdout)
                        .await;
                } else {
                    let _ = self
                        .send_info_log("Returning success response", stdout)
                        .await;
                }
            }
            None => {
                let _ = self
                    .send_info_log("No response (notification)", stdout)
                    .await;
            }
        }

        if let Some(resp) = &result {
            if self.pretty_print_results {
                if let Ok(pretty) = serde_json::to_string_pretty(resp) {
                    println!("|<-- response\n{}\n", pretty);
                }
            }
        }
        result
    }

    /// Start the server
    pub async fn start(&mut self) -> Result<()> {
        use serde_json::Value;
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

        let stdin = BufReader::new(tokio::io::stdin());
        let mut lines = stdin.lines();
        let mut stdout = tokio::io::stdout();

        // Send startup log message
        self.send_info_log("MCP server starting", &mut stdout)
            .await?;

        while let Some(line) = lines.next_line().await? {
            eprintln!("MCP IN: {}", line);

            // Handle JSON parsing errors explicitly
            let req = match serde_json::from_str::<Value>(&line) {
                Ok(req) => req,
                Err(e) => {
                    let error_msg = format!("Failed to parse JSON: {}", e);
                    self.send_error_log(&error_msg, &mut stdout).await?;

                    // Send error response to client
                    let error_response = serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": null,
                        "error": {
                            "code": -32700,
                            "message": error_msg
                        }
                    });

                    let resp_str = error_response.to_string();
                    eprintln!("MCP OUT: {}", resp_str);
                    if let Err(write_err) = stdout.write_all(resp_str.as_bytes()).await {
                        eprintln!("MCP ERROR: Failed to write error response: {}", write_err);
                    }
                    if let Err(write_err) = stdout.write_all(b"\n").await {
                        eprintln!("MCP ERROR: Failed to write newline: {}", write_err);
                    }
                    if let Err(flush_err) = stdout.flush().await {
                        eprintln!("MCP ERROR: Failed to flush stdout: {}", flush_err);
                    }
                    continue;
                }
            };

            let id = req.get("id").cloned();
            let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");

            // Validate required JSON-RPC fields
            if req.get("jsonrpc").and_then(|v| v.as_str()) != Some("2.0") {
                let error_msg = "Invalid JSON-RPC version. Expected '2.0'".to_string();
                self.send_error_log(&error_msg, &mut stdout).await?;

                let error_response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id.clone().unwrap_or(Value::Null),
                    "error": {
                        "code": -32600,
                        "message": error_msg
                    }
                });

                let resp_str = error_response.to_string();
                eprintln!("MCP OUT: {}", resp_str);
                if let Err(write_err) = stdout.write_all(resp_str.as_bytes()).await {
                    eprintln!("MCP ERROR: Failed to write error response: {}", write_err);
                }
                if let Err(write_err) = stdout.write_all(b"\n").await {
                    eprintln!("MCP ERROR: Failed to write newline: {}", write_err);
                }
                if let Err(flush_err) = stdout.flush().await {
                    eprintln!("MCP ERROR: Failed to flush stdout: {}", flush_err);
                }
                continue;
            }

            // Check if this is an initialize method
            let is_initialize = method == "initialize";

            // If not initialized and not an initialize method, queue it
            if !is_initialize {
                let initialized = *self.initialized.read().await;
                if !initialized {
                    let queue_msg = format!(
                        "Queuing non-initialize method '{}' until after initialize",
                        method
                    );
                    self.send_warning_log(&queue_msg, &mut stdout).await?;
                    let mut pending = self.pending_messages.write().await;
                    pending.push_back(req.clone());
                    continue;
                }
            }

            let response = if is_initialize {
                let protocol_version = req
                    .get("params")
                    .and_then(|p| p.get("protocolVersion"))
                    .cloned()
                    .unwrap_or(Value::String("2025-03-26".to_string()));
                let result = serde_json::json!({
                    "serverInfo": {
                        "name": "barefoot-runner",
                        "version": "0.1.0",
                        "description": "Barefoot MCP server"
                    },
                    "capabilities": {
                        "tools": true,
                        "prompts": false,
                        "resources": false,
                        "logging": false
                    },
                    "protocolVersion": protocol_version
                });

                // Mark as initialized
                {
                    let mut initialized = self.initialized.write().await;
                    *initialized = true;
                }

                // Add a delay to ensure initialize response is processed
                self.send_info_log(
                    "Initialize complete, adding delay for race condition workaround",
                    &mut stdout,
                )
                .await?;
                sleep(Duration::from_millis(4000)).await;

                // Process any pending messages
                {
                    let mut pending = self.pending_messages.write().await;
                    while let Some(pending_req) = pending.pop_front() {
                        self.send_info_log("Processing queued message", &mut stdout)
                            .await?;
                        if let Some(response) =
                            self.process_message(&pending_req, &mut stdout).await
                        {
                            let resp_str = response.to_string();
                            eprintln!("MCP OUT: {}", resp_str);
                            if let Err(write_err) = stdout.write_all(resp_str.as_bytes()).await {
                                eprintln!(
                                    "MCP ERROR: Failed to write queued response: {}",
                                    write_err
                                );
                            }
                            if let Err(write_err) = stdout.write_all(b"\n").await {
                                eprintln!("MCP ERROR: Failed to write newline: {}", write_err);
                            }
                            if let Err(flush_err) = stdout.flush().await {
                                eprintln!("MCP ERROR: Failed to flush stdout: {}", flush_err);
                            }
                        }
                    }
                }

                Some(serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id.clone().unwrap_or(Value::Null),
                    "result": result
                }))
            } else {
                // Process non-initialize messages normally
                self.process_message(&req, &mut stdout).await
            };

            if let Some(resp) = response {
                let resp_str = resp.to_string();
                eprintln!("MCP OUT: {}", resp_str);
                if let Err(write_err) = stdout.write_all(resp_str.as_bytes()).await {
                    eprintln!("MCP ERROR: Failed to write response: {}", write_err);
                }
                if let Err(write_err) = stdout.write_all(b"\n").await {
                    eprintln!("MCP ERROR: Failed to write newline: {}", write_err);
                }
                if let Err(flush_err) = stdout.flush().await {
                    eprintln!("MCP ERROR: Failed to flush stdout: {}", flush_err);
                }
            }
        }

        self.send_info_log("MCP server stopping", &mut stdout)
            .await?;
        Ok(())
    }

    /// Start the server with HTTP transport
    pub async fn start_http(&mut self, host: &str, port: u16) -> Result<()> {
        use axum::routing::{get, post};
        use std::net::SocketAddr;
        use tower_http::cors::{Any, CorsLayer};

        // Create CORS layer
        let cors = CorsLayer::new()
            .allow_methods([axum::http::Method::GET, axum::http::Method::POST])
            .allow_origin(Any);

        // Create a shared server instance for the handler
        let server = Arc::new(self.clone());

        // Create router
        let app = axum::Router::new()
            .route("/health", get(|| async { "OK" }))
            .route(
                "/mcp",
                post(move |axum::Json(request): axum::Json<serde_json::Value>| {
                    let server = server.clone();
                    async move { server.handle_mcp_http_request(request).await }
                }),
            )
            .layer(cors);

        let addr = format!("{}:{}", host, port).parse::<SocketAddr>()?;

        self.send_info_log(
            &format!("Starting HTTP MCP server on {}:{}", host, port),
            &mut tokio::io::stdout(),
        )
        .await?;

        let listener = tokio::net::TcpListener::bind(&addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }

    /// Handle MCP HTTP request
    async fn handle_mcp_http_request(
        &self,
        request: serde_json::Value,
    ) -> axum::Json<serde_json::Value> {
        if self.pretty_print_results {
            if let Ok(pretty) = serde_json::to_string_pretty(&request) {
                println!("|--> request\n{}\n", pretty);
            }
        }
        let id = request.get("id").cloned();
        let method = request.get("method").and_then(|m| m.as_str()).unwrap_or("");

        // Log the incoming request
        let _ = self
            .send_info_log(
                &format!("HTTP MCP request: {}", method),
                &mut tokio::io::stdout(),
            )
            .await;

        // Validate JSON-RPC version
        if request.get("jsonrpc").and_then(|v| v.as_str()) != Some("2.0") {
            let error_msg = "Invalid JSON-RPC version. Expected '2.0'".to_string();
            let _ = self
                .send_error_log(&error_msg, &mut tokio::io::stdout())
                .await;

            return axum::Json(serde_json::json!({
                "jsonrpc": "2.0",
                "id": id.clone().unwrap_or(serde_json::Value::Null),
                "error": {
                    "code": -32600,
                    "message": error_msg
                }
            }));
        }

        // Process the request
        let response = match method {
            "initialize" => {
                let protocol_version = request
                    .get("params")
                    .and_then(|p| p.get("protocolVersion"))
                    .cloned()
                    .unwrap_or(serde_json::Value::String("2025-03-26".to_string()));

                // Return the exact format Cursor expects
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id.clone().unwrap_or(serde_json::Value::Null),
                    "result": {
                        "protocolVersion": protocol_version,
                        "capabilities": {
                            "tools": {},
                            "prompts": {},
                            "resources": {},
                            "logging": {}
                        },
                        "serverInfo": {
                            "name": "barefoot-runner",
                            "version": "0.1.0",
                            "description": "Barefoot MCP server for CI/CD and job management"
                        }
                    }
                })
            }
            "notifications/initialized" => {
                // This is a notification, no response needed
                let _ = self
                    .send_info_log(
                        "Received initialized notification",
                        &mut tokio::io::stdout(),
                    )
                    .await;
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id.clone().unwrap_or(serde_json::Value::Null),
                    "result": null
                })
            }
            "list_tools" | "tools/list" => {
                let tools = self.tool_manager.list_tools();
                let tool_definitions = tools
                    .into_iter()
                    .map(|tool| {
                        serde_json::json!({
                            "name": tool.name,
                            "description": tool.description,
                            "inputSchema": tool.input_schema,
                            "permissions": tool.permissions
                        })
                    })
                    .collect::<Vec<_>>();

                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id.clone().unwrap_or(serde_json::Value::Null),
                    "result": {
                        "tools": tool_definitions
                    }
                })
            }
            "list_offerings" => {
                let tools = self.tool_manager.list_tools();
                let tool_definitions = tools
                    .into_iter()
                    .map(|tool| {
                        serde_json::json!({
                            "name": tool.name,
                            "description": tool.description,
                            "inputSchema": tool.input_schema,
                            "permissions": tool.permissions
                        })
                    })
                    .collect::<Vec<_>>();

                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id.clone().unwrap_or(serde_json::Value::Null),
                    "result": {
                        "tools": tool_definitions
                    }
                })
            }
            "call_tool" | "tools/call" => {
                let params = request
                    .get("params")
                    .cloned()
                    .unwrap_or(serde_json::Value::Null);
                // Accept both 'tool'/'args' and 'name'/'arguments'
                let tool_name = params
                    .get("tool")
                    .or_else(|| params.get("name"))
                    .and_then(|t| t.as_str());
                let _args = params
                    .get("args")
                    .or_else(|| params.get("arguments"))
                    .cloned()
                    .unwrap_or(serde_json::Value::Null);

                if let Some(tool_name) = tool_name {
                    let _ = self
                        .send_info_log(
                            &format!("HTTP calling tool '{}'", tool_name),
                            &mut tokio::io::stdout(),
                        )
                        .await;

                    // Try to execute the actual tool
                    match self.tool_manager.execute_tool(tool_name, _args).await {
                        Ok(result) => {
                            serde_json::json!({
                                "jsonrpc": "2.0",
                                "id": id.clone().unwrap_or(serde_json::Value::Null),
                                "result": {
                                    "content": result.content,
                                    "isError": !result.success,
                                    "error": result.error
                                }
                            })
                        }
                        Err(e) => {
                            let error_msg = format!("Tool execution failed: {}", e);
                            let _ = self
                                .send_error_log(&error_msg, &mut tokio::io::stdout())
                                .await;
                            serde_json::json!({
                                "jsonrpc": "2.0",
                                "id": id.clone().unwrap_or(serde_json::Value::Null),
                                "error": {
                                    "code": -32603,
                                    "message": error_msg
                                }
                            })
                        }
                    }
                } else {
                    let error_msg = "Missing tool name in call_tool params".to_string();
                    let _ = self
                        .send_error_log(&error_msg, &mut tokio::io::stdout())
                        .await;
                    serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": id.clone().unwrap_or(serde_json::Value::Null),
                        "error": {
                            "code": -32602,
                            "message": error_msg
                        }
                    })
                }
            }
            _ => {
                let error_msg = format!("Method '{}' not found", method);
                let _ = self
                    .send_error_log(&error_msg, &mut tokio::io::stdout())
                    .await;
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id.clone().unwrap_or(serde_json::Value::Null),
                    "error": {
                        "code": -32601,
                        "message": error_msg
                    }
                })
            }
        };

        if self.pretty_print_results {
            if let Ok(pretty) = serde_json::to_string_pretty(&response) {
                println!("|<-- response\n{}\n", pretty);
            }
        }
        axum::Json(response)
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

        let uptime = self
            .start_time
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
                tool_result.duration =
                    Some(chrono::Duration::from_std(duration).unwrap_or_default());
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

    /// Send a log message to the client via JSON-RPC notification
    async fn send_log_message(
        &self,
        level: &str,
        message: &str,
        stdout: &mut tokio::io::Stdout,
    ) -> Result<()> {
        let log_notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "logging",
            "params": {
                "level": level,
                "message": message,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }
        });

        let resp_str = log_notification.to_string();
        eprintln!("MCP LOG [{}]: {}", level.to_uppercase(), message);

        if let Err(write_err) = stdout.write_all(resp_str.as_bytes()).await {
            eprintln!("MCP ERROR: Failed to write log message: {}", write_err);
        }
        if let Err(write_err) = stdout.write_all(b"\n").await {
            eprintln!("MCP ERROR: Failed to write newline: {}", write_err);
        }
        if let Err(flush_err) = stdout.flush().await {
            eprintln!("MCP ERROR: Failed to flush stdout: {}", flush_err);
        }

        Ok(())
    }

    /// Send an info log message to the client
    async fn send_info_log(&self, message: &str, stdout: &mut tokio::io::Stdout) -> Result<()> {
        self.send_log_message("info", message, stdout).await
    }

    /// Send a warning log message to the client
    async fn send_warning_log(&self, message: &str, stdout: &mut tokio::io::Stdout) -> Result<()> {
        self.send_log_message("warning", message, stdout).await
    }

    /// Send an error log message to the client
    async fn send_error_log(&self, message: &str, stdout: &mut tokio::io::Stdout) -> Result<()> {
        self.send_log_message("error", message, stdout).await
    }
}

#[cfg(test)]
mod tests {
    // use super::*; // Unused import
    use serde_json::json;
    use tokio::io::{duplex, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt};

    async fn run_protocol_once(input: &str) -> serde_json::Value {
        let (mut client_write, server_read) = duplex(1024);
        let (server_write, mut client_read) = duplex(1024);
        client_write.write_all(input.as_bytes()).await.unwrap();
        // Simulate a single protocol step
        let mut lines = tokio::io::BufReader::new(server_read).lines();
        let mut stdout = server_write;
        if let Ok(Some(line)) = lines.next_line().await {
            if let Ok(req) = serde_json::from_str::<serde_json::Value>(&line) {
                let id = req.get("id").cloned().unwrap_or(serde_json::Value::Null);
                let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");
                let response = if method == "initialize" {
                    let protocol_version = req
                        .get("params")
                        .and_then(|p| p.get("protocolVersion"))
                        .cloned()
                        .unwrap_or(serde_json::Value::String("2025-03-26".to_string()));
                    let result = json!({
                        "serverInfo": {
                            "name": "barefoot-runner",
                            "version": "0.1.0",
                            "description": "Barefoot MCP server"
                        },
                        "capabilities": {
                            "tools": true,
                            "prompts": false,
                            "resources": false,
                            "logging": false
                        },
                        "protocolVersion": protocol_version
                    });
                    json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": result
                    })
                } else if method == "list_tools" {
                    let result = json!({ "tools": [] });
                    json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": result
                    })
                } else if method == "list_offerings" {
                    let result = json!({ "tools": [] });
                    json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": result
                    })
                } else if method == "server_info" {
                    let result = json!({
                        "name": "barefoot-runner",
                        "version": "0.1.0",
                        "description": "Barefoot MCP server"
                    });
                    json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": result
                    })
                } else if req.get("id").is_some() {
                    json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "error": {
                            "code": -32601,
                            "message": format!("Method '{}' not found", method)
                        }
                    })
                } else {
                    // Notification: no response
                    return serde_json::Value::Null;
                };
                let resp_str = response.to_string();
                stdout.write_all(resp_str.as_bytes()).await.unwrap();
                stdout.write_all(b"\n").await.unwrap();
                stdout.flush().await.unwrap();
            }
        }
        let mut buf = vec![0; 1024];
        let n = client_read.read(&mut buf).await.unwrap();
        if n == 0 {
            serde_json::Value::Null
        } else {
            serde_json::from_slice(&buf[..n]).unwrap()
        }
    }

    #[tokio::test]
    async fn test_initialize_happy_path() {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 42,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-06-18",
                "capabilities": { "tools": true }
            }
        })
        .to_string()
            + "\n";
        let response = run_protocol_once(&request).await;
        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["id"], 42);
        assert_eq!(response["result"]["protocolVersion"], "2025-06-18");
        assert_eq!(response["result"]["serverInfo"]["name"], "barefoot-runner");
        assert_eq!(
            response["result"]["serverInfo"]["description"],
            "Barefoot MCP server"
        );
    }

    #[tokio::test]
    async fn test_initialize_missing_protocol_version() {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 7,
            "method": "initialize",
            "params": {
                "capabilities": { "tools": true }
            }
        })
        .to_string()
            + "\n";
        let response = run_protocol_once(&request).await;
        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["id"], 7);
        assert_eq!(response["result"]["protocolVersion"], "2025-03-26");
    }

    #[tokio::test]
    async fn test_list_tools() {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "list_tools"
        })
        .to_string()
            + "\n";
        let response = run_protocol_once(&request).await;
        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["id"], 1);
        assert!(response["result"]["tools"].is_array());
    }

    #[tokio::test]
    async fn test_list_offerings() {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "list_offerings"
        })
        .to_string()
            + "\n";
        let response = run_protocol_once(&request).await;
        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["id"], 2);
        assert!(response["result"]["tools"].is_array());
    }

    #[tokio::test]
    async fn test_server_info() {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "server_info"
        })
        .to_string()
            + "\n";
        let response = run_protocol_once(&request).await;
        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["id"], 3);
        assert_eq!(response["result"]["name"], "barefoot-runner");
        assert_eq!(response["result"]["description"], "Barefoot MCP server");
    }

    #[tokio::test]
    async fn test_unknown_method_error() {
        let request = json!({
            "jsonrpc": "2.0",
            "id": 99,
            "method": "not_a_real_method"
        })
        .to_string()
            + "\n";
        let response = run_protocol_once(&request).await;
        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["id"], 99);
        assert!(response["error"].is_object());
        assert_eq!(response["error"]["code"], -32601);
    }

    #[tokio::test]
    async fn test_notification_no_response() {
        let request = json!({
            "jsonrpc": "2.0",
            "method": "initialized"
        })
        .to_string()
            + "\n";
        let response = run_protocol_once(&request).await;
        assert_eq!(response, serde_json::Value::Null);
    }
}
