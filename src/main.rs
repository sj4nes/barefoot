use barefoot::{config::BarefootConfig, core::RunnerCore, runner::JobExecutor, service::ServiceFactory, Result, VERSION};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::{error, info, warn};

#[derive(Parser)]
#[command(name = "barefoot")]
#[command(about = "A modern, flexible runner system for GitHub-like services and Jujutsu")]
#[command(version = VERSION)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Configuration file path
    #[arg(short, long, default_value = "barefoot.toml")]
    config: PathBuf,

    /// Log level
    #[arg(short, long, default_value = "info")]
    log_level: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the runner
    Start {
        /// Run in foreground (don't daemonize)
        #[arg(short, long)]
        foreground: bool,
    },
    
    /// Stop the runner
    Stop,
    
    /// Show runner status
    Status,
    
    /// Configure the runner
    Config {
        /// Service type (github, gitlab, gitea, jujutsu)
        #[arg(long)]
        service_type: Option<String>,
        
        /// Service URL
        #[arg(long)]
        service_url: Option<String>,
        
        /// Service token
        #[arg(long)]
        service_token: Option<String>,
        
        /// Runner name
        #[arg(long)]
        runner_name: Option<String>,
        
        /// Runner token
        #[arg(long)]
        runner_token: Option<String>,
        
        /// Work directory
        #[arg(long)]
        work_dir: Option<String>,
    },
    
    /// Test configuration
    Test,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(format!("barefoot={}", cli.log_level))
        .with_target(false)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true)
        .with_ansi(true)
        .pretty()
        .init();

    info!("Barefoot runner starting, version: {}", VERSION);

    match cli.command {
        Commands::Start { foreground } => {
            start_runner(&cli.config, foreground).await?;
        }
        Commands::Stop => {
            stop_runner().await?;
        }
        Commands::Status => {
            show_status().await?;
        }
        Commands::Config { service_type, service_url, service_token, runner_name, runner_token, work_dir } => {
            configure_runner(&cli.config, service_type, service_url, service_token, runner_name, runner_token, work_dir).await?;
        }
        Commands::Test => {
            test_configuration(&cli.config).await?;
        }
    }

    Ok(())
}

async fn start_runner(config_path: &PathBuf, foreground: bool) -> Result<()> {
    info!("Loading configuration from: {:?}", config_path);
    
    // Load configuration
    let config = if config_path.exists() {
        BarefootConfig::from_file(config_path)?
    } else {
        warn!("Configuration file not found, using defaults");
        BarefootConfig::default()
    };

    // Validate configuration
    config.validate()?;
    info!("Configuration validated successfully");

    // Create runner core
    let core = RunnerCore::new(config.clone());
    info!("Runner core initialized");

    // Create service client
    let service_client = ServiceFactory::create_client(config.clone())?;
    info!("Service client created");

    // Register runner with service
    service_client.register_runner(&config.runner.capabilities).await?;
    info!("Runner registered with service");

    // Create job executor
    let executor = JobExecutor::new(core);

    if foreground {
        info!("Running in foreground mode");
        run_foreground(executor, service_client).await?;
    } else {
        info!("Running in daemon mode");
        run_daemon(executor, service_client).await?;
    }

    Ok(())
}

async fn run_foreground(
    executor: JobExecutor,
    service_client: Box<dyn barefoot::service::ServiceClient + Send + Sync>,
) -> Result<()> {
    info!("Starting foreground runner loop");
    
    loop {
        // Check for new jobs
        match service_client.get_jobs().await {
            Ok(jobs) => {
                for job in jobs {
                    info!("Processing job: {}", job.id);
                    
                    // Execute the job
                    match executor.execute_job(job.clone()).await {
                        Ok(status) => {
                            info!("Job {} completed with status: {:?}", job.id, status);
                            
                            // Update job status
                            let status_str = match status {
                                barefoot::types::JobStatus::Completed => "completed",
                                barefoot::types::JobStatus::Failed => "failed",
                                barefoot::types::JobStatus::Cancelled => "cancelled",
                                _ => "unknown",
                            };
                            
                            if let Err(e) = service_client.update_job_status(&job.id.to_string(), status_str).await {
                                error!("Failed to update job status: {}", e);
                            }
                        }
                        Err(e) => {
                            error!("Job {} failed: {}", job.id, e);
                            
                            if let Err(e) = service_client.update_job_status(&job.id.to_string(), "failed").await {
                                error!("Failed to update job status: {}", e);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                error!("Failed to get jobs: {}", e);
            }
        }

        // Sleep before next check
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    }
}

async fn run_daemon(
    _executor: JobExecutor,
    _service_client: Box<dyn barefoot::service::ServiceClient + Send + Sync>,
) -> Result<()> {
    // TODO: Implement daemon mode
    warn!("Daemon mode not yet implemented");
    Ok(())
}

async fn stop_runner() -> Result<()> {
    // TODO: Implement stop functionality
    info!("Stopping runner...");
    warn!("Stop functionality not yet implemented");
    Ok(())
}

async fn show_status() -> Result<()> {
    // TODO: Implement status display
    info!("Runner status:");
    warn!("Status functionality not yet implemented");
    Ok(())
}

async fn configure_runner(
    config_path: &PathBuf,
    service_type: Option<String>,
    service_url: Option<String>,
    service_token: Option<String>,
    runner_name: Option<String>,
    runner_token: Option<String>,
    work_dir: Option<String>,
) -> Result<()> {
    info!("Configuring runner...");
    
    let mut config = if config_path.exists() {
        BarefootConfig::from_file(config_path)?
    } else {
        BarefootConfig::default()
    };

    // Update configuration based on provided arguments
    if let Some(service_type) = service_type {
        config.service.service_type = match service_type.as_str() {
            "github" => barefoot::types::ServiceType::GitHub,
            "gitlab" => barefoot::types::ServiceType::GitLab,
            "gitea" => barefoot::types::ServiceType::Gitea,
            "jujutsu" => barefoot::types::ServiceType::Jujutsu,
            _ => barefoot::types::ServiceType::Custom,
        };
    }

    if let Some(service_url) = service_url {
        config.service.url = service_url;
    }

    if let Some(service_token) = service_token {
        config.service.token = service_token;
    }

    if let Some(runner_name) = runner_name {
        config.runner.name = runner_name;
    }

    if let Some(runner_token) = runner_token {
        config.runner.token = runner_token;
    }

    if let Some(work_dir) = work_dir {
        config.runner.work_dir = work_dir;
    }

    // Validate configuration
    config.validate()?;

    // Save configuration
    let config_content = toml::to_string_pretty(&config)
        .map_err(|e| barefoot::error::BarefootError::Serialization(e))?;
    
    std::fs::write(config_path, config_content)
        .map_err(|e| barefoot::error::BarefootError::Io(e))?;

    info!("Configuration saved to: {:?}", config_path);
    Ok(())
}

async fn test_configuration(config_path: &PathBuf) -> Result<()> {
    info!("Testing configuration...");
    
    let config = if config_path.exists() {
        BarefootConfig::from_file(config_path)?
    } else {
        return Err(barefoot::error::BarefootError::Config(
            "Configuration file not found".to_string(),
        ));
    };

    // Validate configuration
    config.validate()?;
    info!("Configuration is valid");

    // Test service connection
    let service_client = ServiceFactory::create_client(config.clone())?;
    
    match service_client.get_jobs().await {
        Ok(_) => {
            info!("Service connection successful");
        }
        Err(e) => {
            warn!("Service connection failed: {}", e);
        }
    }

    info!("Configuration test completed");
    Ok(())
}
