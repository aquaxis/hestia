//! hestia-pcb-cli -- PCB conductor CLI client (in-process Handler invocation)

use anyhow::Result;
use clap::{Parser, Subcommand};
use conductor_sdk::config::CommonOpts;
use conductor_sdk::message::{MessageId, Request, Response};
use conductor_sdk::server::MessageHandler;
use hestia_pcb_conductor::handler::PcbHandler;

#[derive(Parser)]
#[command(name = "hestia-pcb-cli", version, about = "PCB domain CLI (in-process)")]
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
        Commands::Init => ("pcb.init", serde_json::json!({})),
        Commands::Build => ("pcb.build", serde_json::json!({})),
        Commands::AiSynthesize => ("pcb.ai_synthesize", serde_json::json!({})),
        Commands::Output { output_command } => match output_command {
            OutputCommands::Kicad => (
                "pcb.generate_output",
                serde_json::json!({ "format": "kicad" }),
            ),
            OutputCommands::Gerber => (
                "pcb.generate_output",
                serde_json::json!({ "format": "gerber" }),
            ),
            OutputCommands::Bom => ("pcb.generate_bom", serde_json::json!({})),
        },
        Commands::Drc => ("pcb.run_drc", serde_json::json!({})),
        Commands::Erc => ("pcb.run_erc", serde_json::json!({})),
        Commands::Status => ("pcb.status", serde_json::json!({})),
    };

    let request = build_request(method, params);
    let handler = PcbHandler;
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
