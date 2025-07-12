//! barefoot: A modern, flexible runner system for GitHub-like services and Jujutsu
//!
//! # TODOs
//!
//! ## MCP (Model Context Protocol) Integration
//! - Implement MCP server that exposes barefoot runner state and capabilities
//! - Add MCP Resources for job status, runner health, and historical data
//! - Create MCP Tools for job management (start, stop, pause, resume jobs)
//! - Implement MCP Prompts for common runner operations and troubleshooting
//! - Add MCP Sampling for LLM-assisted job analysis and optimization
//! - Support MCP Transports (stdio, TCP, WebSocket) for different deployment scenarios
//! - Implement secure authentication and authorization for MCP connections
//! - Add MCP server discovery and registration mechanisms
//! - Create MCP client library for other applications to connect to barefoot
//! - Implement MCP resource streaming for real-time job status updates
//!
//! ## Weather Dashboard & Monitoring
//! - Add a "weather conditions" dashboard that provides a high-level summary of all running tasks, showing which jobs are healthy (succeeding), which are having issues (failing), and overall system health.
//! - Add weather alerts and notifications for when jobs start failing consistently or when system health degrades.
//! - Add weather forecasting to predict potential issues based on historical job performance and current trends.
//!
//! ## Visualization & Analytics
//! - Add support for displaying sparklines of successful job statuses, so users can visualize trends of job success over time.
//! - Add support for declaratively showing the dependency graph of tasks/jobs, so users can visualize and understand job/task dependencies.
//! - Sparklines should show minutes (green/red), hours (green/red), or days (green/red) depending on the period
//! - Sparklines should display cycle time for each task: average, p99, and last
//!
//! This crate provides a runner system that can be used with any GitHub-like service
//! or Jujutsu. It's designed to be more flexible and efficient than traditional runners.

pub mod config;
pub mod core;
pub mod error;
pub mod runner;
pub mod service;
pub mod types;
pub mod utils;
pub mod mcp;

pub use error::{BarefootError, Result};

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize the barefoot runner system
pub async fn init() -> Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("Barefoot runner initialized, version: {}", VERSION);
    Ok(())
} 