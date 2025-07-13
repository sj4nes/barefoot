# Using Barefoot MCP with Cursor

This guide explains how to use the Model Context Protocol (MCP) integration in barefoot with Cursor IDE for enhanced AI-assisted development and CI/CD management.

## Overview

Barefoot's MCP integration allows Cursor to interact with your CI/CD runner system programmatically, enabling:

- **Real-time job monitoring**: View running jobs, queue status, and system health
- **Job management**: Start, stop, and control jobs through AI commands
- **Troubleshooting**: AI-assisted debugging of failed jobs and system issues
- **Resource optimization**: Get insights into runner performance and capacity
- **Historical analysis**: Access job history and performance metrics

## Prerequisites

- Barefoot runner installed and configured
- Cursor IDE with MCP support
- Basic understanding of CI/CD concepts

## Setup

### 1. Install Barefoot MCP Server

The MCP server is built into barefoot. To start it in MCP mode:

```bash
# Start barefoot in MCP mode with stdio transport
barefoot mcp --transport stdio

# Or with TCP transport for network access
barefoot mcp --transport tcp --host 127.0.0.1 --port 8080

# Or with WebSocket for real-time communication
barefoot mcp --transport websocket --host 127.0.0.1 --port 8081
```

### 2. Configure Cursor for MCP

In Cursor, you can configure MCP servers through:

1. **Settings**: Go to `Cursor > Settings > Extensions > MCP`
2. **Add Server**: Click "Add MCP Server"
3. **Configure**: Enter the server details

#### Configuration Options

**For stdio transport:**
```json
{
  "name": "barefoot-runner",
  "command": "barefoot",
  "args": ["mcp", "--transport", "stdio"],
  "env": {}
}
```

**For TCP transport:**
```json
{
  "name": "barefoot-runner",
  "url": "tcp://127.0.0.1:8080"
}
```

**For WebSocket transport:**
```json
{
  "name": "barefoot-runner",
  "url": "ws://127.0.0.1:8081"
}
```

## Available MCP Resources

### 1. Runner Health (`barefoot://runner/health`)

Get overall system health and status:

```json
{
  "status": "idle|busy|error",
  "active_jobs": 2,
  "queue_size": 5,
  "capabilities": {...},
  "uptime": 3600,
  "last_error": null,
  "can_accept_jobs": true,
  "health_score": 95.0
}
```

**Usage in Cursor:**
- Ask: "What's the current health of my CI runner?"
- Command: "Show me the runner status and active jobs"

### 2. Active Jobs (`barefoot://jobs/active`)

View currently running and queued jobs:

```json
{
  "active_jobs": [...],
  "queued_jobs": [...],
  "total_count": 7,
  "summary": {
    "running": 2,
    "queued": 5,
    "total": 7
  },
  "last_updated": "2024-01-15T10:30:00Z"
}
```

**Usage in Cursor:**
- Ask: "What jobs are currently running?"
- Command: "Show me the job queue status"

### 3. Job History (`barefoot://jobs/history`)

Access historical job data and performance metrics:

```json
{
  "recent_jobs": [...],
  "success_rate": 87.5,
  "average_duration_ms": 45000,
  "total_jobs": 24,
  "last_updated": "2024-01-15T10:30:00Z"
}
```

**Usage in Cursor:**
- Ask: "What's the success rate of my recent jobs?"
- Command: "Show me job performance trends"

### 4. Configuration (`barefoot://config/runner`)

View current runner configuration:

```json
{
  "service_type": "GitHub",
  "service_url": "https://api.github.com",
  "runner_name": "my-runner",
  "max_concurrent_jobs": 4,
  "log_level": "info",
  "differential_logging": {...}
}
```

**Usage in Cursor:**
- Ask: "What's my runner configuration?"
- Command: "Show me the current runner settings"

## Available MCP Tools

### 1. Job Management Tools

#### Start Job (`start_job`)
Start execution of a specific job:

```json
{
  "job_id": "550e8400-e29b-41d4-a716-446655440000",
  "priority": 5
}
```

**Usage in Cursor:**
- Ask: "Start job with ID 550e8400-e29b-41d4-a716-446655440000"
- Command: "Queue a new job with high priority"

#### Stop Job (`stop_job`)
Stop/cancel a running job:

```json
{
  "job_id": "550e8400-e29b-41d4-a716-446655440000",
  "force": true
}
```

**Usage in Cursor:**
- Ask: "Stop the failing job immediately"
- Command: "Cancel job 550e8400-e29b-41d4-a716-446655440000"

### 2. Health Check Tool (`health_check`)

Perform comprehensive health checks:

```json
{
  "detailed": true
}
```

**Usage in Cursor:**
- Ask: "Check the health of my CI system"
- Command: "Run a detailed health check"

## Common Use Cases

### 1. Monitoring CI/CD Pipeline

**Scenario**: You want to monitor your CI/CD pipeline in real-time.

**Cursor Commands:**
```
"Show me the current status of all my CI jobs"
"What's the health score of my runner?"
"Are there any stuck jobs in the queue?"
```

### 2. Troubleshooting Failed Jobs

**Scenario**: A job failed and you need to investigate.

**Cursor Commands:**
```
"Show me the logs for the failed job"
"What's the success rate of similar jobs?"
"Check if the runner is healthy"
```

### 3. Resource Management

**Scenario**: You want to optimize resource usage.

**Cursor Commands:**
```
"How many jobs can the runner accept?"
"What's the average job duration?"
"Show me the job queue length"
```

### 4. Job Control

**Scenario**: You need to manage running jobs.

**Cursor Commands:**
```
"Start a new job with high priority"
"Stop the job that's consuming too many resources"
"Cancel all queued jobs"
```

## Advanced Features

### 1. Real-time Updates

With WebSocket transport, Cursor can receive real-time updates:

- Job status changes
- Queue updates
- System health alerts
- Performance metrics

### 2. AI-Assisted Debugging

Cursor can use MCP data to:

- Analyze job failure patterns
- Suggest optimizations
- Identify resource bottlenecks
- Provide troubleshooting recommendations

### 3. Historical Analysis

Access historical data for:

- Performance trending
- Success rate analysis
- Duration optimization
- Capacity planning

## Configuration Examples

### Development Environment

```json
{
  "name": "barefoot-dev",
  "command": "barefoot",
  "args": ["mcp", "--transport", "stdio"],
  "env": {
    "RUST_LOG": "debug",
    "BAREFOOT_CONFIG": "/path/to/dev/config.toml"
  }
}
```

### Production Environment

```json
{
  "name": "barefoot-prod",
  "url": "tcp://runner.company.com:8080",
  "auth": {
    "type": "token",
    "token": "your-auth-token"
  }
}
```

### Multi-Runner Setup

```json
{
  "name": "barefoot-cluster",
  "url": "ws://runner-cluster.company.com:8081",
  "auth": {
    "type": "api_key",
    "key": "your-api-key"
  }
}
```

## Troubleshooting

### Common Issues

1. **Connection Failed**
   - Check if barefoot MCP server is running
   - Verify transport configuration
   - Check firewall settings for TCP/WebSocket

2. **Authentication Errors**
   - Verify API keys or tokens
   - Check authentication configuration
   - Ensure proper permissions

3. **Resource Not Found**
   - Verify resource URIs are correct
   - Check if resources are available
   - Ensure proper MCP protocol version

### Debug Mode

Enable debug logging for troubleshooting:

```bash
RUST_LOG=debug barefoot mcp --transport stdio
```

### Health Checks

Use the health check tool to diagnose issues:

```json
{
  "detailed": true
}
```

## Best Practices

### 1. Security

- Use authentication for production deployments
- Restrict network access to MCP servers
- Rotate API keys regularly
- Monitor access logs

### 2. Performance

- Use appropriate transport for your use case
- Monitor resource usage
- Implement rate limiting
- Cache frequently accessed data

### 3. Reliability

- Implement retry logic
- Use health checks
- Monitor connection status
- Have fallback configurations

### 4. Monitoring

- Track MCP usage metrics
- Monitor response times
- Alert on failures
- Log important events

## Integration Examples

### GitHub Actions Integration

```bash
# Start barefoot with GitHub integration
barefoot mcp --transport tcp --host 0.0.0.0 --port 8080 \
  --config /path/to/github-config.toml
```

### GitLab CI Integration

```bash
# Start barefoot with GitLab integration
barefoot mcp --transport websocket --host 0.0.0.0 --port 8081 \
  --config /path/to/gitlab-config.toml
```

### Jujutsu Integration

```bash
# Start barefoot with Jujutsu integration
barefoot mcp --transport stdio \
  --config /path/to/jujutsu-config.toml
```

## Future Enhancements

### Planned Features

1. **Advanced Analytics**
   - Machine learning-based failure prediction
   - Performance optimization recommendations
   - Resource usage forecasting

2. **Enhanced Security**
   - Role-based access control
   - Audit logging
   - Encryption at rest

3. **Integration Ecosystem**
   - Slack/Discord notifications
   - Grafana dashboards
   - Prometheus metrics

4. **AI-Powered Features**
   - Automatic job optimization
   - Intelligent resource allocation
   - Predictive scaling

## Support

For issues and questions:

1. **Documentation**: Check the main barefoot documentation
2. **Issues**: Report bugs on the GitHub repository
3. **Discussions**: Join community discussions
4. **MCP Protocol**: Refer to the MCP specification

## Conclusion

The MCP integration in barefoot provides powerful capabilities for AI-assisted CI/CD management through Cursor. By following this guide, you can effectively monitor, control, and optimize your CI/CD pipeline using natural language commands and real-time data access.

The combination of barefoot's robust runner capabilities with Cursor's AI assistance creates a powerful development environment that can significantly improve your CI/CD workflow efficiency and reliability. 