//! hestia-fpga-cli -- FPGA conductor CLI client (in-process Handler invocation)

use anyhow::Result;
use clap::{Parser, Subcommand};
use conductor_sdk::config::CommonOpts;
use conductor_sdk::message::{MessageId, Request, Response};
use conductor_sdk::server::MessageHandler;
use hestia_fpga_conductor::handler::FpgaHandler;

#[derive(Parser)]
#[command(name = "hestia-fpga-cli", version, about = "FPGA domain CLI (in-process)")]
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
        kind: "prompt".to_string(),
        from: "cli".to_string(),
        method: method.to_string(),
        params,
        id: MessageId::new(),
        trace_id: None,
    }
}

fn emit(common: &CommonOpts, label: &str, value: &serde_json::Value, is_error: bool) -> Result<()> {
    let json = serde_json::to_string(value)?;
    if common.output == "json" {
        if is_error {
            eprintln!("{}", json);
        } else {
            println!("{}", json);
        }
    } else if is_error {
        eprintln!("[{label}] error: {json}");
    } else {
        println!("[{label}] {json}");
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    if cli.common.verbose {
        let _ = tracing_subscriber::fmt::try_init();
    }

    let (method, params) = match &cli.command {
        Commands::Init => ("fpga.init", serde_json::json!({})),
        Commands::Build { target } => (
            "fpga.build.v1.start",
            serde_json::json!({ "target": target }),
        ),
        Commands::Synthesize => ("fpga.synthesize", serde_json::json!({})),
        Commands::Implement => ("fpga.implement", serde_json::json!({})),
        Commands::Bitstream => ("fpga.bitstream", serde_json::json!({})),
        Commands::Simulate => ("fpga.simulate", serde_json::json!({})),
        Commands::Program => ("fpga.program", serde_json::json!({})),
        Commands::Report { report_type } => (
            match report_type.as_str() {
                "timing" => "report_timing",
                "resource" => "report_resource",
                _ => "report_messages",
            },
            serde_json::json!({ "report_type": report_type }),
        ),
        Commands::Status => ("fpga.status", serde_json::json!({})),
    };

    let request = build_request(method, params);
    let handler = FpgaHandler;
    match handler.handle_request(request).await {
        Response::Success(s) => {
            emit(&cli.common, method, &s.result, false)?;
            Ok(())
        }
        Response::Error(e) => {
            let err_value = serde_json::to_value(&e.error)?;
            emit(&cli.common, method, &err_value, true)?;
            std::process::exit(1);
        }
    }
}
