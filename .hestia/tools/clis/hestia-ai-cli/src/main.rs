//! hestia-ai-cli -- AI conductor CLI client
//!
//! Phase 16 revision: two subcommand lineages
//! - `exec` / `spec.*` / `agent_*` / `container.*` / `workflow.*` / `status` ->
//!   calls `AiHandler` in-process and returns structured JSON immediately
//! - `run --file` -> posts to AI conductor (LLM) via agent-cli send, then polls
//!   `.hestia/run_log/<run-id>.json` for the result file to appear

use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use conductor_sdk::config::{CommonOpts, HestiaClientConfig};
use conductor_sdk::message::{MessageId, Request, Response};
use conductor_sdk::server::MessageHandler;
use hestia_ai_conductor::handler::AiHandler;
use rand::Rng;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

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
    /// Run an instruction from a file via AI conductor LLM orchestration (agent-cli send + result file polling).
    /// Polls the run-log JSON until the conductor writes a terminal status
    /// (`ok` / `error` / anything other than `in_progress`). No automatic
    /// timeout — use `hestia stop` or `hestia kill` to abort a hung run.
    Run {
        /// Path to instruction file
        #[arg(long, short)]
        file: String,
        /// Polling interval in milliseconds (default: 500)
        #[arg(long, default_value_t = 500)]
        poll_interval_ms: u64,
        /// Fire-and-forget: after the prompt is sent to ai-conductor, emit the
        /// submission envelope ({status:"submitted", run_id, result_path}) and
        /// exit 0 without waiting for the result file. Inspect later via
        /// `hestia tail ai` or by reading `result_path`.
        #[arg(long)]
        no_wait: bool,
    },
    /// Send an inline prompt to the AI conductor via the in-process AiHandler
    /// (same dispatch path as `exec`). The handler returns a structured JSON
    /// response synchronously; it is printed in `human` or `json` form per
    /// `--output`. Set `HESTIA_DISABLE_AUTO_REVIEW=1` to suppress the post-run
    /// ai-reviewer auto-spawn for Q&A-only invocations.
    Qa {
        /// The prompt text. Positional; quote if it contains spaces.
        prompt: String,
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

/// Human-readable rendering of a success `Value`. Returns `Some` only when the
/// payload carries a string `answer` (today: `ai.qa`). The returned string is
/// the answer verbatim -- it already contains real newline characters; serde
/// decoded the wire `\n` when the conductor's JSON was parsed into this Value.
/// A concise trailing status line (after a blank line, no literal `\n`) keeps
/// `run_id`/`status` discoverable without corrupting the Markdown body.
/// `None` => caller keeps the existing `[label] {json}` line (back-compat for
/// every other subcommand whose result has no string `answer`).
fn human_render(value: &serde_json::Value) -> Option<String> {
    let answer = value.get("answer")?.as_str()?;
    let run_id = value.get("run_id").and_then(|v| v.as_str());
    let status = value.get("status").and_then(|v| v.as_str());
    let mut out = String::from(answer);
    match (status, run_id) {
        (Some(s), Some(r)) => out.push_str(&format!("\n\n[ai.qa] status={s} run_id={r}")),
        (Some(s), None) => out.push_str(&format!("\n\n[ai.qa] status={s}")),
        (None, Some(r)) => out.push_str(&format!("\n\n[ai.qa] run_id={r}")),
        (None, None) => {}
    }
    Some(out)
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
    } else if let Some(text) = human_render(value) {
        println!("{text}");
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

/// Phase 115 -- Resolve the engine binary name.
///
/// Priority order:
/// 1. env `HESTIA_ENGINE_BINARY` (exported by the hestia parent process)
/// 2. Direct read of the `[engine]` section from `.hestia/config.toml` in CWD
///    (ensures compatibility when hestia-ai-cli is invoked independently)
/// 3. `agent-cli` (backward-compatible fallback)
fn engine_binary() -> String {
    if let Ok(v) = std::env::var("HESTIA_ENGINE_BINARY") {
        if !v.is_empty() {
            return v;
        }
    }
    if let Some(b) = read_engine_from_config() {
        return b;
    }
    "agent-cli".to_string()
}

fn read_engine_from_config() -> Option<String> {
    let path = std::env::current_dir().ok()?.join(".hestia/config.toml");
    let text = std::fs::read_to_string(&path).ok()?;
    let v: toml::Value = toml::from_str(&text).ok()?;
    let engine = v.get("engine")?;
    if let Some(b) = engine.get("binary").and_then(|x| x.as_str()) {
        if !b.is_empty() {
            return Some(b.to_string());
        }
    }
    let typ = engine.get("type").and_then(|x| x.as_str())?;
    Some(match typ {
        "claude_cli_shim" => "claude-cli-shim".to_string(),
        _ => "agent-cli".to_string(),
    })
}

/// `hestia ai run --file …` route: posts the instruction body to the AI
/// conductor LLM via `<engine> send ai`, polls the run-log JSON until the
/// conductor writes a terminal status, then emits the full terminal JSON via
/// `emit()`.
///
/// The watchdog stack (timeout / heartbeat / phase-stall) was deleted per the
/// 2026-05-14 instruction; the loop now polls indefinitely until status leaves
/// `in_progress`. External signals (`hestia stop` → monitor-daemon SIGTERM →
/// agent-cli SIGKILL, or `hestia kill` → SIGKILL everything) are the supported
/// abort paths.
async fn run_with_orchestrator(
    common: &CommonOpts,
    file_path: &str,
    poll_interval_ms: u64,
    no_wait: bool,
) -> Result<()> {
    let label = "ai.run";

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
        eprintln!("[{label}] sending prompt to ai conductor (run_id={run_id})");
        eprintln!("[{label}] result_path={result_path_str}");
    }

    let engine_bin = engine_binary();
    let status = tokio::process::Command::new(&engine_bin)
        .args(["send", "ai", &prompt])
        .status()
        .await
        .map_err(|e| anyhow!("failed to invoke {engine_bin} send: {e}"))?;
    if !status.success() {
        return Err(anyhow!(
            "{engine_bin} send exited with non-zero status: {status}"
        ));
    }

    // Phase 135 — fire-and-forget exit.
    if no_wait {
        let envelope = serde_json::json!({
            "status": "submitted",
            "run_id": run_id,
            "result_path": result_path_str,
            "synthesized_by": "hestia-ai-cli",
            "note": "Prompt sent; not waiting for result. Inspect via `hestia tail ai` or read result_path later.",
        });
        emit(common, label, &envelope, false)?;
        std::io::stdout().flush().ok();
        std::io::stderr().flush().ok();
        return Ok(());
    }

    let interval = Duration::from_millis(poll_interval_ms);
    loop {
        tokio::time::sleep(interval).await;
        if !result_path.exists() {
            continue;
        }
        let parsed = std::fs::read_to_string(&result_path)
            .ok()
            .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok());
        let Some(v) = parsed else { continue };
        let status_field = v.get("status").and_then(|s| s.as_str()).unwrap_or("");
        if status_field == "in_progress" {
            continue;
        }
        // Terminal status — re-read with retry to avoid mid-write partial JSON.
        let value = read_result_with_retry(&result_path).await?;
        let status_field = value
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let is_error = status_field == "error";
        emit(common, label, &value, is_error)?;
        std::io::stdout().flush().ok();
        std::io::stderr().flush().ok();
        if is_error {
            std::process::exit(1);
        }
        return Ok(());
    }
}

/// To avoid JSON parse failures caused by reading a file while fs_write is
/// still in progress, retry up to 5 times until a stable complete JSON can be read.
async fn read_result_with_retry(path: &Path) -> Result<serde_json::Value> {
    let mut last_err: Option<String> = None;
    for _ in 0..5 {
        match std::fs::read_to_string(path) {
            Ok(content) if !content.trim().is_empty() => {
                match serde_json::from_str::<serde_json::Value>(&content) {
                    Ok(v) => return Ok(v),
                    Err(e) => last_err = Some(format!("parse error: {e}")),
                }
            }
            Ok(_) => last_err = Some("file empty".to_string()),
            Err(e) => last_err = Some(format!("read error: {e}")),
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    Err(anyhow!(
        "failed to read stable result JSON {} after retries: {}",
        path.display(),
        last_err.unwrap_or_else(|| "unknown".to_string())
    ))
}

/// In-process route: calls AiHandler directly and returns immediately
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
        // If a config file is specified, only read it (do not use in-process Handler)
        if !Path::new(config_path).exists() {
            return Err(anyhow!("config file not found: {config_path}"));
        }
    }

    match &cli.command {
        Commands::Run {
            file,
            poll_interval_ms,
            no_wait,
        } => {
            run_with_orchestrator(
                &cli.common,
                file,
                *poll_interval_ms,
                *no_wait,
            )
            .await
        }
        Commands::Qa { prompt } => {
            run_in_process(
                &cli.common,
                "ai.qa",
                serde_json::json!({ "instruction": prompt }),
            )
            .await
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

#[cfg(test)]
mod tests {
    use super::human_render;
    use serde_json::json;

    // AC4 case (a): a value with an `answer` containing `\n` renders to
    // multi-line text -- real line breaks, no `{`/`}` wrapping the body,
    // no two-character `\n` sequence in the report body.
    #[test]
    fn answer_renders_as_multiline_text() {
        let v = json!({
            "status": "ok",
            "answer": "# Title\nline A\n\n## Sub\nline B",
            "run_id": "qa-x",
            "kind": "qa"
        });
        let out = human_render(&v).expect("answer present => Some");
        let (body, _footer) = out.split_once("\n\n[ai.qa] ").expect("status footer appended");
        assert_eq!(body, "# Title\nline A\n\n## Sub\nline B");
        assert!(body.contains('\n'), "real newline characters present");
        assert!(!body.contains("\\n"), "no literal backslash-n in the body");
        assert!(!body.starts_with('{') && !body.ends_with('}'), "not JSON-wrapped");
        assert_eq!(out.lines().next().unwrap(), "# Title");
        // run_id/status remain discoverable on the trailing line (D5).
        assert!(out.ends_with("[ai.qa] status=ok run_id=qa-x"));
    }

    // AC4 case (b): a value WITHOUT a string `answer` => None, so the caller
    // keeps the existing `[label] {json}` line (no collateral change to
    // non-ai.qa subcommands -- D4).
    #[test]
    fn no_answer_field_returns_none() {
        assert!(human_render(&json!({"status": "ok", "result": 42})).is_none());
        assert!(human_render(&json!({"answer": 123})).is_none(), "non-string answer => None");
        assert!(human_render(&json!("plain string")).is_none());
        assert!(human_render(&json!([1, 2, 3])).is_none());
    }

    // AC4 case (c): the `--output json` machine path is byte-unchanged --
    // serde_json::to_string of the value (what emit's json branch prints)
    // is exactly the original compact JSON, independent of human_render.
    #[test]
    fn json_mode_serialization_unchanged() {
        let v = json!({"status":"ok","answer":"a\nb","run_id":"qa-x","kind":"qa"});
        assert_eq!(
            serde_json::to_string(&v).unwrap(),
            r#"{"answer":"a\nb","kind":"qa","run_id":"qa-x","status":"ok"}"#
        );
    }
}

