//! hestia-pcb-cli -- PCB conductor CLI client

use anyhow::Result;
use clap::{Parser, Subcommand};
use conductor_sdk::agent::ConductorId;
use conductor_sdk::config::{CommonOpts, HestiaClientConfig};
use conductor_sdk::message::{MessageId, Payload, Request};
use conductor_sdk::transport::AgentCliClient;

#[derive(Parser)]
#[command(name = "hestia-pcb-cli", version, about = "PCB domain CLI")]
struct Cli {
    #[command(flatten)]
    common: CommonOpts,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize PCB project workspace
    Init,
    /// Run PCB build flow
    Build,
    /// AI-driven PCB synthesis
    AiSynthesize,
    /// Export PCB output artifacts
    Output {
        #[command(subcommand)]
        output_command: OutputCommands,
    },
    /// Run design rule check
    Drc,
    /// Run electrical rule check
    Erc,
    /// Show PCB conductor status
    Status,
}

#[derive(Subcommand)]
enum OutputCommands {
    /// Export KiCad format
    Kicad,
    /// Export Gerber files
    Gerber,
    /// Export bill of materials
    Bom,
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
        Commands::Init => ("pcb.init", serde_json::json!({})),
        Commands::Build => ("pcb.build", serde_json::json!({})),
        Commands::AiSynthesize => ("pcb.ai_synthesize", serde_json::json!({})),
        Commands::Output { output_command } => match output_command {
            OutputCommands::Kicad => ("pcb.output.kicad", serde_json::json!({})),
            OutputCommands::Gerber => ("pcb.output.gerber", serde_json::json!({})),
            OutputCommands::Bom => ("pcb.output.bom", serde_json::json!({})),
        },
        Commands::Drc => ("pcb.drc", serde_json::json!({})),
        Commands::Erc => ("pcb.erc", serde_json::json!({})),
        Commands::Status => ("pcb.status", serde_json::json!({})),
    };

    let request = build_request(method, params);
    let payload = Payload::Structured(serde_json::to_value(&request)?);
    let response = client.send_to_conductor(ConductorId::Pcb, &payload).await?;

    output_result(&cli.common, method, &response);
    Ok(())
}

fn common_timeout_seconds(opts: &CommonOpts) -> Option<u64> {
    opts.timeout
}