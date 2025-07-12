//! Barefoot - A modern, flexible runner system
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