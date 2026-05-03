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
        /// Target device (e.g. xilinx, intel, lattice, artix7)
        target: String,
        /// Top module name (overrides auto-detection)
        #[arg(long)]
        top: Option<String>,
        /// Part number (overrides project template)
        #[arg(long)]
        part: Option<String>,
        /// Constraints file (overrides project template)
        #[arg(long)]
        constraints: Option<String>,
        /// Actually invoke Vivado batch (long-running). Without this, only emits TCL/manifest.
        #[arg(long)]
        execute: bool,
    },
    /// Run logic synthesis
    Synthesize,
    /// Run place-and-route implementation
    Implement,
    /// Generate bitstream
    Bitstream,
    /// Run FPGA simulation
    Simulate,
    /// Program target device with a bitstream
    Program {
        /// Bitstream path (default: first .bit in <root>/fpga/output/)
        #[arg(long)]
        bitstream: Option<String>,
        /// Target device hint (passed to template)
        #[arg(long, default_value = "")]
        device: String,
        /// Actually invoke Vivado JTAG programming
        #[arg(long)]
        execute: bool,
    },
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
        Commands::Build { target, top, part, constraints, execute } => {
            let mut p = serde_json::json!({ "target": target, "execute": execute });
            if let Some(v) = top { p["top"] = serde_json::json!(v); }
            if let Some(v) = part { p["part"] = serde_json::json!(v); }
            if let Some(v) = constraints { p["constraints"] = serde_json::json!(v); }
            ("fpga.build.v1.start", p)
        }
        Commands::Synthesize => ("fpga.synthesize", serde_json::json!({})),
        Commands::Implement => ("fpga.implement", serde_json::json!({})),
        Commands::Bitstream => ("fpga.bitstream", serde_json::json!({})),
        Commands::Simulate => ("fpga.simulate", serde_json::json!({})),
        Commands::Program { bitstream, device, execute } => {
            let mut p = serde_json::json!({ "device": device, "execute": execute });
            if let Some(v) = bitstream { p["bitstream"] = serde_json::json!(v); }
            ("fpga.program", p)
        }
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
