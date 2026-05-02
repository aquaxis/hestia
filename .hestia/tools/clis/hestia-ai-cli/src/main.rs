//! hestia-ai-cli -- AI conductor CLI client
//!
//! Phase 16 改修: 2 系統サブコマンド構成
//! - `exec` / `spec.*` / `agent_*` / `container.*` / `workflow.*` / `status` →
//!   `AiHandler` を in-process で呼び出して構造化 JSON を即時返却
//! - `run --file` → agent-cli send で AI conductor (LLM) に投函し、
//!   `.hestia/run_log/<run-id>.json` のファイル出現を待機して結果を取得

use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use conductor_sdk::config::{CommonOpts, HestiaClientConfig};
use conductor_sdk::message::{MessageId, Request, Response};
use conductor_sdk::server::MessageHandler;
use hestia_ai_conductor::handler::AiHandler;
use rand::Rng;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

#[derive(Parser)]
#[command(name = "hestia-ai-cli", version, about = "AI conductor CLI")]
struct Cli {
    #[command(flatten)]
    common: CommonOpts,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Execute a natural language or structured instruction (in-process AiHandler)
    Exec {
        /// Instruction text to execute
        instruction: String,
    },
    /// Run an instruction from a file via AI conductor LLM orchestration (agent-cli send + result file polling)
    Run {
        /// Path to instruction file
        #[arg(long, short)]
        file: String,
        /// Polling timeout in seconds (default: 600)
        #[arg(long, default_value_t = 600)]
        timeout_secs: u64,
        /// Polling interval in milliseconds (default: 500)
        #[arg(long, default_value_t = 500)]
        poll_interval_ms: u64,
    },
    /// Initialize a specification session
    SpecInit {
        /// Specification text (natural language or structured)
        spec_text: Option<String>,
        /// Format of the specification (default: natural)
        #[arg(long, default_value = "natural")]
        format: String,
    },
    /// Update an existing specification
    SpecUpdate,
    /// Start a specification review
    SpecReview,
    /// List registered sub-agents
    AgentLs,
    /// List containers
    ContainerLs,
    /// Start a container
    ContainerStart {
        /// Container name
        name: String,
    },
    /// Stop a container
    ContainerStop {
        /// Container name
        name: String,
    },
    /// Create a container from container.toml
    ContainerCreate {
        /// Container name
        name: String,
    },
    /// Run a workflow
    WorkflowRun {
        /// Workflow name
        name: String,
    },
    /// Start a review
    ReviewStart,
    /// Show AI conductor status
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

fn generate_run_id() -> String {
    let now = chrono::Utc::now();
    let mut rng = rand::thread_rng();
    let suffix: String = (0..8)
        .map(|_| {
            let n: u8 = rng.gen_range(0..36);
            if n < 10 {
                (b'0' + n) as char
            } else {
                (b'a' + (n - 10)) as char
            }
        })
        .collect();
    format!("{}-{}", now.format("%Y%m%dT%H%M%SZ"), suffix)
}

fn run_log_dir() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".hestia/run_log")
}

/// `run --file` 経路: agent-cli send で AI conductor LLM に投函 → 結果ファイル待機
async fn run_with_orchestrator(
    common: &CommonOpts,
    file_path: &str,
    timeout_secs: u64,
    poll_interval_ms: u64,
) -> Result<()> {
    let body = std::fs::read_to_string(file_path)
        .map_err(|e| anyhow!("failed to read instruction file '{}': {e}", file_path))?;

    let run_id = generate_run_id();
    let log_dir = run_log_dir();
    std::fs::create_dir_all(&log_dir)
        .map_err(|e| anyhow!("failed to create run_log dir {}: {e}", log_dir.display()))?;
    let result_path = log_dir.join(format!("{run_id}.json"));
    let result_path_str = result_path.to_string_lossy().to_string();

    let prompt = format!(
        "RUN_ID: {run_id}\nRESULT_PATH: {result_path_str}\nINSTRUCTION:\n{body}"
    );

    if common.verbose {
        eprintln!("[ai.run] sending prompt to ai conductor (run_id={run_id})");
        eprintln!("[ai.run] result_path={result_path_str}");
    }

    let status = std::process::Command::new("agent-cli")
        .args(["send", "ai", &prompt])
        .status()
        .map_err(|e| anyhow!("failed to invoke agent-cli send: {e}"))?;
    if !status.success() {
        return Err(anyhow!(
            "agent-cli send exited with non-zero status: {status}"
        ));
    }

    let deadline = Instant::now() + Duration::from_secs(timeout_secs);
    let interval = Duration::from_millis(poll_interval_ms);
    while !result_path.exists() {
        if Instant::now() >= deadline {
            return Err(anyhow!(
                "timeout waiting for result file: {result_path_str} (after {timeout_secs}s)"
            ));
        }
        tokio::time::sleep(interval).await;
    }

    let content = std::fs::read_to_string(&result_path)
        .map_err(|e| anyhow!("failed to read result file {}: {e}", result_path.display()))?;
    let value: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| anyhow!("failed to parse result JSON {}: {e}", result_path.display()))?;

    let status_field = value.get("status").and_then(|v| v.as_str()).unwrap_or("unknown");
    emit(common, "ai.run", &value, status_field == "error")?;
    if status_field == "error" {
        std::process::exit(1);
    }
    Ok(())
}

/// in-process 経路: AiHandler を直接呼び出して即時応答
async fn run_in_process(
    common: &CommonOpts,
    method: &str,
    params: serde_json::Value,
) -> Result<()> {
    let request = build_request(method, params);
    let handler = AiHandler::new(HestiaClientConfig::default());
    match handler.handle_request(request).await {
        Response::Success(s) => {
            emit(common, method, &s.result, false)?;
            Ok(())
        }
        Response::Error(e) => {
            let err_value = serde_json::to_value(&e.error)?;
            emit(common, method, &err_value, true)?;
            std::process::exit(1);
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.common.verbose {
        let _ = tracing_subscriber::fmt::try_init();
    }

    if let Some(ref config_path) = cli.common.config {
        // 設定ファイルが指定された場合は読み込みのみ実行（in-process Handler は使わない）
        if !Path::new(config_path).exists() {
            return Err(anyhow!("config file not found: {config_path}"));
        }
    }

    match &cli.command {
        Commands::Run {
            file,
            timeout_secs,
            poll_interval_ms,
        } => {
            run_with_orchestrator(&cli.common, file, *timeout_secs, *poll_interval_ms).await
        }
        Commands::Exec { instruction } => {
            run_in_process(
                &cli.common,
                "ai.exec",
                serde_json::json!({ "instruction": instruction }),
            )
            .await
        }
        Commands::SpecInit { spec_text, format } => {
            run_in_process(
                &cli.common,
                "ai.spec.init",
                serde_json::json!({
                    "spec_text": spec_text.as_deref().unwrap_or(""),
                    "format": format,
                }),
            )
            .await
        }
        Commands::SpecUpdate => {
            run_in_process(&cli.common, "ai.spec.update", serde_json::json!({})).await
        }
        Commands::SpecReview => {
            run_in_process(&cli.common, "ai.spec.review", serde_json::json!({})).await
        }
        Commands::AgentLs => run_in_process(&cli.common, "agent_list", serde_json::json!({})).await,
        Commands::ContainerLs => {
            run_in_process(&cli.common, "container.list", serde_json::json!({})).await
        }
        Commands::ContainerStart { name } => {
            run_in_process(
                &cli.common,
                "container.start",
                serde_json::json!({ "name": name }),
            )
            .await
        }
        Commands::ContainerStop { name } => {
            run_in_process(
                &cli.common,
                "container.stop",
                serde_json::json!({ "name": name }),
            )
            .await
        }
        Commands::ContainerCreate { name } => {
            run_in_process(
                &cli.common,
                "container.create",
                serde_json::json!({ "name": name }),
            )
            .await
        }
        Commands::WorkflowRun { name } => {
            run_in_process(
                &cli.common,
                "meta.dualBuild",
                serde_json::json!({ "workflow": name }),
            )
            .await
        }
        Commands::ReviewStart => {
            run_in_process(&cli.common, "ai.spec.review", serde_json::json!({})).await
        }
        Commands::Status => {
            run_in_process(&cli.common, "system.health.v1", serde_json::json!({})).await
        }
    }
}
