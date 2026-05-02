//! hestia-rtl-cli -- RTL conductor CLI client (in-process Handler invocation)
//!
//! Phase 16 改修: AgentCliClient 経由 IPC ではなく、`hestia-rtl-conductor` の
//! `RtlHandler` を in-process で呼び出して構造化 JSON を stdout に返す。
//! AI conductor LLM が shell ツール経由でこの CLI を起動して結果を集約する。

use anyhow::Result;
use clap::{Parser, Subcommand};
use conductor_sdk::config::CommonOpts;
use conductor_sdk::message::{MessageId, Request, Response};
use conductor_sdk::server::MessageHandler;
use hestia_rtl_conductor::handler::RtlHandler;

#[derive(Parser)]
#[command(name = "hestia-rtl-cli", version, about = "RTL domain CLI (in-process)")]
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
        Commands::Init => ("rtl.init", serde_json::json!({})),
        Commands::Lint => ("rtl.lint.v1", serde_json::json!({})),
        Commands::Simulate => ("rtl.simulate.v1", serde_json::json!({})),
        Commands::Formal => ("rtl.formal.v1", serde_json::json!({})),
        Commands::Transpile => ("rtl.transpile.v1", serde_json::json!({})),
        Commands::Handoff => ("rtl.handoff.v1", serde_json::json!({})),
        Commands::Status => ("rtl.status", serde_json::json!({})),
    };

    let request = build_request(method, params);
    let handler = RtlHandler;
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
