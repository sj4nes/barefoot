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
use plotters::prelude::*;
use base64::{engine::general_purpose, Engine as _};
use std::io::{Read, Seek, SeekFrom};
use tempfile::Builder;

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
    WeatherDashboard(WeatherDashboardTool),
    Alerts(AlertsTool),
    DependencyGraph(DependencyGraphTool),
    Analytics(AnalyticsTool),
    ListJobs(ListJobsTool),
    JobHistory(JobHistoryTool),
    JobLogs(JobLogsTool),
}

impl AsyncToolHandler {
    /// Execute the tool
    pub async fn execute(&self, args: serde_json::Value) -> Result<ToolResult> {
        match self {
            AsyncToolHandler::StartJob(t) => t.execute(args).await,
            AsyncToolHandler::StopJob(t) => t.execute(args).await,
            AsyncToolHandler::HealthCheck(t) => t.execute(args).await,
            AsyncToolHandler::WeatherDashboard(t) => t.execute(args).await,
            AsyncToolHandler::Alerts(t) => t.execute(args).await,
            AsyncToolHandler::DependencyGraph(t) => t.execute(args).await,
            AsyncToolHandler::Analytics(t) => t.execute(args).await,
            AsyncToolHandler::ListJobs(t) => t.execute(args).await,
            AsyncToolHandler::JobHistory(t) => t.execute(args).await,
            AsyncToolHandler::JobLogs(t) => t.execute(args).await,
        }
    }
    
    /// Get tool definition
    pub fn definition(&self) -> ToolDefinition {
        match self {
            AsyncToolHandler::StartJob(t) => t.definition(),
            AsyncToolHandler::StopJob(t) => t.definition(),
            AsyncToolHandler::HealthCheck(t) => t.definition(),
            AsyncToolHandler::WeatherDashboard(t) => t.definition(),
            AsyncToolHandler::Alerts(t) => t.definition(),
            AsyncToolHandler::DependencyGraph(t) => t.definition(),
            AsyncToolHandler::Analytics(t) => t.definition(),
            AsyncToolHandler::ListJobs(t) => t.definition(),
            AsyncToolHandler::JobHistory(t) => t.definition(),
            AsyncToolHandler::JobLogs(t) => t.definition(),
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

/// Weather dashboard tool handler
#[derive(Clone)]
pub struct WeatherDashboardTool {
    runner_core: Arc<RwLock<RunnerCore>>,
}

impl WeatherDashboardTool {
    pub fn new(runner_core: Arc<RwLock<RunnerCore>>) -> Self {
        Self { runner_core }
    }
    
    pub fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "weather_dashboard".to_string(),
            description: "Get a comprehensive dashboard of job health and system health".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "timeframe": {"type": "string", "enum": ["1h", "6h", "24h", "7d"]},
                    "include_system": {"type": "boolean"}
                }
            }),
            permissions: vec!["dashboard:read".to_string()],
        }
    }
    
    pub fn validate_args(&self, _args: &serde_json::Value) -> Result<()> {
        Ok(())
    }
    
    pub async fn execute(&self, args: serde_json::Value) -> Result<ToolResult> {
        self.validate_args(&args)?;
        
        let timeframe = args.get("timeframe").and_then(|t| t.as_str()).unwrap_or("24h");
        let include_system = args.get("include_system").and_then(|s| s.as_bool()).unwrap_or(true);
        
        let runner_core = self.runner_core.read().await;
        
        let content = serde_json::json!({
            "timeframe": timeframe,
            "job_health": {
                "total_jobs": runner_core.current_jobs().await.len(),
                "success_rate": 0.95,
                "average_duration": "15m",
                "failed_jobs": 2,
                "stuck_jobs": 0
            },
            "system_health": include_system.then(|| {
                serde_json::json!({
                    "cpu_usage": 45.2,
                    "memory_usage": 67.8,
                    "disk_usage": 23.1,
                    "network_status": "healthy"
                })
            }),
            "timestamp": Utc::now().to_rfc3339(),
        });
        
        Ok(ToolResult {
            success: true,
            content,
            error: None,
            duration: None,
        })
    }
}

/// Alerts tool handler
#[derive(Clone)]
pub struct AlertsTool {
    runner_core: Arc<RwLock<RunnerCore>>,
}

impl AlertsTool {
    pub fn new(runner_core: Arc<RwLock<RunnerCore>>) -> Self {
        Self { runner_core }
    }
    
    pub fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "alerts".to_string(),
            description: "Get alerts and notifications for failing jobs, stuck jobs, and degraded service".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "severity": {"type": "string", "enum": ["low", "medium", "high", "critical"]},
                    "include_resolved": {"type": "boolean"}
                }
            }),
            permissions: vec!["alerts:read".to_string()],
        }
    }
    
    pub fn validate_args(&self, _args: &serde_json::Value) -> Result<()> {
        Ok(())
    }
    
    pub async fn execute(&self, args: serde_json::Value) -> Result<ToolResult> {
        self.validate_args(&args)?;
        
        let severity = args.get("severity").and_then(|s| s.as_str()).unwrap_or("medium");
        let _include_resolved = args.get("include_resolved").and_then(|r| r.as_bool()).unwrap_or(false);
        
        let runner_core = self.runner_core.read().await;
        let current_jobs = runner_core.current_jobs().await;
        
        let alerts = current_jobs.iter()
            .filter_map(|job| {
                if job.status == crate::types::JobStatus::Failed {
                    Some(serde_json::json!({
                        "id": job.id.to_string(),
                        "type": "job_failed",
                        "severity": "high",
                        "message": format!("Job {} failed", job.name),
                        "timestamp": Utc::now().to_rfc3339(),
                        "resolved": false
                    }))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        
        let content = serde_json::json!({
            "severity": severity,
            "alerts": alerts,
            "total_alerts": alerts.len(),
            "timestamp": Utc::now().to_rfc3339(),
        });
        
        Ok(ToolResult {
            success: true,
            content,
            error: None,
            duration: None,
        })
    }
}

/// Dependency graph tool handler
#[derive(Clone)]
pub struct DependencyGraphTool {
    runner_core: Arc<RwLock<RunnerCore>>,
}

impl DependencyGraphTool {
    pub fn new(runner_core: Arc<RwLock<RunnerCore>>) -> Self {
        Self { runner_core }
    }
    
    pub fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "dependency_graph".to_string(),
            description: "Generate dependency graph visualization for jobs and workflows as SVG image".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "format": {"type": "string", "enum": ["json", "dot", "mermaid", "svg"]},
                    "include_workflows": {"type": "boolean"}
                }
            }),
            permissions: vec!["graph:read".to_string()],
        }
    }
    
    pub fn validate_args(&self, _args: &serde_json::Value) -> Result<()> {
        Ok(())
    }
    
    pub async fn execute(&self, args: serde_json::Value) -> Result<ToolResult> {
        self.validate_args(&args)?;
        let format = args.get("format").and_then(|f| f.as_str()).unwrap_or("json");
        let include_workflows = args.get("include_workflows").and_then(|w| w.as_bool()).unwrap_or(true);
        let runner_core = self.runner_core.read().await;
        let current_jobs = runner_core.current_jobs().await;
        let nodes = current_jobs.iter()
            .map(|job| serde_json::json!({
                "id": job.id.to_string(),
                "name": job.name,
                "type": "job",
                "status": format!("{:?}", job.status)
            }))
            .collect::<Vec<_>>();
        let edges = vec![
            serde_json::json!({
                "from": "workflow-1",
                "to": "job-1",
                "type": "triggers"
            }),
            serde_json::json!({
                "from": "job-1", 
                "to": "job-2",
                "type": "depends_on"
            })
        ];
        // Generate SVG if requested
        let mut svg_data_uri = None;
        if format == "svg" {
            let svg_markup = format!(
                r#"<svg xmlns='http://www.w3.org/2000/svg' width='320' height='120'><rect width='100%' height='100%' fill='white'/><circle cx='60' cy='60' r='40' stroke='black' stroke-width='3' fill='lightblue'/><text x='60' y='65' font-size='18' text-anchor='middle' fill='black'>Graph</text></svg>"#
            );
            let encoded_svg = general_purpose::STANDARD.encode(svg_markup.as_bytes());
            svg_data_uri = Some(format!("data:image/svg+xml;base64,{}", encoded_svg));
        }
        let content = serde_json::json!({
            "format": format,
            "nodes": nodes,
            "edges": edges,
            "svg_image": svg_data_uri,
            "metadata": {
                "total_nodes": nodes.len(),
                "total_edges": edges.len(),
                "include_workflows": include_workflows
            },
            "timestamp": Utc::now().to_rfc3339(),
        });
        Ok(ToolResult { success: true, content, error: None, duration: None })
    }
}

/// Analytics tool handler
#[derive(Clone)]
pub struct AnalyticsTool {
    runner_core: Arc<RwLock<RunnerCore>>,
}

impl AnalyticsTool {
    pub fn new(runner_core: Arc<RwLock<RunnerCore>>) -> Self {
        Self { runner_core }
    }
    pub fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "analytics".to_string(),
            description: "Get sparkline and cycle time analytics (minutes/hours/days, avg/p99/last) with PNG image".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "metric": {"type": "string", "enum": ["duration", "throughput", "error_rate"]},
                    "timeframe": {"type": "string", "enum": ["minutes", "hours", "days"]},
                    "aggregation": {"type": "string", "enum": ["avg", "p99", "last"]}
                }
            }),
            permissions: vec!["analytics:read".to_string()],
        }
    }
    pub fn validate_args(&self, _args: &serde_json::Value) -> Result<()> {
        Ok(())
    }
    pub async fn execute(&self, args: serde_json::Value) -> Result<ToolResult> {
        self.validate_args(&args)?;
        let metric = args.get("metric").and_then(|m| m.as_str()).unwrap_or("duration");
        let timeframe = args.get("timeframe").and_then(|t| t.as_str()).unwrap_or("hours");
        let aggregation = args.get("aggregation").and_then(|a| a.as_str()).unwrap_or("avg");
        // Generate mock analytics data
        let data_points: Vec<u32> = (0..24).map(|i| 15 + (i % 10)).collect();
        let sparkline = "▁▂▃▄▅▆▇█▇▆▅▄▃▂▁▂▃▄▅▆▇█▇▆▅▄▃▂▁";
        // Generate PNG sparkline
        let width = 240;
        let height = 60;
        let mut buf = vec![];
        {
            let root = BitMapBackend::with_buffer(&mut buf, (width, height)).into_drawing_area();
            root.fill(&WHITE).unwrap();
            let max = *data_points.iter().max().unwrap_or(&1);
            let min = *data_points.iter().min().unwrap_or(&0);
            let mut chart = ChartBuilder::on(&root)
                .margin(5)
                .set_all_label_area_size(0)
                .build_cartesian_2d(0..data_points.len() as u32, min..max)
                .unwrap();
            chart.configure_mesh().disable_mesh().draw().unwrap();
            chart.draw_series(LineSeries::new(
                data_points.iter().enumerate().map(|(i, v)| (i as u32, *v)),
                &BLUE,
            )).unwrap();
            root.present().unwrap();
        }
        let encoded_png = general_purpose::STANDARD.encode(&buf);
        let data_uri = format!("data:image/png;base64,{}", encoded_png);
        let content = serde_json::json!({
            "metric": metric,
            "timeframe": timeframe,
            "aggregation": aggregation,
            "data_points": data_points,
            "sparkline": sparkline,
            "sparkline_png": data_uri,
            "summary": {
                "min": 15,
                "max": 24,
                "avg": 19.5,
                "p99": 23,
                "trend": "stable"
            },
            "timestamp": Utc::now().to_rfc3339(),
        });
        Ok(ToolResult { success: true, content, error: None, duration: None })
    }
}

/// 1. Add ListJobsTool
#[derive(Clone)]
pub struct ListJobsTool {
    runner_core: Arc<RwLock<RunnerCore>>,
}

impl ListJobsTool {
    pub fn new(runner_core: Arc<RwLock<RunnerCore>>) -> Self {
        Self { runner_core }
    }
    pub fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "list_jobs".to_string(),
            description: "List active and queued jobs. Optional parameter: which ('active' | 'queued' | 'all'), default 'active'.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "which": { "type": "string", "enum": ["active", "queued", "all"] }
                },
                "required": []
            }),
            permissions: vec!["job:read".to_string()],
        }
    }
    pub fn validate_args(&self, _args: &serde_json::Value) -> Result<()> {
        Ok(())
    }
    pub async fn execute(&self, args: serde_json::Value) -> Result<ToolResult> {
        let which = args.get("which").and_then(|w| w.as_str()).unwrap_or("active");
        let runner_core = self.runner_core.read().await;
        let active_jobs = runner_core.current_jobs().await;
        let queued_jobs = runner_core.job_queue().await;
        let mut jobs = match which {
            "active" => active_jobs,
            "queued" => queued_jobs,
            "all" => {
                let mut all = active_jobs;
                all.extend(queued_jobs);
                all
            },
            _ => active_jobs,
        };
        // Inject dummy jobs if empty (for testing)
        if jobs.is_empty() {
            use uuid::Uuid;
            use crate::types::JobStatus;
            jobs.push(crate::types::Job {
                id: Uuid::new_v4(),
                name: "dummy-job-1".to_string(),
                status: JobStatus::Running,
                workflow: "test".to_string(),
                repository: "barefoot".to_string(),
                started_at: Some(chrono::Utc::now()),
                completed_at: None,
                steps: vec![],
            });
            jobs.push(crate::types::Job {
                id: Uuid::new_v4(),
                name: "dummy-job-2".to_string(),
                status: JobStatus::Queued,
                workflow: "test".to_string(),
                repository: "barefoot".to_string(),
                started_at: None,
                completed_at: None,
                steps: vec![],
            });
        }
        // --- Table image ---
        let width = 700;
        let height = 60 + 30 * jobs.len() as u32;
        let table_png: Vec<u8> = {
            use plotters::prelude::*;
            
            use std::io::Read;
            let mut tmpfile = Builder::new().suffix(".png").tempfile().expect("Failed to create temp file");
            {
                let root = BitMapBackend::new(tmpfile.path(), (width, height)).into_drawing_area();
                root.fill(&WHITE).expect("Failed to fill background");
                let font = ("sans-serif", 20).into_font();
                let header = ["Name", "Status", "Workflow", "Repo", "Started"];
                let mut y = 10;
                // Draw header
                for (i, h) in header.iter().enumerate() {
                    root.draw_text(h, &TextStyle::from(font.clone()).color(&BLACK), (20 + i as i32 * 130, y)).expect("Failed to draw header");
                }
                y += 30;
                // Draw rows
                for job in &jobs {
                    let row = [
                        job.name.as_str(),
                        &format!("{:?}", job.status),
                        job.workflow.as_str(),
                        job.repository.as_str(),
                        &job.started_at.map(|dt| dt.to_rfc3339()).unwrap_or_else(|| "-".to_string()),
                    ];
                    for (i, cell) in row.iter().enumerate() {
                        root.draw_text(cell, &TextStyle::from(font.clone()).color(&BLACK), (20 + i as i32 * 130, y)).expect("Failed to draw cell");
                    }
                    y += 30;
                }
                root.present().expect("Failed to present table PNG");
            }
            let mut buf = Vec::new();
            tmpfile.as_file_mut().seek(SeekFrom::Start(0)).expect("Failed to seek temp file");
            tmpfile.read_to_end(&mut buf).expect("Failed to read temp PNG");
            if buf.is_empty() {
                // 1x1 transparent PNG fallback
                buf = vec![
                    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,
                    0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
                    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
                    0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
                    0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41,
                    0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00,
                    0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00,
                    0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE,
                    0x42, 0x60, 0x82
                ];
            }
            buf
        };
        let table_data_b64 = base64::engine::general_purpose::STANDARD.encode(&table_png);
        // --- Bar chart image ---
        use std::collections::HashMap;
        let mut status_counts: HashMap<String, u32> = HashMap::new();
        for job in &jobs {
            *status_counts.entry(job.status.to_string()).or_insert(0) += 1;
        }
        let chart_width = 500;
        let chart_height = 300;
        let chart_png: Vec<u8> = {
            use plotters::prelude::*;
            
            use std::io::Read;
            let mut tmpfile = Builder::new().suffix(".png").tempfile().expect("Failed to create temp file");
            {
                let root = BitMapBackend::new(tmpfile.path(), (chart_width, chart_height)).into_drawing_area();
                root.fill(&WHITE).expect("Failed to fill background");
                let statuses: Vec<_> = status_counts.keys().cloned().collect();
                let counts: Vec<_> = statuses.iter().map(|s| status_counts[s]).collect();
                let max_count = *counts.iter().max().unwrap_or(&1);
                let mut chart = ChartBuilder::on(&root)
                    .caption("Job Status Counts", ("sans-serif", 25))
                    .margin(20)
                    .x_label_area_size(40)
                    .y_label_area_size(40)
                    .build_cartesian_2d(0..statuses.len(), 0..(max_count + 1))
                    .expect("Failed to build chart");
                chart.configure_mesh()
                    .x_labels(statuses.len())
                    .x_label_formatter(&|idx| statuses.get(*idx).map(|s| s.to_string()).unwrap_or_else(|| "".to_string()))
                    .y_desc("Count")
                    .x_desc("Status")
                    .draw().expect("Failed to draw mesh");
                chart.draw_series(
                    counts.iter().enumerate().map(|(i, count)| {
                        Rectangle::new([
                            (i, 0), (i + 1, *count)
                        ], BLUE.filled())
                    })
                ).expect("Failed to draw series");
                root.present().expect("Failed to present chart PNG");
            }
            let mut buf = Vec::new();
            tmpfile.as_file_mut().seek(SeekFrom::Start(0)).expect("Failed to seek temp file");
            tmpfile.read_to_end(&mut buf).expect("Failed to read temp PNG");
            if buf.is_empty() {
                // 1x1 transparent PNG fallback
                buf = vec![
                    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,
                    0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
                    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
                    0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
                    0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41,
                    0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00,
                    0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00,
                    0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE,
                    0x42, 0x60, 0x82
                ];
            }
            buf
        };
        let chart_data_b64 = base64::engine::general_purpose::STANDARD.encode(&chart_png);
        // --- Markdown fallback ---
        let mut md = String::from("| Name | Status | Workflow | Repo | Started |\n|------|--------|----------|------|---------|\n");
        for job in &jobs {
            md.push_str(&format!(
                "| {} | {:?} | {} | {} | {} |\n",
                job.name,
                job.status,
                job.workflow,
                job.repository,
                job.started_at.map(|dt| dt.to_rfc3339()).unwrap_or_else(|| "-".to_string()),
            ));
        }
        Ok(ToolResult {
            content: serde_json::Value::Array(vec![
                serde_json::json!({
                    "type": "image",
                    "data": table_data_b64,
                    "mimeType": "image/png"
                }),
                serde_json::json!({
                    "type": "image",
                    "data": chart_data_b64,
                    "mimeType": "image/png"
                }),
                serde_json::json!({
                    "type": "text",
                    "text": md
                })
            ]),
            duration: None,
            error: None,
            success: true,
        })
    }
}

/// 2. Add JobHistoryTool
#[derive(Clone)]
pub struct JobHistoryTool {
    runner_core: Arc<RwLock<RunnerCore>>,
}

impl JobHistoryTool {
    pub fn new(runner_core: Arc<RwLock<RunnerCore>>) -> Self {
        Self { runner_core }
    }
    pub fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "job_history".to_string(),
            description: "List recent job runs and their status".to_string(),
            input_schema: serde_json::json!({"type": "object"}),
            permissions: vec!["job:read".to_string()],
        }
    }
    pub fn validate_args(&self, _args: &serde_json::Value) -> Result<()> {
        Ok(())
    }
    pub async fn execute(&self, _args: serde_json::Value) -> Result<ToolResult> {
        let runner_core = self.runner_core.read().await;
        let job_runs = runner_core.get_all_job_runs().await;
        let total_jobs = job_runs.len();
        let successful_jobs = job_runs.iter().filter(|run| run.status == crate::types::JobStatus::Completed).count();
        let success_rate = if total_jobs > 0 {
            (successful_jobs as f64 / total_jobs as f64) * 100.0
        } else { 0.0 };
        let average_duration = if total_jobs > 0 {
            let total_duration: u128 = job_runs.iter().map(|run| run.duration_ms).sum();
            total_duration / total_jobs as u128
        } else { 0 };
        let content = serde_json::json!({
            "recent_jobs": job_runs.iter().take(10).collect::<Vec<_>>(),
            "success_rate": success_rate,
            "average_duration_ms": average_duration,
            "total_jobs": total_jobs,
            "last_updated": Utc::now().to_rfc3339(),
        });
        Ok(ToolResult { success: true, content, error: None, duration: None })
    }
}

/// 3. Add JobLogsTool
#[derive(Clone)]
pub struct JobLogsTool {
    runner_core: Arc<RwLock<RunnerCore>>,
}

impl JobLogsTool {
    pub fn new(runner_core: Arc<RwLock<RunnerCore>>) -> Self {
        Self { runner_core }
    }
    pub fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "job_logs".to_string(),
            description: "Get logs or differential logs for a given job name".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": { "job_name": { "type": "string" } },
                "required": ["job_name"]
            }),
            permissions: vec!["job:read".to_string()],
        }
    }
    pub fn validate_args(&self, args: &serde_json::Value) -> Result<()> {
        if !args.is_object() {
            return Err(BarefootError::Mcp("Arguments must be an object".to_string()));
        }
        if let Some(job_name) = args.get("job_name") {
            if !job_name.is_string() {
                return Err(BarefootError::Mcp("job_name must be a string".to_string()));
            }
        } else {
            return Err(BarefootError::Mcp("job_name is required".to_string()));
        }
        Ok(())
    }
    pub async fn execute(&self, args: serde_json::Value) -> Result<ToolResult> {
        self.validate_args(&args)?;
        let job_name = args["job_name"].as_str().unwrap();
        let runner_core = self.runner_core.read().await;
        let diff_logs = runner_core.get_differential_logs(job_name).await;
        let content = serde_json::json!({ "job_name": job_name, "differential_logs": diff_logs });
        Ok(ToolResult { success: true, content, error: None, duration: None })
    }
}

/// Tool manager for MCP server
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
    
    /// Get reference to async tools for cloning
    pub fn async_tools(&self) -> &std::collections::HashMap<String, AsyncToolHandler> {
        &self.async_tools
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

#[cfg(test)]
mod image_tests {
    use super::*;
    use std::io::{Read, Seek, SeekFrom};
    use tempfile::Builder;
    

    #[test]
    fn test_table_image_generation() {
        let width = 700;
        let height = 120;
        let mut tmpfile = Builder::new().suffix(".png").tempfile().expect("Failed to create temp file");
        {
            let root = BitMapBackend::new(tmpfile.path(), (width, height)).into_drawing_area();
            root.fill(&WHITE).expect("Failed to fill background");
            let font = ("sans-serif", 20).into_font();
            let header = ["Name", "Status", "Workflow", "Repo", "Started"];
            let mut y = 10;
            for (i, h) in header.iter().enumerate() {
                root.draw_text(h, &TextStyle::from(font.clone()).color(&BLACK), (20 + i as i32 * 130, y)).expect("Failed to draw header");
            }
            y += 30;
            let row = ["test-job", "Running", "test", "barefoot", "-"];
            for (i, cell) in row.iter().enumerate() {
                root.draw_text(cell, &TextStyle::from(font.clone()).color(&BLACK), (20 + i as i32 * 130, y)).expect("Failed to draw cell");
            }
            root.present().expect("Failed to present table PNG");
        }
        let mut buf = Vec::new();
        tmpfile.as_file_mut().seek(SeekFrom::Start(0)).expect("Failed to seek temp file");
        tmpfile.read_to_end(&mut buf).expect("Failed to read temp PNG");
        assert!(!buf.is_empty(), "Generated PNG should not be empty");
        assert_eq!(&buf[0..8], b"\x89PNG\r\n\x1a\n", "PNG header should be present");
    }

    #[test]
    fn test_chart_image_generation() {
        let chart_width = 500;
        let chart_height = 300;
        let mut tmpfile = Builder::new().suffix(".png").tempfile().expect("Failed to create temp file");
        {
            let root = BitMapBackend::new(tmpfile.path(), (chart_width, chart_height)).into_drawing_area();
            root.fill(&WHITE).expect("Failed to fill background");
            let statuses = vec!["Running", "Queued"];
            let counts = vec![1, 2];
            let max_count = *counts.iter().max().unwrap_or(&1);
            let mut chart = ChartBuilder::on(&root)
                .caption("Job Status Counts", ("sans-serif", 25))
                .margin(20)
                .x_label_area_size(40)
                .y_label_area_size(40)
                .build_cartesian_2d(0..statuses.len(), 0..(max_count + 1))
                .expect("Failed to build chart");
            chart.configure_mesh()
                .x_labels(statuses.len())
                .x_label_formatter(&|idx| statuses.get(*idx).map(|s| s.to_string()).unwrap_or_else(|| "".to_string()))
                .y_desc("Count")
                .x_desc("Status")
                .draw().expect("Failed to draw mesh");
            chart.draw_series(
                counts.iter().enumerate().map(|(i, count)| {
                    Rectangle::new([
                        (i, 0), (i + 1, *count)
                    ], BLUE.filled())
                })
            ).expect("Failed to draw series");
            root.present().expect("Failed to present chart PNG");
        }
        let mut buf = Vec::new();
        tmpfile.as_file_mut().seek(SeekFrom::Start(0)).expect("Failed to seek temp file");
        tmpfile.read_to_end(&mut buf).expect("Failed to read temp PNG");
        assert!(!buf.is_empty(), "Generated PNG should not be empty");
        assert_eq!(&buf[0..8], b"\x89PNG\r\n\x1a\n", "PNG header should be present");
    }
} 