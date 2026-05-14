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
    /// Send an inline prompt to the AI conductor and print its reply.
    /// Same orchestration path as `run`: posts the prompt via `<engine> send ai`
    /// with a generated RUN_ID / RESULT_PATH envelope, then polls the run-log
    /// JSON until the conductor writes a terminal status. The reply's primary
    /// text payload (`answer` / `halt_message` / `summary` / `text`, with a
    /// pretty-printed JSON fallback) is printed to stdout in plain form.
    Qa {
        /// The prompt text. Positional; quote if it contains spaces.
        prompt: String,
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

/// Output shape for the shared polling loop.
///
/// `Run`: emit the full terminal JSON via `emit()` (preserves the `[ai.run]`
/// label decoration when `--output human`, raw JSON when `--output json`).
/// `Qa`: extract the response's primary text via `extract_qa_answer` and print
/// it to stdout as plain text — Q&A presentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RunMode {
    Run,
    Qa,
}

/// Pull the primary text payload out of a terminal-status run-log JSON.
///
/// Field-name preference order:
///   1. `answer`       — preferred plain-text reply field
///   2. `halt_message` — error / aborted case
///   3. `summary`      — some conductor variants use this
///   4. `text`         — generic fallback
///   5. Pretty-printed full JSON — last resort so the user sees *something*.
///
/// Empty-string values are skipped so the caller falls through to the next
/// candidate field.
pub(crate) fn extract_qa_answer(v: &serde_json::Value) -> String {
    for key in ["answer", "halt_message", "summary", "text"] {
        if let Some(s) = v.get(key).and_then(|x| x.as_str()) {
            if !s.trim().is_empty() {
                return s.to_string();
            }
        }
    }
    serde_json::to_string_pretty(v).unwrap_or_else(|_| v.to_string())
}

/// Shared `run --file` / `qa <prompt>` route: posts the prompt body to the AI
/// conductor LLM via `<engine> send ai`, polls the run-log JSON until the
/// conductor writes a terminal status, then surfaces the result in the
/// caller's preferred shape (`RunMode`).
///
/// The watchdog stack (timeout / heartbeat / phase-stall) was deleted per the
/// 2026-05-14 instruction; the loop now polls indefinitely until status leaves
/// `in_progress`. External signals (`hestia stop` → monitor-daemon SIGTERM →
/// agent-cli SIGKILL, or `hestia kill` → SIGKILL everything) are the supported
/// abort paths.
async fn run_with_orchestrator(
    common: &CommonOpts,
    body: String,
    poll_interval_ms: u64,
    no_wait: bool,
    mode: RunMode,
) -> Result<()> {
    let label = match mode {
        RunMode::Run => "ai.run",
        RunMode::Qa => "ai.qa",
    };

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

    // Phase 135 — fire-and-forget exit (Run only; Qa never sets no_wait).
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
        match mode {
            RunMode::Run => {
                emit(common, label, &value, is_error)?;
            }
            RunMode::Qa => {
                let text = extract_qa_answer(&value);
                if is_error {
                    eprintln!("{text}");
                } else {
                    println!("{text}");
                }
            }
        }
        std::io::stdout().flush().ok();
        std::io::stderr().flush().ok();
        if is_error {
            std::process::exit(1);
        }
        return Ok(());
    }
}

/// `hestia ai run --file …` thin wrapper: read the instruction body from the
/// file then dispatch into the shared polling loop in `RunMode::Run`.
async fn run_run(
    common: &CommonOpts,
    file_path: &str,
    poll_interval_ms: u64,
    no_wait: bool,
) -> Result<()> {
    let body = std::fs::read_to_string(file_path)
        .map_err(|e| anyhow!("failed to read instruction file '{}': {e}", file_path))?;
    run_with_orchestrator(common, body, poll_interval_ms, no_wait, RunMode::Run).await
}

/// `hestia ai qa <prompt>` thin wrapper: pass the prompt straight through as
/// the body and dispatch into the shared polling loop in `RunMode::Qa`.
/// `no_wait` is hard-coded false — fire-and-forget makes no sense for an
/// interactive Q&A.
async fn run_qa(
    common: &CommonOpts,
    prompt: &str,
    poll_interval_ms: u64,
) -> Result<()> {
    run_with_orchestrator(common, prompt.to_string(), poll_interval_ms, false, RunMode::Qa).await
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
            run_run(
                &cli.common,
                file,
                *poll_interval_ms,
                *no_wait,
            )
            .await
        }
        Commands::Qa {
            prompt,
            poll_interval_ms,
        } => {
            run_qa(
                &cli.common,
                prompt,
                *poll_interval_ms,
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
    use super::extract_qa_answer;

    #[test]
    fn extract_qa_answer_prefers_answer_field() {
        let v = serde_json::json!({
            "status": "ok",
            "answer": "the answer is 42",
            "halt_message": "ignored",
        });
        assert_eq!(extract_qa_answer(&v), "the answer is 42");
    }

    #[test]
    fn extract_qa_answer_falls_back_to_halt_message_on_error() {
        let v = serde_json::json!({
            "status": "error",
            "halt_message": "ai-conductor refused to answer",
        });
        assert_eq!(extract_qa_answer(&v), "ai-conductor refused to answer");
    }

    #[test]
    fn extract_qa_answer_uses_summary_then_text() {
        let v = serde_json::json!({
            "status": "ok",
            "summary": "S",
        });
        assert_eq!(extract_qa_answer(&v), "S");

        let v = serde_json::json!({
            "status": "ok",
            "text": "T",
        });
        assert_eq!(extract_qa_answer(&v), "T");
    }

    #[test]
    fn extract_qa_answer_skips_empty_strings_and_falls_through() {
        // `answer` is present but empty → should fall through to `halt_message`.
        let v = serde_json::json!({
            "status": "error",
            "answer": "   ",
            "halt_message": "real message here",
        });
        assert_eq!(extract_qa_answer(&v), "real message here");
    }

    #[test]
    fn extract_qa_answer_fallback_pretty_prints_full_json() {
        // None of the preferred fields are present → pretty-printed full JSON.
        let v = serde_json::json!({
            "status": "ok",
            "data": { "foo": 1 },
        });
        let out = extract_qa_answer(&v);
        assert!(out.starts_with("{"), "fallback must look like JSON: {out}");
        assert!(out.contains("\"data\""), "fallback must include the data key: {out}");
    }
}
