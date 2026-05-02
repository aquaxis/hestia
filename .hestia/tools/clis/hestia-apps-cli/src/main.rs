//! hestia-apps-cli -- Apps conductor CLI client

use anyhow::Result;
use clap::{Parser, Subcommand};
use conductor_sdk::agent::ConductorId;
use conductor_sdk::config::{CommonOpts, HestiaClientConfig};
use conductor_sdk::message::{MessageId, Payload, Request};
use conductor_sdk::transport::AgentCliClient;

#[derive(Parser)]
#[command(name = "hestia-apps-cli", version, about = "Apps domain CLI")]
struct Cli {
    #[command(flatten)]
    common: CommonOpts,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize application project workspace
    Init,
    /// Build application firmware
    Build,
    /// Flash firmware to target
    Flash,
    /// Run tests
    Test {
        /// Test level: sil | hil | qemu
        level: String,
    },
    /// Show binary size analysis
    Size,
    /// Start debug session
    Debug,
    /// Show Apps conductor status
    Status,
}

fn build_request(method: &str, params: serde_json::Value) -> Request {
    Request {
        kind: "request".to_string(),
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
        Commands::Init => ("apps.init", serde_json::json!({})),
        Commands::Build => ("apps.build.v1", serde_json::json!({})),
        Commands::Flash => ("apps.flash.v1", serde_json::json!({})),
        Commands::Test { level } => ("apps.test.v1", serde_json::json!({ "level": level })),
        Commands::Size => ("apps.size.v1", serde_json::json!({})),
        Commands::Debug => ("apps.debug.v1", serde_json::json!({})),
        Commands::Status => ("apps.status", serde_json::json!({})),
    };

    let request = build_request(method, params);
    let payload = Payload::Structured(serde_json::to_value(&request)?);
    let response = client.send_to_conductor(ConductorId::Apps, &payload).await?;

    output_result(&cli.common, method, &response);
    Ok(())
}

fn common_timeout_seconds(opts: &CommonOpts) -> Option<u64> {
    opts.timeout
}