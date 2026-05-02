//! hestia-debug-cli -- Debug conductor CLI client

use anyhow::Result;
use clap::{Parser, Subcommand};
use conductor_sdk::agent::ConductorId;
use conductor_sdk::config::{CommonOpts, HestiaClientConfig};
use conductor_sdk::message::{MessageId, Payload, Request};
use conductor_sdk::transport::AgentCliClient;

#[derive(Parser)]
#[command(name = "hestia-debug-cli", version, about = "Debug domain CLI")]
struct Cli {
    #[command(flatten)]
    common: CommonOpts,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a debug session
    Create,
    /// Connect to a debug target
    Connect,
    /// Disconnect from a debug target
    Disconnect,
    /// Program the target device
    Program,
    /// Capture signal waveforms
    Capture {
        #[command(subcommand)]
        capture_command: CaptureCommands,
    },
    /// List available signals
    Signals,
    /// Set a trigger condition
    Trigger,
    /// Reset the target
    Reset,
    /// Show Debug conductor status
    Status,
}

#[derive(Subcommand)]
enum CaptureCommands {
    /// Start signal capture
    Start,
    /// Stop signal capture
    Stop,
}

fn build_request(method: &str, params: serde_json::Value) -> Request {
    Request {
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
        Commands::Create => ("debug.create", serde_json::json!({})),
        Commands::Connect => ("debug.connect", serde_json::json!({})),
        Commands::Disconnect => ("debug.disconnect", serde_json::json!({})),
        Commands::Program => ("debug.program", serde_json::json!({})),
        Commands::Capture { capture_command } => match capture_command {
            CaptureCommands::Start => ("debug.startCapture", serde_json::json!({})),
            CaptureCommands::Stop => ("debug.stopCapture", serde_json::json!({})),
        },
        Commands::Signals => ("debug.read_signals", serde_json::json!({})),
        Commands::Trigger => ("debug.set_trigger", serde_json::json!({})),
        Commands::Reset => ("debug.reset", serde_json::json!({})),
        Commands::Status => ("debug.status", serde_json::json!({})),
    };

    let request = build_request(method, params);
    let payload = Payload::Structured(serde_json::to_value(&request)?);
    let response = client.send_to_conductor(ConductorId::Debug, &payload).await?;

    output_result(&cli.common, method, &response);
    Ok(())
}

fn common_timeout_seconds(opts: &CommonOpts) -> Option<u64> {
    opts.timeout
}