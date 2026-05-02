//! hestia-asic-cli -- ASIC conductor CLI client (in-process Handler invocation)

use anyhow::Result;
use clap::{Parser, Subcommand};
use conductor_sdk::config::CommonOpts;
use conductor_sdk::message::{MessageId, Request, Response};
use conductor_sdk::server::MessageHandler;
use hestia_asic_conductor::handler::AsicHandler;

#[derive(Parser)]
#[command(name = "hestia-asic-cli", version, about = "ASIC domain CLI (in-process)")]
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
        Commands::Init => ("asic.init", serde_json::json!({})),
        Commands::Build => ("asic.build", serde_json::json!({})),
        Commands::Pdk { pdk_command } => match pdk_command {
            PdkCommands::Install { name } => (
                "asic.pdk.install",
                serde_json::json!({ "name": name }),
            ),
            PdkCommands::List => ("asic.pdk.list", serde_json::json!({})),
        },
        Commands::Advance => ("asic.advance", serde_json::json!({})),
        Commands::Drc => ("asic.drc", serde_json::json!({})),
        Commands::Lvs => ("asic.lvs", serde_json::json!({})),
        Commands::Status => ("asic.status", serde_json::json!({})),
    };

    let request = build_request(method, params);
    let handler = AsicHandler;
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
