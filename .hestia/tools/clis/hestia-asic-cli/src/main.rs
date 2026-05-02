//! hestia-asic-cli -- ASIC conductor CLI client

use anyhow::Result;
use clap::{Parser, Subcommand};
use conductor_sdk::agent::ConductorId;
use conductor_sdk::config::{CommonOpts, HestiaClientConfig};
use conductor_sdk::message::{MessageId, Payload, Request};
use conductor_sdk::transport::AgentCliClient;

#[derive(Parser)]
#[command(name = "hestia-asic-cli", version, about = "ASIC domain CLI")]
struct Cli {
    #[command(flatten)]
    common: CommonOpts,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize ASIC project workspace
    Init,
    /// Run ASIC build flow
    Build,
    /// PDK management
    Pdk {
        #[command(subcommand)]
        pdk_command: PdkCommands,
    },
    /// Advance to next design stage
    Advance,
    /// Run design rule check
    Drc,
    /// Run layout vs. schematic check
    Lvs,
    /// Show ASIC conductor status
    Status,
}

#[derive(Subcommand)]
enum PdkCommands {
    /// Install a PDK
    Install {
        /// PDK name
        name: String,
    },
    /// List available PDKs
    List,
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
        Commands::Init => ("asic.init", serde_json::json!({})),
        Commands::Build => ("asic.build", serde_json::json!({})),
        Commands::Pdk { pdk_command } => match pdk_command {
            PdkCommands::Install { name } => {
                ("asic.pdk.install", serde_json::json!({ "name": name }))
            }
            PdkCommands::List => ("asic.pdk.list", serde_json::json!({})),
        },
        Commands::Advance => ("asic.advance", serde_json::json!({})),
        Commands::Drc => ("asic.drc", serde_json::json!({})),
        Commands::Lvs => ("asic.lvs", serde_json::json!({})),
        Commands::Status => ("asic.status", serde_json::json!({})),
    };

    let request = build_request(method, params);
    let payload = Payload::Structured(serde_json::to_value(&request)?);
    let response = client.send_to_conductor(ConductorId::Asic, &payload).await?;

    output_result(&cli.common, method, &response);
    Ok(())
}

fn common_timeout_seconds(opts: &CommonOpts) -> Option<u64> {
    opts.timeout
}