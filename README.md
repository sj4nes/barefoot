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

## Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/barefoot/barefoot.git
cd barefoot

# Build the project
cargo build --release

# Install globally (optional)
cargo install --path .
```

### Configuration

Create a configuration file `barefoot.toml`:

```toml
[runner]
name = "my-runner"
url = "http://localhost:8080"
token = "your-runner-token"
labels = ["self-hosted", "linux"]
max_concurrent_jobs = 2
work_dir = "./work"

[runner.capabilities]
os = "linux"
architecture = "x86_64"
labels = ["self-hosted", "linux"]
features = { "docker" = "true", "gpu" = "false" }

[service]
service_type = "GitHub"
url = "https://api.github.com"
token = "your-service-token"
api_version = "2022-11-28"

[logging]
level = "info"
format = "json"
file = null

[security]
enable_ssl_verification = true
allowed_origins = ["*"]
max_upload_size = 104857600
```

### Usage

```bash
# Start the runner in foreground mode
barefoot start --foreground

# Configure the runner
barefoot config --service-type github --service-token YOUR_TOKEN

# Test configuration
barefoot test

# Show status
barefoot status

# Stop the runner
barefoot stop
```

## Architecture

### Core Components

- **Runner Core**: Manages runner state, job queue, and concurrent execution
- **Job Executor**: Handles workflow parsing and job execution
- **Service Client**: Abstracts communication with different Git hosting platforms
- **Configuration Manager**: Handles loading and validation of configuration
- **Utilities**: Common utilities for file operations, networking, and crypto

### Service Integration

The runner supports multiple service types through a trait-based architecture:

- **GitHub**: Full GitHub Actions compatibility
- **GitLab**: GitLab CI/CD integration (TODO: Implement)
- **Gitea**: Gitea Actions support (TODO: Implement)
- **Forgejo**: Forgejo Actions support (TODO: Implement - Gitea fork)
- **Codeberg**: Codeberg Actions support (TODO: Implement - Gitea-based)
- **Sourcehut**: Sourcehut builds support (TODO: Implement)
- **Jujutsu**: Native Jujutsu integration
- **Custom**: Extensible for other services

### Job Execution

1. **Job Discovery**: Runner polls service for available jobs
2. **Job Assignment**: Service assigns jobs based on runner capabilities
3. **Workflow Parsing**: YAML workflows are parsed and validated
4. **Step Execution**: Each step is executed in sequence
5. **Status Updates**: Job status is reported back to service
6. **Log Collection**: Step outputs and logs are captured

## Development

### Prerequisites

- Rust 1.70 or later
- Cargo (comes with Rust)

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Run with specific log level
RUST_LOG=debug cargo run -- start --foreground
```

### Project Structure

```
src/
├── main.rs          # CLI entry point
├── lib.rs           # Library entry point
├── config.rs        # Configuration management
├── core.rs          # Core runner logic
├── error.rs         # Error types and handling
├── runner.rs        # Job execution and workflow parsing
├── service.rs       # Service client implementations
├── types.rs         # Core data structures
└── utils.rs         # Utility functions
```

### Adding New Services

To add support for a new service:

1. Implement the `ServiceClient` trait
2. Add the service type to the `ServiceType` enum
3. Update the `ServiceFactory` to create the new client
4. Add tests for the new service

Example:

```rust
pub struct MyServiceClient {
    client: Client,
    config: BarefootConfig,
}

#[async_trait::async_trait]
impl ServiceClient for MyServiceClient {
    async fn get_jobs(&self) -> Result<Vec<Job>> {
        // Implementation
    }
    
    // ... other methods
}
```

## Configuration

### Environment Variables

All configuration can be overridden with environment variables:

```bash
export BAREFOOT_RUNNER_NAME="my-runner"
export BAREFOOT_SERVICE_TOKEN="your-token"
export BAREFOOT_LOGGING_LEVEL="debug"
```

### Configuration File

The configuration file supports TOML format with nested sections:

```toml
[runner]
name = "runner-name"
# ... runner configuration

[service]
service_type = "GitHub"
# ... service configuration

[logging]
level = "info"
# ... logging configuration

[security]
enable_ssl_verification = true
# ... security configuration
```

## Security

### Authentication

- **Runner Token**: Used for runner registration and authentication
- **Service Token**: Used for API communication with the service
- **HMAC Signatures**: Used for webhook verification

### Network Security

- **SSL Verification**: Enabled by default
- **Custom Headers**: Support for custom authentication headers
- **Origin Validation**: Configurable allowed origins

### File System Security

- **Work Directory Isolation**: Jobs run in isolated directories
- **File Permissions**: Proper file permissions for sensitive data
- **Cleanup**: Automatic cleanup of job artifacts

## Performance

### Concurrency

- **Async I/O**: Built on Tokio for efficient async operations
- **Concurrent Jobs**: Configurable maximum concurrent job execution
- **Connection Pooling**: HTTP client connection reuse

### Resource Management

- **Memory Efficient**: Minimal memory footprint
- **CPU Optimization**: Efficient job scheduling
- **Disk Usage**: Configurable work directory cleanup

## Troubleshooting

### Common Issues

1. **Configuration Errors**
   ```bash
   barefoot test
   ```

2. **Network Issues**
   ```bash
   # Check service connectivity
   curl -H "Authorization: token YOUR_TOKEN" https://api.github.com/user
   ```

3. **Permission Issues**
   ```bash
   # Ensure work directory is writable
   chmod 755 ./work
   ```

### Logging

Enable debug logging for troubleshooting:

```bash
RUST_LOG=debug barefoot start --foreground
```

### Health Checks

The runner provides health check endpoints:

```bash
# Check runner status
barefoot status

# Test configuration
barefoot test
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Run the test suite
6. Submit a pull request

### Development Guidelines

- Follow Rust coding conventions
- Add tests for new features
- Update documentation
- Use meaningful commit messages

## License

MIT License - see LICENSE file for details.

## Acknowledgments

- Inspired by GitHub Actions Runner
- Built with modern Rust ecosystem
- Community-driven development 