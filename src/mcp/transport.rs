//! MCP transport implementations for barefoot runner

use super::*;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};
use futures::StreamExt;
use serde_json::Value;
use crate::error::Result;

/// Transport trait for MCP communication
pub trait Transport: Send + Sync {
    /// Start the transport
    async fn start(&mut self) -> Result<()>;
    
    /// Stop the transport
    async fn stop(&mut self) -> Result<()>;
    
    /// Check if transport is running
    fn is_running(&self) -> bool;
    
    /// Get transport type
    fn transport_type(&self) -> TransportType;
}

/// stdio transport implementation
pub struct StdioTransport {
    running: bool,
    server: Option<server::BarefootMcpServer>,
}

impl StdioTransport {
    pub fn new() -> Self {
        Self {
            running: false,
            server: None,
        }
    }
    
    pub fn with_server(mut self, server: server::BarefootMcpServer) -> Self {
        self.server = Some(server);
        self
    }
}

impl Transport for StdioTransport {
    async fn start(&mut self) -> Result<()> {
        self.running = true;
        tracing::info!("MCP stdio transport started");
        
        // TODO: Implement stdio-based MCP protocol
        // This would involve reading from stdin and writing to stdout
        // following the MCP protocol specification
        
        Ok(())
    }
    
    async fn stop(&mut self) -> Result<()> {
        self.running = false;
        tracing::info!("MCP stdio transport stopped");
        Ok(())
    }
    
    fn is_running(&self) -> bool {
        self.running
    }
    
    fn transport_type(&self) -> TransportType {
        TransportType::Stdio
    }
}

/// TCP transport implementation
pub struct TcpTransport {
    host: String,
    port: u16,
    running: bool,
    listener: Option<TcpListener>,
    server: Option<server::BarefootMcpServer>,
}

impl TcpTransport {
    pub fn new(host: String, port: u16) -> Self {
        Self {
            host,
            port,
            running: false,
            listener: None,
            server: None,
        }
    }
    
    pub fn with_server(mut self, server: server::BarefootMcpServer) -> Self {
        self.server = Some(server);
        self
    }
    
    async fn handle_connection(&self, stream: TcpStream) -> Result<()> {
        let (mut read, mut write) = stream.into_split();
        
        // TODO: Implement MCP protocol over TCP
        // This would involve parsing JSON-RPC messages and handling MCP requests
        
        tracing::info!("TCP connection established");
        
        // For now, just log the connection
        let _ = write.write_all(b"{\"jsonrpc\": \"2.0\", \"method\": \"initialize\", \"params\": {}}\n").await;
        
        Ok(())
    }
}

impl Transport for TcpTransport {
    async fn start(&mut self) -> Result<()> {
        if self.running {
            return Ok(());
        }
        
        let listener = TcpListener::bind(format!("{}:{}", self.host, self.port)).await
            .map_err(|e| BarefootError::Mcp(format!("Failed to bind to {}:{}: {}", self.host, self.port, e)))?;
        
        self.running = true;
        self.listener = Some(listener);
        
        tracing::info!("MCP TCP transport started on {}:{}", self.host, self.port);
        
        Ok(())
    }
    
    async fn stop(&mut self) -> Result<()> {
        self.running = false;
        self.listener = None;
        tracing::info!("MCP TCP transport stopped");
        Ok(())
    }
    
    fn is_running(&self) -> bool {
        self.running
    }
    
    fn transport_type(&self) -> TransportType {
        TransportType::Tcp { host: self.host.clone(), port: self.port }
    }
}

/// WebSocket transport implementation
pub struct WebSocketTransport {
    host: String,
    port: u16,
    running: bool,
    server: Option<server::BarefootMcpServer>,
}

impl WebSocketTransport {
    pub fn new(host: String, port: u16) -> Self {
        Self {
            host,
            port,
            running: false,
            server: None,
        }
    }
    
    pub fn with_server(mut self, server: server::BarefootMcpServer) -> Self {
        self.server = Some(server);
        self
    }
}

impl Transport for WebSocketTransport {
    async fn start(&mut self) -> Result<()> {
        self.running = true;
        tracing::info!("MCP WebSocket transport started on {}:{}", self.host, self.port);
        
        // TODO: Implement WebSocket-based MCP protocol
        // This would involve using a WebSocket library like tokio-tungstenite
        
        Ok(())
    }
    
    async fn stop(&mut self) -> Result<()> {
        self.running = false;
        tracing::info!("MCP WebSocket transport stopped");
        Ok(())
    }
    
    fn is_running(&self) -> bool {
        self.running
    }
    
    fn transport_type(&self) -> TransportType {
        TransportType::WebSocket { host: self.host.clone(), port: self.port }
    }
}

/// Transport factory for creating different transport types
pub struct TransportFactory;

impl TransportFactory {
    /// Create a transport based on type and server
    pub fn create(transport_type: TransportType, server: server::BarefootMcpServer) -> Result<TransportEnum> {
        match transport_type {
            TransportType::Stdio => {
                let transport = StdioTransport::new().with_server(server);
                Ok(TransportEnum::Stdio(transport))
            }
            TransportType::Tcp { host, port } => {
                let transport = TcpTransport::new(host, port).with_server(server);
                Ok(TransportEnum::Tcp(transport))
            }
            TransportType::WebSocket { host, port } => {
                let transport = WebSocketTransport::new(host, port).with_server(server);
                Ok(TransportEnum::WebSocket(transport))
            }
        }
    }
}

/// Enum to hold different transport types
pub enum TransportEnum {
    Stdio(StdioTransport),
    Tcp(TcpTransport),
    WebSocket(WebSocketTransport),
}

impl Transport for TransportEnum {
    async fn start(&mut self) -> Result<()> {
        match self {
            TransportEnum::Stdio(transport) => transport.start().await,
            TransportEnum::Tcp(transport) => transport.start().await,
            TransportEnum::WebSocket(transport) => transport.start().await,
        }
    }
    
    async fn stop(&mut self) -> Result<()> {
        match self {
            TransportEnum::Stdio(transport) => transport.stop().await,
            TransportEnum::Tcp(transport) => transport.stop().await,
            TransportEnum::WebSocket(transport) => transport.stop().await,
        }
    }
    
    fn is_running(&self) -> bool {
        match self {
            TransportEnum::Stdio(transport) => transport.is_running(),
            TransportEnum::Tcp(transport) => transport.is_running(),
            TransportEnum::WebSocket(transport) => transport.is_running(),
        }
    }
    
    fn transport_type(&self) -> TransportType {
        match self {
            TransportEnum::Stdio(transport) => transport.transport_type(),
            TransportEnum::Tcp(transport) => transport.transport_type(),
            TransportEnum::WebSocket(transport) => transport.transport_type(),
        }
    }
}

/// MCP message for JSON-RPC communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpMessage {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: Option<String>,
    pub params: Option<Value>,
    pub result: Option<Value>,
    pub error: Option<McpError>,
}

/// MCP error for JSON-RPC responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
    pub data: Option<Value>,
}

/// MCP protocol handler for processing messages
pub struct McpProtocolHandler {
    server: server::BarefootMcpServer,
}

impl McpProtocolHandler {
    pub fn new(server: server::BarefootMcpServer) -> Self {
        Self { server }
    }
    
    pub async fn handle_message(&self, message: McpMessage) -> Result<McpMessage> {
        match message.method.as_deref() {
            Some("initialize") => self.handle_initialize(message).await,
            Some("list_resources") => self.handle_list_resources(message).await,
            Some("read_resource") => self.handle_read_resource(message).await,
            Some("list_tools") => self.handle_list_tools(message).await,
            Some("call_tool") => self.handle_call_tool(message).await,
            Some("list_prompts") => self.handle_list_prompts(message).await,
            Some("get_prompt") => self.handle_get_prompt(message).await,
            _ => Err(BarefootError::Mcp("Unknown method".to_string())),
        }
    }
    
    async fn handle_initialize(&self, message: McpMessage) -> Result<McpMessage> {
        // TODO: Implement proper initialization
        Ok(McpMessage {
            jsonrpc: "2.0".to_string(),
            id: message.id,
            method: None,
            params: None,
            result: Some(serde_json::json!({
                "protocolVersion": "2025-06-18",
                "capabilities": {
                    "resources": {},
                    "tools": {},
                    "prompts": {}
                }
            })),
            error: None,
        })
    }
    
    async fn handle_list_resources(&self, message: McpMessage) -> Result<McpMessage> {
        let resources = self.server.list_resources().await?;
        Ok(McpMessage {
            jsonrpc: "2.0".to_string(),
            id: message.id,
            method: None,
            params: None,
            result: Some(serde_json::to_value(resources)?),
            error: None,
        })
    }
    
    async fn handle_read_resource(&self, message: McpMessage) -> Result<McpMessage> {
        let uri = message.params
            .and_then(|p| p.get("uri").cloned())
            .and_then(|u| u.as_str().map(|s| s.to_string()))
            .ok_or_else(|| BarefootError::Mcp("Missing uri parameter".to_string()))?;
        
        let resource = self.server.get_resource(&uri).await?;
        Ok(McpMessage {
            jsonrpc: "2.0".to_string(),
            id: message.id,
            method: None,
            params: None,
            result: Some(serde_json::to_value(resource)?),
            error: None,
        })
    }
    
    async fn handle_list_tools(&self, message: McpMessage) -> Result<McpMessage> {
        let tools = self.server.list_tools().await?;
        Ok(McpMessage {
            jsonrpc: "2.0".to_string(),
            id: message.id,
            method: None,
            params: None,
            result: Some(serde_json::to_value(tools)?),
            error: None,
        })
    }
    
    async fn handle_call_tool(&self, message: McpMessage) -> Result<McpMessage> {
        let params = message.params
            .ok_or_else(|| BarefootError::Mcp("Missing params".to_string()))?;
        
        let name = params.get("name")
            .and_then(|n| n.as_str())
            .ok_or_else(|| BarefootError::Mcp("Missing tool name".to_string()))?;
        
        let args = params.get("arguments")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({}));
        
        let result = self.server.execute_tool(name, args).await?;
        Ok(McpMessage {
            jsonrpc: "2.0".to_string(),
            id: message.id,
            method: None,
            params: None,
            result: Some(serde_json::to_value(result)?),
            error: None,
        })
    }
    
    async fn handle_list_prompts(&self, message: McpMessage) -> Result<McpMessage> {
        let prompts = self.server.list_prompts().await?;
        Ok(McpMessage {
            jsonrpc: "2.0".to_string(),
            id: message.id,
            method: None,
            params: None,
            result: Some(serde_json::to_value(prompts)?),
            error: None,
        })
    }
    
    async fn handle_get_prompt(&self, message: McpMessage) -> Result<McpMessage> {
        let name = message.params
            .and_then(|p| p.get("name").cloned())
            .and_then(|n| n.as_str().map(|s| s.to_string()))
            .ok_or_else(|| BarefootError::Mcp("Missing prompt name".to_string()))?;
        
        let prompt = self.server.get_prompt(&name).await?;
        Ok(McpMessage {
            jsonrpc: "2.0".to_string(),
            id: message.id,
            method: None,
            params: None,
            result: Some(serde_json::to_value(prompt)?),
            error: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_stdio_transport() {
        let mut transport = StdioTransport::new();
        assert!(!transport.is_running());
        
        let result = transport.start().await;
        assert!(result.is_ok());
        assert!(transport.is_running());
        
        let result = transport.stop().await;
        assert!(result.is_ok());
        assert!(!transport.is_running());
    }
    
    #[tokio::test]
    async fn test_tcp_transport() {
        let transport = TcpTransport::new("127.0.0.1".to_string(), 8080);
        assert_eq!(transport.transport_type(), TransportType::Tcp { host: "127.0.0.1".to_string(), port: 8080 });
    }
    
    #[tokio::test]
    async fn test_websocket_transport() {
        let transport = WebSocketTransport::new("127.0.0.1".to_string(), 8081);
        assert_eq!(transport.transport_type(), TransportType::WebSocket { host: "127.0.0.1".to_string(), port: 8081 });
    }
    
    #[tokio::test]
    async fn test_transport_factory() {
        let config = McpConfig::default();
        let server = server::BarefootMcpServer::new(config);
        
        let transport = TransportFactory::create(TransportType::Stdio, server);
        assert!(transport.is_ok());
    }
    
    #[tokio::test]
    async fn test_mcp_protocol_handler() {
        let config = McpConfig::default();
        let server = server::BarefootMcpServer::new(config);
        let handler = McpProtocolHandler::new(server);
        
        let message = McpMessage {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::json!(1)),
            method: Some("initialize".to_string()),
            params: Some(serde_json::json!({})),
            result: None,
            error: None,
        };
        
        let response = handler.handle_message(message).await;
        assert!(response.is_ok());
    }
} 