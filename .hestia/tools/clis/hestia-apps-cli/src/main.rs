//! hestia-apps-cli -- Apps conductor CLI client (in-process Handler invocation)

use anyhow::Result;
use clap::{Parser, Subcommand};
use conductor_sdk::config::CommonOpts;
use conductor_sdk::message::{MessageId, Request, Response};
use conductor_sdk::server::MessageHandler;
use hestia_apps_conductor::handler::AppsHandler;

#[derive(Parser)]
#[command(name = "hestia-apps-cli", version, about = "Apps domain CLI (in-process)")]
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
        Commands::Init => ("apps.init", serde_json::json!({})),
        Commands::Build => ("apps.build.v1", serde_json::json!({})),
        Commands::Flash => ("apps.flash.v1", serde_json::json!({})),
        Commands::Test { level } => ("apps.test.v1", serde_json::json!({ "level": level })),
        Commands::Size => ("apps.size.v1", serde_json::json!({})),
        Commands::Debug => ("apps.debug.v1", serde_json::json!({})),
        Commands::Status => ("apps.status", serde_json::json!({})),
    };

    let request = build_request(method, params);
    let handler = AppsHandler;
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
