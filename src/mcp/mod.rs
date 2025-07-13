//! MCP (Model Context Protocol) integration for barefoot runner
//!
//! This module provides integration with the Model Context Protocol,
//! allowing AI agents to interact with the barefoot runner system.

pub mod prompts;
pub mod resources;
pub mod server;
pub mod tools;
pub mod transport;

use crate::error::Result;
use crate::BarefootError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// MCP configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    /// Transport type (stdio, tcp, websocket)
    pub transport: TransportType,
    /// Authentication settings
    pub auth: AuthConfig,
    /// Resource limits
    pub limits: ResourceLimits,
    /// Enable features
    pub features: McpFeatures,
}

/// Transport type for MCP communication
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransportType {
    /// Standard input/output (default)
    Stdio,
    /// TCP socket
    Tcp { host: String, port: u16 },
    /// WebSocket
    WebSocket { host: String, port: u16 },
    /// HTTP
    Http { host: String, port: u16 },
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Enable authentication
    pub enabled: bool,
    /// API key for authentication
    pub api_key: Option<String>,
    /// Token-based authentication
    pub tokens: Vec<String>,
    /// Role-based access control
    pub roles: HashMap<String, Vec<String>>,
}

/// Resource limits configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum concurrent connections
    pub max_connections: usize,
    /// Maximum resource size in bytes
    pub max_resource_size: usize,
    /// Request timeout in seconds
    pub request_timeout: u64,
    /// Rate limiting (requests per minute)
    pub rate_limit: usize,
}

/// MCP features configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpFeatures {
    /// Enable resource streaming
    pub streaming: bool,
    /// Enable real-time updates
    pub realtime: bool,
    /// Enable sampling capabilities
    pub sampling: bool,
    /// Enable prompt templates
    pub prompts: bool,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            transport: TransportType::Stdio,
            auth: AuthConfig::default(),
            limits: ResourceLimits::default(),
            features: McpFeatures::default(),
        }
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            api_key: None,
            tokens: Vec::new(),
            roles: HashMap::new(),
        }
    }
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_connections: 10,
            max_resource_size: 1024 * 1024, // 1MB
            request_timeout: 30,
            rate_limit: 100,
        }
    }
}

impl Default for McpFeatures {
    fn default() -> Self {
        Self {
            streaming: true,
            realtime: true,
            sampling: false,
            prompts: true,
        }
    }
}

/// MCP resource identifier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceId {
    /// Resource URI
    pub uri: String,
    /// Resource MIME type
    pub mime_type: String,
    /// Resource title
    pub title: String,
    /// Resource description
    pub description: String,
}

/// MCP resource content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceContent {
    /// Resource ID
    pub id: ResourceId,
    /// Resource content (JSON)
    pub content: serde_json::Value,
    /// Content hash for caching
    pub hash: Option<String>,
    /// Expiration timestamp
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// MCP tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: String,
    /// Input schema (JSON Schema)
    pub input_schema: serde_json::Value,
    /// Required permissions
    pub permissions: Vec<String>,
}

/// MCP tool result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// Success status
    pub success: bool,
    /// Result content
    pub content: serde_json::Value,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Execution duration
    pub duration: Option<chrono::Duration>,
}

/// MCP prompt template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    /// Template name
    pub name: String,
    /// Template description
    pub description: String,
    /// Template content
    pub content: String,
    /// Template variables
    pub variables: Vec<String>,
    /// Template category
    pub category: String,
}

/// MCP server status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStatus {
    /// Server is running
    pub running: bool,
    /// Active connections
    pub connections: usize,
    /// Server uptime
    pub uptime: chrono::Duration,
    /// Last error
    pub last_error: Option<String>,
    /// Resource count
    pub resource_count: usize,
    /// Tool count
    pub tool_count: usize,
}

/// MCP error types
#[derive(Debug, thiserror::Error)]
pub enum McpError {
    #[error("MCP server error: {0}")]
    Server(String),

    #[error("Resource not found: {0}")]
    ResourceNotFound(String),

    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("Invalid tool arguments: {0}")]
    InvalidArguments(String),

    #[error("Authentication failed: {0}")]
    Authentication(String),

    #[error("Authorization failed: {0}")]
    Authorization(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Request timeout")]
    RequestTimeout,

    #[error("Transport error: {0}")]
    Transport(String),
}

impl From<McpError> for BarefootError {
    fn from(err: McpError) -> Self {
        BarefootError::Mcp(err.to_string())
    }
}

/// MCP server builder
pub struct McpServerBuilder {
    config: McpConfig,
}

impl McpServerBuilder {
    /// Create a new MCP server builder
    pub fn new() -> Self {
        Self {
            config: McpConfig::default(),
        }
    }

    /// Set transport type
    pub fn transport(mut self, transport: TransportType) -> Self {
        self.config.transport = transport;
        self
    }

    /// Enable authentication
    pub fn auth(mut self, auth: AuthConfig) -> Self {
        self.config.auth = auth;
        self
    }

    /// Set resource limits
    pub fn limits(mut self, limits: ResourceLimits) -> Self {
        self.config.limits = limits;
        self
    }

    /// Enable features
    pub fn features(mut self, features: McpFeatures) -> Self {
        self.config.features = features;
        self
    }

    /// Build the MCP server
    pub fn build(self) -> Result<server::BarefootMcpServer> {
        Ok(server::BarefootMcpServer::new(self.config))
    }
}

impl Default for McpServerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_config_default() {
        let config = McpConfig::default();
        assert!(!config.auth.enabled);
        assert_eq!(config.limits.max_connections, 10);
        assert!(config.features.streaming);
    }

    #[test]
    fn test_mcp_server_builder() {
        let builder = McpServerBuilder::new()
            .transport(TransportType::Stdio)
            .features(McpFeatures {
                streaming: true,
                realtime: true,
                sampling: false,
                prompts: true,
            });

        assert_eq!(builder.config.transport, TransportType::Stdio);
        assert!(builder.config.features.streaming);
    }
}
