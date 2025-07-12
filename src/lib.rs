//! barefoot: A modern, flexible runner system for GitHub-like services and Jujutsu
//!
//! # TODOs
//!
//! - Add a Machine Control Protocol (MCP) interface to the runner so agents can quickly query and see the state of all actions and jobs.
//! - Add support for displaying sparklines of successful job statuses, so users can visualize trends of job success over time.
//! - Add support for declaratively showing the dependency graph of tasks/jobs, so users can visualize and understand job/task dependencies.
//! - Add a "weather conditions" dashboard that provides a high-level summary of all running tasks, showing which jobs are healthy (succeeding), which are having issues (failing), and overall system health.
//! - Add weather alerts and notifications for when jobs start failing consistently or when system health degrades.
//! - Add weather forecasting to predict potential issues based on historical job performance and current trends.
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

pub use error::{BarefootError, Result};

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize the barefoot runner system
pub async fn init() -> Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("Barefoot runner initialized, version: {}", VERSION);
    Ok(())
} 