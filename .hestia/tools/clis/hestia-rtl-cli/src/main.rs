//! hestia-rtl-cli -- RTL conductor CLI client

use anyhow::Result;
use clap::{Parser, Subcommand};
use conductor_sdk::agent::ConductorId;
use conductor_sdk::config::{CommonOpts, HestiaClientConfig};
use conductor_sdk::message::{MessageId, Payload, Request};
use conductor_sdk::transport::AgentCliClient;

#[derive(Parser)]
#[command(name = "hestia-rtl-cli", version, about = "RTL domain CLI")]
struct Cli {
    #[command(flatten)]
    common: CommonOpts,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize RTL project workspace
    Init,
    /// Lint RTL source files
    Lint,
    /// Run RTL simulation
    Simulate,
    /// Run formal verification
    Formal,
    /// Transpile RTL between formats
    Transpile,
    /// Hand off RTL artifacts to downstream domain
    Handoff,
    /// Show RTL conductor status
    Status,
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

    let method = match &cli.command {
        Commands::Init => "rtl.init",
        Commands::Lint => "rtl.lint",
        Commands::Simulate => "rtl.simulate",
        Commands::Formal => "rtl.formal",
        Commands::Transpile => "rtl.transpile",
        Commands::Handoff => "rtl.handoff",
        Commands::Status => "rtl.status",
    };

    let params = match &cli.command {
        Commands::Init => serde_json::json!({}),
        Commands::Lint => serde_json::json!({}),
        Commands::Simulate => serde_json::json!({}),
        Commands::Formal => serde_json::json!({}),
        Commands::Transpile => serde_json::json!({}),
        Commands::Handoff => serde_json::json!({}),
        Commands::Status => serde_json::json!({}),
    };

    let label = method;
    let request = build_request(method, params);
    let payload = Payload::Structured(serde_json::to_value(&request)?);
    let response = client.send_to_conductor(ConductorId::Rtl, &payload).await?;

    output_result(&cli.common, label, &response);
    Ok(())
}

fn common_timeout_seconds(opts: &CommonOpts) -> Option<u64> {
    opts.timeout
}