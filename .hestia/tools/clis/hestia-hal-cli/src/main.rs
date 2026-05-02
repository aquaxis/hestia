//! hestia-hal-cli -- HAL conductor CLI client (in-process Handler invocation)

use anyhow::Result;
use clap::{Parser, Subcommand};
use conductor_sdk::config::CommonOpts;
use conductor_sdk::message::{MessageId, Request, Response};
use conductor_sdk::server::MessageHandler;
use hestia_hal_conductor::handler::HalHandler;

#[derive(Parser)]
#[command(name = "hestia-hal-cli", version, about = "HAL domain CLI (in-process)")]
struct Cli {
    #[command(flatten)]
    common: CommonOpts,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize HAL project workspace
    Init,
    /// Parse HAL specification
    Parse,
    /// Validate HAL specification
    Validate,
    /// Generate output from HAL
    Generate {
        /// Output language: c | rust | python | svd
        language: String,
    },
    /// Export HAL to RTL
    ExportRtl,
    /// Diff HAL specification versions
    Diff,
    /// Show HAL conductor status
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
        Commands::Init => ("hal.init", serde_json::json!({})),
        Commands::Parse => ("hal.parse.v1", serde_json::json!({})),
        Commands::Validate => ("hal.validate.v1", serde_json::json!({})),
        Commands::Generate { language } => (
            "hal.generate.v1",
            serde_json::json!({ "language": language }),
        ),
        Commands::ExportRtl => ("hal.export.v1", serde_json::json!({})),
        Commands::Diff => ("hal.diff.v1", serde_json::json!({})),
        Commands::Status => ("hal.status", serde_json::json!({})),
    };

    let request = build_request(method, params);
    let handler = HalHandler;
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
