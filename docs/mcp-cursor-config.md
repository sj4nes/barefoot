# Configuring Barefoot MCP in .cursor/mcp.json

This guide shows you how to add barefoot's MCP server to your Cursor configuration.

## Quick Setup

### 1. Create or Edit `.cursor/mcp.json`

Create the file in your project root:

```bash
mkdir -p .cursor
touch .cursor/mcp.json
```

### 2. Add Barefoot MCP Server

Add this configuration to your `.cursor/mcp.json`:

```json
{
  "mcpServers": {
    "barefoot-runner": {
      "command": "barefoot",
      "args": ["mcp", "--transport", "stdio"],
      "env": {}
    }
  }
}
```

## Configuration Options

### Basic Configuration (Recommended)

```json
{
  "mcpServers": {
    "barefoot-runner": {
      "command": "barefoot",
      "args": ["mcp", "--transport", "stdio"],
      "env": {}
    }
  }
}
```

### With Custom Configuration File

```json
{
  "mcpServers": {
    "barefoot-runner": {
      "command": "barefoot",
      "args": ["mcp", "--transport", "stdio", "--config", "barefoot-mcp.toml"],
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
```

### With TCP Transport

```json
{
  "mcpServers": {
    "barefoot-runner": {
      "command": "barefoot",
      "args": ["mcp", "--transport", "tcp", "--host", "127.0.0.1", "--port", "8080"],
      "env": {}
    }
  }
}
```

### With WebSocket Transport

```json
{
  "mcpServers": {
    "barefoot-runner": {
      "command": "barefoot",
      "args": ["mcp", "--transport", "websocket", "--host", "127.0.0.1", "--port", "8081"],
      "env": {}
    }
  }
}
```

## Multiple Runners

If you have multiple barefoot runners, you can configure them separately:

```json
{
  "mcpServers": {
    "barefoot-dev": {
      "command": "barefoot",
      "args": ["mcp", "--transport", "stdio", "--config", "barefoot-dev.toml"],
      "env": {
        "RUST_LOG": "debug"
      }
    },
    "barefoot-prod": {
      "command": "barefoot",
      "args": ["mcp", "--transport", "tcp", "--host", "127.0.0.1", "--port", "8080", "--config", "barefoot-prod.toml"],
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
```

## Environment-Specific Configurations

### Development Environment

```json
{
  "mcpServers": {
    "barefoot-dev": {
      "command": "barefoot",
      "args": ["mcp", "--transport", "stdio"],
      "env": {
        "RUST_LOG": "debug",
        "BAREFOOT_CONFIG": "/path/to/dev/config.toml"
      }
    }
  }
}
```

### Production Environment

```json
{
  "mcpServers": {
    "barefoot-prod": {
      "command": "barefoot",
      "args": ["mcp", "--transport", "tcp", "--host", "0.0.0.0", "--port", "8080"],
      "env": {
        "RUST_LOG": "warn",
        "BAREFOOT_CONFIG": "/path/to/prod/config.toml"
      }
    }
  }
}
```

## Testing Your Configuration

### 1. Restart Cursor

After editing `.cursor/mcp.json`, restart Cursor to load the new configuration.

### 2. Test the Connection

In Cursor, try these commands:

```
"What's the health of my CI runner?"
"Show me the current jobs"
"Check if the runner can accept more jobs"
```

### 3. Check Logs

If you encounter issues, check the logs:

```bash
# Enable debug logging
RUST_LOG=debug barefoot mcp --transport stdio
```

## Troubleshooting

### Common Issues

1. **"Command not found"**
   - Ensure barefoot is installed: `cargo install --path .`
   - Check PATH: `which barefoot`

2. **"Connection failed"**
   - Verify barefoot is running: `ps aux | grep barefoot`
   - Check transport configuration
   - Test manually: `barefoot mcp --transport stdio`

3. **"Permission denied"**
   - Make sure the script is executable: `chmod +x /path/to/barefoot`
   - Check file permissions

### Debug Configuration

Add debug logging to your configuration:

```json
{
  "mcpServers": {
    "barefoot-runner": {
      "command": "barefoot",
      "args": ["mcp", "--transport", "stdio"],
      "env": {
        "RUST_LOG": "debug"
      }
    }
  }
}
```

## Complete Example

Here's a complete `.cursor/mcp.json` example:

```json
{
  "mcpServers": {
    "barefoot-runner": {
      "command": "barefoot",
      "args": ["mcp", "--transport", "stdio", "--config", "barefoot-mcp.toml"],
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
```

## Next Steps

1. **Create configuration file**: Copy `examples/mcp-config.toml` to your project
2. **Edit configuration**: Update with your actual settings
3. **Test connection**: Try the AI commands in Cursor
4. **Explore features**: Check out the full documentation

## Documentation

- **[Quick Start](docs/mcp-cursor-quickstart.md)** - Get started in 5 minutes
- **[Full Integration Guide](docs/mcp-cursor-integration.md)** - Complete documentation
- **[Setup Script](scripts/setup-mcp.sh)** - Automated setup process

---

**That's it!** Your barefoot MCP server is now configured in Cursor. 🚀 