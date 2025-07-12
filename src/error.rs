use thiserror::Error;

/// Custom error type for the barefoot runner
#[derive(Error, Debug)]
pub enum BarefootError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("YAML parsing error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Process execution error: {0}")]
    Process(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Invalid workflow: {0}")]
    Workflow(String),

    #[error("Job execution failed: {0}")]
    JobExecution(String),

    #[error("Service not found: {0}")]
    ServiceNotFound(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Validation error: {0}")]
    Validation(String),
}

/// Result type for the barefoot runner
pub type Result<T> = std::result::Result<T, BarefootError>; 

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_conversions() {
        let config_error = BarefootError::Config("test".to_string());
        assert_eq!(config_error.to_string(), "Configuration error: test");

        let network_error = BarefootError::Network(reqwest::Error::new(reqwest::StatusCode::INTERNAL_SERVER_ERROR));
        assert_eq!(network_error.to_string(), "Network error: reqwest::Error { status: 500, url: None, method: None, source: None }");

        let serialization_error = BarefootError::Serialization(serde_json::Error::new("test"));
        assert_eq!(serialization_error.to_string(), "Serialization error: test");

        let yaml_error = BarefootError::Yaml(serde_yaml::Error::new("test"));
        assert_eq!(yaml_error.to_string(), "YAML parsing error: test");

        let io_error = BarefootError::Io(std::io::Error::new(std::io::ErrorKind::NotFound));
        assert_eq!(io_error.to_string(), "IO error: NotFound");

        let process_error = BarefootError::Process("test".to_string());
        assert_eq!(process_error.to_string(), "Process execution error: test");

        let auth_error = BarefootError::Auth("test".to_string());
        assert_eq!(auth_error.to_string(), "Authentication error: test");

        let workflow_error = BarefootError::Workflow("test".to_string());
        assert_eq!(workflow_error.to_string(), "Invalid workflow: test");

        let job_execution_error = BarefootError::JobExecution("test".to_string());
        assert_eq!(job_execution_error.to_string(), "Job execution failed: test");

        let service_not_found_error = BarefootError::ServiceNotFound("test".to_string());
        assert_eq!(service_not_found_error.to_string(), "Service not found: test");

        let invalid_state_error = BarefootError::InvalidState("test".to_string());
        assert_eq!(invalid_state_error.to_string(), "Invalid state: test");

        let timeout_error = BarefootError::Timeout("test".to_string());
        assert_eq!(timeout_error.to_string(), "Timeout: test");

        let not_found_error = BarefootError::NotFound("test".to_string());
        assert_eq!(not_found_error.to_string(), "Resource not found: test");

        let permission_denied_error = BarefootError::PermissionDenied("test".to_string());
        assert_eq!(permission_denied_error.to_string(), "Permission denied: test");

        let validation_error = BarefootError::Validation("test".to_string());
        assert_eq!(validation_error.to_string(), "Validation error: test");
    }
} 