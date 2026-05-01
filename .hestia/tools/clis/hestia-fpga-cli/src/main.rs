//! hestia-fpga-cli -- FPGA conductor CLI client

use anyhow::Result;
use clap::{Parser, Subcommand};
use conductor_sdk::agent::ConductorId;
use conductor_sdk::config::{CommonOpts, HestiaClientConfig};
use conductor_sdk::message::{MessageId, Payload, Request};
use conductor_sdk::transport::AgentCliClient;

#[derive(Parser)]
#[command(name = "hestia-fpga-cli", version, about = "FPGA domain CLI")]
struct Cli {
    #[command(flatten)]
    common: CommonOpts,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize FPGA project workspace
    Init,
    /// Build FPGA design for a target device
    Build {
        /// Target device (e.g. xilinx, intel, lattice)
        target: String,
    },
    /// Run logic synthesis
    Synthesize,
    /// Run place-and-route implementation
    Implement,
    /// Generate bitstream
    Bitstream,
    /// Run FPGA simulation
    Simulate,
    /// Program target device
    Program,
    /// Show reports
    Report {
        /// Report type: timing | resource
        report_type: String,
    },
    /// Show FPGA conductor status
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

    let (method, params) = match &cli.command {
        Commands::Init => ("fpga.init", serde_json::json!({})),
        Commands::Build { target } => ("fpga.build", serde_json::json!({ "target": target })),
        Commands::Synthesize => ("fpga.synthesize", serde_json::json!({})),
        Commands::Implement => ("fpga.implement", serde_json::json!({})),
        Commands::Bitstream => ("fpga.bitstream", serde_json::json!({})),
        Commands::Simulate => ("fpga.simulate", serde_json::json!({})),
        Commands::Program => ("fpga.program", serde_json::json!({})),
        Commands::Report { report_type } => (
            "fpga.report",
            serde_json::json!({ "report_type": report_type }),
        ),
        Commands::Status => ("fpga.status", serde_json::json!({})),
    };

    let request = build_request(method, params);
    let payload = Payload::Structured(serde_json::to_value(&request)?);
    let response = client.send_to_conductor(ConductorId::Fpga, &payload).await?;

    output_result(&cli.common, method, &response);
    Ok(())
}

fn common_timeout_seconds(opts: &CommonOpts) -> Option<u64> {
    opts.timeout
}