//! hestia-ai-cli -- AI conductor CLI client

use anyhow::Result;
use clap::{Parser, Subcommand};
use conductor_sdk::agent::ConductorId;
use conductor_sdk::config::{CommonOpts, HestiaClientConfig};
use conductor_sdk::message::{MessageId, Payload, Request};
use conductor_sdk::transport::AgentCliClient;

#[derive(Parser)]
#[command(name = "hestia-ai-cli", version, about = "AI conductor CLI")]
struct Cli {
    #[command(flatten)]
    common: CommonOpts,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Execute a natural language or structured instruction
    Exec {
        /// Instruction text to execute
        instruction: String,
    },
    /// Run an instruction from a file
    Run {
        /// Path to instruction file
        #[arg(long, short)]
        file: String,
    },
    /// Initialize a specification session
    SpecInit {
        /// Specification text (natural language or structured)
        spec_text: Option<String>,
        /// Format of the specification (default: natural)
        #[arg(long, default_value = "natural")]
        format: String,
    },
    /// Update an existing specification
    SpecUpdate,
    /// Start a specification review
    SpecReview,
    /// List registered sub-agents
    AgentLs,
    /// List containers
    ContainerLs,
    /// Start a container
    ContainerStart {
        /// Container name
        name: String,
    },
    /// Stop a container
    ContainerStop {
        /// Container name
        name: String,
    },
    /// Create a container from container.toml
    ContainerCreate {
        /// Container name
        name: String,
    },
    /// Run a workflow
    WorkflowRun {
        /// Workflow name
        name: String,
    },
    /// Start a review
    ReviewStart,
    /// Show AI conductor status
    Status,
}

fn build_request(method: &str, params: serde_json::Value) -> Request {
    Request {
        kind: "prompt".to_string(),
        method: method.to_string(),
        params,
        id: MessageId::new(),
        trace_id: None,
    }
}

fn output_result(common: &CommonOpts, label: &str, response: &str) {
    if common.output == "json" {
        println!("{}", response);
    } else {
        println!("[{label}] {response}");
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut config = HestiaClientConfig::default();
    if let Some(timeout) = common_timeout_seconds(&cli.common) {
        config.request_timeout = timeout * 1000;
    }
    if let Some(ref registry) = cli.common.registry {
        config.agent_cli_registry_dir = registry.clone();
    }
    if let Some(ref config_path) = cli.common.config {
        let text = std::fs::read_to_string(config_path)?;
        config = toml::from_str(&text).unwrap_or(config);
    }
    if cli.common.verbose {
        tracing_subscriber::fmt::init();
    }

    let client = AgentCliClient::new(config)?;

    let (method, params) = match &cli.command {
        Commands::Exec { instruction } => (
            "ai.exec",
            serde_json::json!({ "instruction": instruction }),
        ),
        Commands::Run { file } => {
            let content = std::fs::read_to_string(file)
                .map_err(|e| anyhow::anyhow!("failed to read file '{}': {e}", file))?;
            (
                "ai.exec",
                serde_json::json!({ "instruction": content, "source_file": file }),
            )
        }
        Commands::SpecInit { spec_text, format } => (
            "ai.spec.init",
            serde_json::json!({
                "spec_text": spec_text.as_deref().unwrap_or(""),
                "format": format,
            }),
        ),
        Commands::SpecUpdate => ("ai.spec.update", serde_json::json!({})),
        Commands::SpecReview => ("ai.spec.review", serde_json::json!({})),
        Commands::AgentLs => ("agent_list", serde_json::json!({})),
        Commands::ContainerLs => ("container.list", serde_json::json!({})),
        Commands::ContainerStart { name } => (
            "container.start",
            serde_json::json!({ "name": name }),
        ),
        Commands::ContainerStop { name } => (
            "container.stop",
            serde_json::json!({ "name": name }),
        ),
        Commands::ContainerCreate { name } => (
            "container.create",
            serde_json::json!({ "name": name }),
        ),
        Commands::WorkflowRun { name } => (
            "meta.dualBuild",
            serde_json::json!({ "workflow": name }),
        ),
        Commands::ReviewStart => ("ai.spec.review", serde_json::json!({})),
        Commands::Status => ("system.health.v1", serde_json::json!({})),
    };

    let request = build_request(method, params);
    let payload = Payload::Structured(serde_json::to_value(&request)?);
    let response = client.send_to_conductor(ConductorId::Ai, &payload).await?;

    output_result(&cli.common, method, &response);
    Ok(())
}

fn common_timeout_seconds(opts: &CommonOpts) -> Option<u64> {
    opts.timeout
}