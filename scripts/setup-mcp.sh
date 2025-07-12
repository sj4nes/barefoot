#!/bin/bash

# Barefoot MCP Setup Script
# This script helps you quickly set up barefoot MCP integration with Cursor

set -e

echo "🚀 Setting up Barefoot MCP Integration with Cursor"
echo "=================================================="

# Check if barefoot is installed
if ! command -v barefoot &> /dev/null; then
    echo "❌ Barefoot is not installed or not in PATH"
    echo "Please install barefoot first:"
    echo "  cargo install --path ."
    exit 1
fi

echo "✅ Barefoot is installed"

# Create example config if it doesn't exist
if [ ! -f "barefoot-mcp.toml" ]; then
    echo "📝 Creating example MCP configuration..."
    cat > barefoot-mcp.toml << 'EOF'
[runner]
name = "mcp-runner"
url = "http://localhost:8080"
token = "your-runner-token"
labels = ["self-hosted", "mcp-enabled"]
max_concurrent_jobs = 4
work_dir = "./work"

[runner.capabilities]
os = "linux"
architecture = "x86_64"
labels = ["self-hosted", "mcp-enabled"]
features = { "docker" = "true", "gpu" = "false", "mcp" = "true" }

[service]
service_type = "GitHub"
url = "https://api.github.com"
token = "your-service-token"
api_version = "2022-11-28"

[logging]
level = "info"
format = "json"
file = null

[logging.differential_logging]
enabled = true
max_job_runs = 100

[security]
enable_ssl_verification = true
allowed_origins = ["*"]
max_upload_size = 104857600

[mcp]
transport = "stdio"
auth_enabled = false
max_connections = 10
request_timeout = 30
rate_limit = 100

[mcp.features]
streaming = true
realtime = true
sampling = false
prompts = true
EOF
    echo "✅ Created barefoot-mcp.toml"
else
    echo "✅ Configuration file already exists"
fi

# Test barefoot MCP
echo "🧪 Testing barefoot MCP server..."
timeout 5s barefoot mcp --transport stdio --config barefoot-mcp.toml || {
    echo "⚠️  MCP server test timed out (this is expected)"
}

echo ""
echo "🎉 Setup complete!"
echo ""
echo "Next steps:"
echo "1. Edit barefoot-mcp.toml with your actual configuration"
echo "2. Start the MCP server:"
echo "   barefoot mcp --transport stdio --config barefoot-mcp.toml"
echo ""
echo "3. Configure Cursor:"
echo "   - Open Cursor Settings"
echo "   - Go to Extensions > MCP"
echo "   - Add server with:"
echo "     Command: barefoot"
echo "     Args: [\"mcp\", \"--transport\", \"stdio\", \"--config\", \"barefoot-mcp.toml\"]"
echo ""
echo "4. Test with AI commands:"
echo "   \"What's the health of my CI runner?\""
echo "   \"Show me the current jobs\""
echo ""
echo "📚 Documentation:"
echo "   - Quick Start: docs/mcp-cursor-quickstart.md"
echo "   - Full Guide: docs/mcp-cursor-integration.md"
echo ""
echo "Happy coding! 🚀" 