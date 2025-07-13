# Barefoot Runner

A modern, flexible runner system for GitHub-like services and Jujutsu. Built in Rust for performance, reliability, and cross-platform compatibility.

## Features

- **Multi-service support**: Works with GitHub, GitLab, Gitea, and Jujutsu
- **Cross-platform**: Runs on Linux, macOS, and Windows
- **Async/await**: Built with Tokio for high-performance concurrent job execution
- **Flexible configuration**: TOML-based configuration with environment variable support
- **Comprehensive logging**: Structured logging with configurable levels
- **Security**: SSL verification, token-based authentication, and secure defaults
- **Extensible**: Plugin architecture for custom service integrations
- **MCP Integration**: Model Context Protocol support with two operation modes

## MCP (Model Context Protocol) Integration

Barefoot Runner supports the Model Context Protocol with **two distinct operation modes**:

### 1. Runner Mode with MCP Enabled
When you start the runner with MCP enabled, the application processes jobs automatically while providing MCP tools for monitoring and control.

```bash
# Start runner with MCP server enabled
barefoot start --enable-mcp

# With specific transport
barefoot start --enable-mcp --mcp-transport http --mcp-host localhost --mcp-port 3000
```

**Characteristics:**
- ✅ **Automatic job processing** - polls for jobs from services (GitHub, GitLab, etc.)
- ✅ **MCP tools available** - monitor and control jobs via MCP
- ✅ **Real-time status** - tools like `list_jobs` show current state
- ✅ **Background MCP server** - runs alongside job processing
- ✅ **Full automation** - traditional CI/CD runner behavior

### 2. MCP-Only Mode
When you start just the MCP server, the application provides tools for manual job management without automatic processing.

```bash
# Start MCP server only (no automatic job processing)
barefoot mcp start --transport http --host localhost --port 3000

# With stdio transport (for Cursor integration)
barefoot mcp start --transport stdio
```

**Characteristics:**
- ✅ **MCP tools available** - manual job management via tools
- ❌ **No automatic job processing** - jobs must be triggered manually
- ✅ **Lightweight** - minimal resource usage
- ✅ **Manual control** - full control over job execution
- ✅ **Integration focused** - designed for AI agent integration

### Mode Comparison

| Feature | Runner + MCP | MCP-Only |
|---------|-------------|----------|
| Automatic job polling | ✅ | ❌ |
| Manual job control | ✅ | ✅ |
| Service integration | ✅ | ✅ |
| Resource usage | Higher | Lower |
| Use case | Production CI/CD | Development/Control |

## Available MCP Tools

### Job Management
- **`list_jobs`** - View running and queued jobs with visual charts
- **`start_job`** - Manually trigger job execution
- **`stop_job`** - Cancel running jobs

### System Monitoring
- **`health_check`** - Check runner and service health
- **`weather_dashboard`** - System status overview with metrics

### Analytics and Visualization
- **`analytics`** - Job performance analytics and trends
- **`dependency_graph`** - Visualize job dependencies and workflows

### Alerts and Notifications
- **`alerts`** - View and manage system alerts

### Historical Data
- **`job_history`** - Access historical job data
- **`job_logs`** - Retrieve detailed job logs

## Transport Options

Barefoot Runner supports multiple MCP transport protocols:

- **stdio** - Standard input/output (default for Cursor integration)
- **HTTP** - REST API endpoint
- **TCP** - Raw TCP socket
- **WebSocket** - Real-time bidirectional communication

> **⚠️ Caveat**: The `stdio` transport mode does not appear to work with Cursor at this time. For Cursor integration, please use the HTTP transport with a web endpoint instead.

## Example Usage

### Runner Mode with MCP
```bash
# Start runner with automatic job processing and MCP tools
barefoot start --enable-mcp --mcp-transport http --mcp-host 0.0.0.0 --mcp-port 3000

# Use MCP tools to monitor jobs
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "call_tool",
    "params": {
      "name": "list_jobs",
      "arguments": {"which": "all"}
    }
  }'
```

### MCP-Only Mode
```bash
# Start MCP server for manual job control
barefoot mcp start --transport http --host localhost --port 3000

# Manually start a job
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "call_tool",
    "params": {
      "name": "start_job",
      "arguments": {
        "job_id": "550e8400-e29b-41d4-a716-446655440000",
        "priority": 5
      }
    }
  }'
```

## MCP Resources

Barefoot Runner provides real-time data resources:

- **Health Status** - Current system health and status
- **Active Jobs** - Currently running jobs
- **Job History** - Historical job data and metrics

## Quick Start

### 1. Install Barefoot Runner
```bash
cargo install barefoot-runner
```

### 2. Configure the Runner
```bash
barefoot config \
  --service-type github \
  --service-url https://api.github.com \
  --service-token your-token \
  --runner-name my-runner
```

### 3. Start in Runner Mode with MCP
```bash
# Start with automatic job processing and MCP tools
barefoot start --enable-mcp --mcp-transport http --mcp-host 0.0.0.0 --mcp-port 3000
```

### 4. Start in MCP-Only Mode
```bash
# Start MCP server for manual control
barefoot mcp start --transport http --host localhost --port 3000
```

### 5. Use MCP Tools
```bash
# List current jobs
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "call_tool",
    "params": {
      "name": "list_jobs",
      "arguments": {"which": "active"}
    }
  }'

# Check system health
curl -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 2,
    "method": "call_tool",
    "params": {
      "name": "health_check",
      "arguments": {}
    }
  }'
```

## Configuration

Barefoot Runner uses TOML configuration files. Create a `barefoot.toml` file:

```toml
[runner]
name = "my-runner"
max_concurrent_jobs = 4
work_dir = "/tmp/barefoot"

[service]
type = "github"
url = "https://api.github.com"
token = "your-github-token"

[mcp]
transport = "http"
host = "localhost"
port = 3000
```

## Development

### Building from Source
```bash
git clone https://github.com/your-org/barefoot-runner.git
cd barefoot-runner
cargo build --release
```

### Running Tests
```