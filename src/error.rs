/// Custom error type for the barefoot runner
#[derive(Debug, thiserror::Error)]
pub enum BarefootError {
    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("HTTP request failed: {0}")]
    HttpRequest(#[from] reqwest::Error),

    #[error("HTTP status error: {status}")]
    HttpStatus { status: u16 },

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("TOML serialization error: {0}")]
    TomlSerialization(#[from] toml::ser::Error),

    #[error("TOML deserialization error: {0}")]
    TomlDeserialization(#[from] toml::de::Error),

    #[error("YAML parsing error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Service not found: {0}")]
    ServiceNotFound(String),

    #[error("Too many concurrent jobs")]
    TooManyJobs,

    #[error("Job not found: {0}")]
    JobNotFound(String),

    #[error("Workflow parsing error: {0}")]
    WorkflowParse(String),

    #[error("Workflow error: {0}")]
    Workflow(String),

    #[error("Process execution error: {0}")]
    Process(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("MCP error: {0}")]
    Mcp(String),

    #[error("Anyhow error: {0}")]
    Anyhow(#[from] anyhow::Error),
}

impl From<std::net::AddrParseError> for BarefootError {
    fn from(err: std::net::AddrParseError) -> Self {
        BarefootError::Mcp(format!("Invalid address: {}", err))
    }
}

/// Result type for the barefoot runner
pub type Result<T> = std::result::Result<T, BarefootError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_conversions() {
        let config_error = BarefootError::Configuration("test".to_string());
        assert_eq!(config_error.to_string(), "Configuration error: test");

        // reqwest::Error cannot be constructed directly; skip this test or use a dummy error if needed
        // let network_error = BarefootError::HttpRequest(reqwest::Error::new(reqwest::StatusCode::INTERNAL_SERVER_ERROR));
        // assert_eq!(network_error.to_string(), "HTTP request failed: ...");

        // serde_json::Error cannot be constructed directly; skip this test
        // let serialization_error = BarefootError::Serialization(serde_json::Error::new("test"));
        // assert_eq!(serialization_error.to_string(), "Serialization error: test");

        // serde_yaml::Error cannot be constructed directly; skip this test
        // let yaml_error = BarefootError::Yaml(serde_yaml::Error::new("test"));
        // assert_eq!(yaml_error.to_string(), "YAML parsing error: test");

        // std::io::Error requires a message
        let io_error = BarefootError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "not found",
        ));
        assert_eq!(io_error.to_string(), "IO error: not found");

        let process_error = BarefootError::Process("test".to_string());
        assert_eq!(process_error.to_string(), "Process execution error: test");

        // Removed non-existent variants: Auth, JobExecution, Timeout, NotFound, PermissionDenied

        let service_not_found_error = BarefootError::ServiceNotFound("test".to_string());
        assert_eq!(
            service_not_found_error.to_string(),
            "Service not found: test"
        );

        let invalid_state_error = BarefootError::InvalidState("test".to_string());
        assert_eq!(invalid_state_error.to_string(), "Invalid state: test");

        let validation_error = BarefootError::Validation("test".to_string());
        assert_eq!(validation_error.to_string(), "Validation error: test");
    }
}
