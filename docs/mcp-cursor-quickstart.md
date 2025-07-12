# Barefoot MCP + Cursor Quick Start

Get started with AI-powered CI/CD management in 5 minutes.

## Quick Setup

### 1. Start Barefoot MCP Server

```bash
# Install barefoot (if not already installed)
cargo install --path .

# Start MCP server with stdio transport
barefoot mcp --transport stdio
```

### 2. Configure Cursor

1. Open Cursor Settings (`Cmd/Ctrl + ,`)
2. Go to `Extensions > MCP`
3. Click "Add MCP Server"
4. Configure:

```json
{
  "name": "barefoot-runner",
  "command": "barefoot",
  "args": ["mcp", "--transport", "stdio"]
}
```

### 3. Test the Integration

In Cursor, try these commands:

```
"What's the health of my CI runner?"
"Show me the current jobs"
"Check if the runner can accept more jobs"
```

## Common Commands

### Monitor Your CI/CD

```
"Show me the runner status"
"What jobs are running?"
"What's the queue size?"
"Check the health score"
```

### Manage Jobs

```
"Start a new job with high priority"
"Stop the failing job"
"Cancel all queued jobs"
```

### Troubleshoot Issues

```
"What's the success rate of recent jobs?"
"Show me job performance trends"
"Check if there are any stuck jobs"
```

## Available Resources

- **Health**: `barefoot://runner/health` - System status and metrics
- **Active Jobs**: `barefoot://jobs/active` - Running and queued jobs
- **Job History**: `barefoot://jobs/history` - Historical data and trends
- **Configuration**: `barefoot://config/runner` - Current settings

## Available Tools

- **`start_job`** - Queue a new job
- **`stop_job`** - Cancel a running job
- **`health_check`** - Comprehensive system health check

## Transport Options

### stdio (Recommended for local development)
```bash
barefoot mcp --transport stdio
```

### TCP (For network access)
```bash
barefoot mcp --transport tcp --host 127.0.0.1 --port 8080
```

### WebSocket (For real-time updates)
```bash
barefoot mcp --transport websocket --host 127.0.0.1 --port 8081
```

## Troubleshooting

### Connection Issues
```bash
# Check if server is running
ps aux | grep barefoot

# Enable debug logging
RUST_LOG=debug barefoot mcp --transport stdio
```

### Test MCP Connection
```bash
# Test with curl (for TCP transport)
curl -X POST http://127.0.0.1:8080 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}'
```

## Next Steps

1. **Read the full guide**: `docs/mcp-cursor-integration.md`
2. **Explore resources**: Try accessing different MCP resources
3. **Customize**: Configure authentication and security
4. **Scale**: Set up multiple runners with load balancing

## Support

- **Documentation**: Check `docs/` directory
- **Issues**: GitHub repository
- **MCP Protocol**: [Model Context Protocol](https://modelcontextprotocol.io/)

---

**That's it!** You now have AI-powered CI/CD management in Cursor. 🚀 