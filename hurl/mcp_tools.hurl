# Hurl test suite for MCP tools

# 1. Start Job
POST http://127.0.0.1:{{PORT}}/mcp
Content-Type: application/json

{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "call_tool",
  "params": {
    "name": "start_job",
    "arguments": { "job_id": "550e8400-e29b-41d4-a716-446655440000", "priority": 5 }
  }
}

HTTP/1.1 200
jsonpath "$.result.content[0].type" == "text"

# 2. Stop Job
POST http://127.0.0.1:{{PORT}}/mcp
Content-Type: application/json

{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "call_tool",
  "params": {
    "name": "stop_job",
    "arguments": { "job_id": "550e8400-e29b-41d4-a716-446655440000", "force": true }
  }
}

HTTP/1.1 200
jsonpath "$.result.content[0].type" == "text" || "$.result.content.type" == "text"

# 3. Health Check
POST http://127.0.0.1:{{PORT}}/mcp
Content-Type: application/json

{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "call_tool",
  "params": {
    "name": "health_check",
    "arguments": { "detailed": true }
  }
}

HTTP/1.1 200
jsonpath "$.result.content[0].type" == "text"

# 4. Weather Dashboard
POST http://127.0.0.1:{{PORT}}/mcp
Content-Type: application/json

{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "call_tool",
  "params": {
    "name": "weather_dashboard",
    "arguments": { "timeframe": "1h", "include_system": true }
  }
}

HTTP/1.1 200
jsonpath "$.result.content[0].type" == "text"

# 5. Alerts
POST http://127.0.0.1:{{PORT}}/mcp
Content-Type: application/json

{
  "jsonrpc": "2.0",
  "id": 5,
  "method": "call_tool",
  "params": {
    "name": "alerts",
    "arguments": { "severity": "medium", "include_resolved": false }
  }
}

HTTP/1.1 200
jsonpath "$.result.content[0].type" == "text"

# 6. Dependency Graph (SVG)
POST http://127.0.0.1:{{PORT}}/mcp
Content-Type: application/json

{
  "jsonrpc": "2.0",
  "id": 6,
  "method": "call_tool",
  "params": {
    "name": "dependency_graph",
    "arguments": { "format": "svg", "include_workflows": true }
  }
}

HTTP/1.1 200
jsonpath "$.result.content[0].type" == "image"

# 7. Analytics
POST http://127.0.0.1:{{PORT}}/mcp
Content-Type: application/json

{
  "jsonrpc": "2.0",
  "id": 7,
  "method": "call_tool",
  "params": {
    "name": "analytics",
    "arguments": { "metric": "duration", "timeframe": "hours", "aggregation": "avg" }
  }
}

HTTP/1.1 200
jsonpath "$.result.content[0].type" == "text"
jsonpath "$.result.content[1].type" == "image"

# 8. List Jobs
POST http://127.0.0.1:{{PORT}}/mcp
Content-Type: application/json

{
  "jsonrpc": "2.0",
  "id": 8,
  "method": "call_tool",
  "params": {
    "name": "list_jobs",
    "arguments": { "which": "all" }
  }
}

HTTP/1.1 200
jsonpath "$.result.content[0].type" == "text"

# 9. Job History
POST http://127.0.0.1:{{PORT}}/mcp
Content-Type: application/json

{
  "jsonrpc": "2.0",
  "id": 9,
  "method": "call_tool",
  "params": {
    "name": "job_history",
    "arguments": {}
  }
}

HTTP/1.1 200
jsonpath "$.result.content[0].type" == "text"

# 10. Job Logs (dummy job)
POST http://127.0.0.1:{{PORT}}/mcp
Content-Type: application/json

{
  "jsonrpc": "2.0",
  "id": 10,
  "method": "call_tool",
  "params": {
    "name": "job_logs",
    "arguments": { "job_name": "dummy-job-1" }
  }
}

HTTP/1.1 200
jsonpath "$.result.content[0].type" == "text" 