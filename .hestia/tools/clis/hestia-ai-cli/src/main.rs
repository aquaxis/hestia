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
use std::time::{Duration, Instant, SystemTime};

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
        /// Polling timeout in seconds (default: 7200 = 2 h). Most callers should
        /// rely on `--heartbeat-secs` for the live-but-slow detector rather than
        /// tightening this knob. See report_stop.md §6.1 R1-1.
        #[arg(long, default_value_t = 7200)]
        timeout_secs: u64,
        /// Maximum on-disk quiescence (seconds) tolerated before declaring a
        /// stall. Default: timeout_secs / 4, clamped to [60, 3600]. While the
        /// conductor keeps writing status:in_progress JSON to the run-log, the
        /// watchdog does not fire even if elapsed time exceeds `timeout_secs`.
        #[arg(long)]
        heartbeat_secs: Option<u64>,
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

/// `run --file` route: posts to AI conductor LLM via <engine> send -> polls for result file
async fn run_with_orchestrator(
    common: &CommonOpts,
    file_path: &str,
    timeout_secs: u64,
    heartbeat_secs: Option<u64>,
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

    // Phase 115 -- Eliminated hardcoded agent-cli. Uses the binary resolved
    // from HESTIA_ENGINE_BINARY env or .hestia/config.toml [engine]
    // (e.g. `claude-cli-shim send` when using the claude_cli_shim engine).
    let engine_bin = engine_binary();
    // Run asynchronously via tokio::process::Command to avoid blocking the async runtime
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

    // Phase 132 (post-incident, report_stop.md §6) — heartbeat-aware polling.
    // The old `while !result_path.exists()` loop conflated two distinct
    // contracts: it assumed the conductor would write the run-log JSON only at
    // terminal completion, and treated absence as silence. In practice the
    // conductor writes status:in_progress checkpoints throughout the run, so
    // the loop now:
    //   - waits for the file to appear within `deadline`,
    //   - once it appears, returns Done only when status != "in_progress",
    //   - treats mtime advancement as a heartbeat (no overall deadline while
    //     the conductor keeps making progress),
    //   - declares a stall only when on-disk content stops advancing for
    //     longer than `heartbeat`.
    let deadline = Instant::now() + Duration::from_secs(timeout_secs);
    let interval = Duration::from_millis(poll_interval_ms);
    let heartbeat = compute_heartbeat(timeout_secs, heartbeat_secs);

    let mut last_progress_at = Instant::now();
    let mut last_seen_mtime: Option<SystemTime> = None;
    let mut cached_status: Option<String> = None;

    loop {
        let file_exists = result_path.exists();
        let file_mtime = std::fs::metadata(&result_path)
            .ok()
            .and_then(|m| m.modified().ok());

        let advanced = match (file_mtime, last_seen_mtime) {
            (Some(cur), Some(prev)) => cur > prev,
            (Some(_), None) => true,
            _ => false,
        };
        if advanced {
            last_progress_at = Instant::now();
            last_seen_mtime = file_mtime;
            // Re-parse on-disk status only when mtime moved, to avoid burning
            // CPU re-reading a quiescent file thousands of times.
            cached_status = std::fs::read_to_string(&result_path)
                .ok()
                .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
                .and_then(|v| {
                    v.get("status")
                        .and_then(|s| s.as_str())
                        .map(|s| s.to_string())
                });
        }

        let now = Instant::now();
        let quiescent_for = now.saturating_duration_since(last_progress_at);

        match classify_poll(
            now,
            deadline,
            file_exists,
            quiescent_for,
            heartbeat,
            cached_status.as_deref(),
        ) {
            PollDecision::Done => break,
            PollDecision::KeepPolling => {
                tokio::time::sleep(interval).await;
                continue;
            }
            PollDecision::Stalled { .. } | PollDecision::DeadlineWithoutFile => {
                // Phase 132 — synthesize a halt aggregate that carries the on-disk
                // evidence (current_phase, updated_at, task_status, …) rather than
                // speculating about agent-cli's iteration budget. See
                // report_stop.md §6.2 R2-3.
                let on_disk = if file_exists {
                    read_result_with_retry(&result_path).await.ok()
                } else {
                    None
                };
                let synthetic = build_halt_aggregate(
                    &run_id,
                    &body,
                    &result_path_str,
                    timeout_secs,
                    on_disk.as_ref(),
                );
                if let Ok(serialized) = serde_json::to_string_pretty(&synthetic) {
                    let _ = std::fs::write(&result_path, serialized);
                }
                emit(common, "ai.run", &synthetic, true)?;
                std::io::stdout().flush().ok();
                std::io::stderr().flush().ok();
                std::process::exit(1);
            }
        }
    }

    // Avoid file write race condition: retry until stable content is available.
    // Retry up to 5 times (total 1 second) until content ends with `}` and
    // JSON parsing succeeds.
    let value = read_result_with_retry(&result_path).await?;

    let status_field = value
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    emit(common, "ai.run", &value, status_field == "error")?;
    // Flush stdout reliably before exit (ensures reliability through pipe/redirect paths)
    std::io::stdout().flush().ok();
    std::io::stderr().flush().ok();
    if status_field == "error" {
        std::process::exit(1);
    }
    Ok(())
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

/// Phase 132 (post-incident, report_stop.md §6.2 R2-1) — Outcome of one tick of
/// the heartbeat-aware polling loop in `run_with_orchestrator`.
#[derive(Debug, PartialEq)]
pub(crate) enum PollDecision {
    KeepPolling,
    Done,
    Stalled { quiescent_for: Duration },
    DeadlineWithoutFile,
}

/// Phase 132 (post-incident) — Classify a single poll tick.
///
/// Pure function (no I/O); the caller is responsible for stat()'ing the file
/// and tracking mtime advancement to compute `quiescent_for`. Truth table is
/// AI_PRJ_DESIGN.md §3.2.
pub(crate) fn classify_poll(
    now: Instant,
    deadline: Instant,
    file_exists: bool,
    quiescent_for: Duration,
    heartbeat: Duration,
    on_disk_status: Option<&str>,
) -> PollDecision {
    if !file_exists {
        return if now >= deadline {
            PollDecision::DeadlineWithoutFile
        } else {
            PollDecision::KeepPolling
        };
    }
    if matches!(on_disk_status, Some(s) if s != "in_progress") {
        return PollDecision::Done;
    }
    if quiescent_for > heartbeat {
        return PollDecision::Stalled { quiescent_for };
    }
    PollDecision::KeepPolling
}

/// Phase 132 (post-incident) — Resolve the heartbeat window. When the user
/// doesn't pass `--heartbeat-secs`, default to `timeout_secs / 4` clamped to
/// `[60, 3600]`. See AI_PRJ_DESIGN.md §3.1.
pub(crate) fn compute_heartbeat(timeout_secs: u64, override_secs: Option<u64>) -> Duration {
    let secs = override_secs
        .unwrap_or_else(|| (timeout_secs / 4).clamp(60, 3600))
        .max(1);
    Duration::from_secs(secs)
}

/// Phase 132 (post-incident, report_stop.md §6.2 R2-3) — Build the halt-aggregate
/// JSON emitted when the watchdog fires. When `on_disk` is `Some`, carry
/// through the in-flight evidence (`current_phase`, `updated_at`, `task_status`,
/// `conductors_spawned`, `phases_completed`) and word the halt message around
/// those facts — replacing the old speculative blame about agent-cli's
/// iteration budget, which the incident logs in report_stop.md showed to be
/// demonstrably wrong for the run that motivated this fix.
///
/// Preserves `synthesized_by`, `halted_reason`, and `aborted_reason` (legacy
/// alias) for downstream parsers.
pub(crate) fn build_halt_aggregate(
    run_id: &str,
    instruction: &str,
    result_path: &str,
    timeout_secs: u64,
    on_disk: Option<&serde_json::Value>,
) -> serde_json::Value {
    use serde_json::Value;
    let mut obj = serde_json::Map::new();
    obj.insert("run_id".to_string(), Value::String(run_id.to_string()));
    obj.insert("status".to_string(), Value::String("error".into()));
    obj.insert(
        "instruction".to_string(),
        Value::String(instruction.to_string()),
    );
    obj.insert("halted_reason".to_string(), Value::String("timeout".into()));
    obj.insert("aborted_reason".to_string(), Value::String("timeout".into()));
    obj.insert(
        "synthesized_by".to_string(),
        Value::String("hestia-ai-cli".into()),
    );

    let halt_message = match on_disk {
        Some(v) => {
            let status = v.get("status").and_then(|s| s.as_str()).unwrap_or("unknown");
            let phase = v
                .get("current_phase")
                .and_then(|s| s.as_str())
                .unwrap_or("(none recorded)");
            let updated_at = v
                .get("updated_at")
                .and_then(|s| s.as_str())
                .unwrap_or("(no updated_at)");
            format!(
                "Watchdog fired after {timeout_secs}s while {result_path} was still \
                 status:{status} at phase {phase} (last updated_at={updated_at}). \
                 The conductor and sub-agents may still be running; inspect \
                 `agent-cli list` and `hestia tail ai` before declaring failure."
            )
        }
        None => format!(
            "AI conductor did not write {result_path} within {timeout_secs}s; \
             the file does not exist. Run `hestia tail ai` to inspect orchestrator activity."
        ),
    };
    obj.insert("halt_message".to_string(), Value::String(halt_message));
    obj.insert(
        "aborted_message".to_string(),
        Value::String(format!(
            "AI conductor watchdog fired after {timeout_secs}s for {result_path}."
        )),
    );

    if let Some(v) = on_disk {
        for key in [
            "current_phase",
            "updated_at",
            "task_status",
            "conductors_spawned",
            "phases_completed",
        ] {
            if let Some(val) = v.get(key) {
                obj.insert(key.to_string(), val.clone());
            }
        }
    }

    obj.insert("workflow_steps".to_string(), Value::Array(vec![]));
    obj.insert("results".to_string(), Value::Array(vec![]));
    Value::Object(obj)
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
            timeout_secs,
            heartbeat_secs,
            poll_interval_ms,
        } => {
            run_with_orchestrator(
                &cli.common,
                file,
                *timeout_secs,
                *heartbeat_secs,
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
    use super::*;

    fn t0() -> Instant {
        Instant::now()
    }

    #[test]
    fn classify_poll_keeps_polling_when_file_absent_within_deadline() {
        let now = t0();
        let d = classify_poll(
            now,
            now + Duration::from_secs(60),
            false,
            Duration::from_secs(0),
            Duration::from_secs(10),
            None,
        );
        assert_eq!(d, PollDecision::KeepPolling);
    }

    #[test]
    fn classify_poll_deadline_without_file_when_past_deadline() {
        let now = t0();
        let d = classify_poll(
            now + Duration::from_secs(120),
            now + Duration::from_secs(60),
            false,
            Duration::from_secs(0),
            Duration::from_secs(10),
            None,
        );
        assert_eq!(d, PollDecision::DeadlineWithoutFile);
    }

    #[test]
    fn classify_poll_done_on_terminal_status() {
        let now = t0();
        let d = classify_poll(
            now,
            now + Duration::from_secs(60),
            true,
            Duration::from_secs(0),
            Duration::from_secs(10),
            Some("completed"),
        );
        assert_eq!(d, PollDecision::Done);
    }

    #[test]
    fn classify_poll_done_treats_error_as_terminal() {
        let now = t0();
        let d = classify_poll(
            now,
            now + Duration::from_secs(60),
            true,
            Duration::from_secs(0),
            Duration::from_secs(10),
            Some("error"),
        );
        assert_eq!(d, PollDecision::Done);
    }

    #[test]
    fn classify_poll_keeps_polling_on_in_progress_within_heartbeat() {
        let now = t0();
        let d = classify_poll(
            now,
            now + Duration::from_secs(60),
            true,
            Duration::from_secs(5),
            Duration::from_secs(10),
            Some("in_progress"),
        );
        assert_eq!(d, PollDecision::KeepPolling);
    }

    #[test]
    fn classify_poll_stalls_when_in_progress_quiescent_past_heartbeat() {
        let now = t0();
        let d = classify_poll(
            now,
            now + Duration::from_secs(60),
            true,
            Duration::from_secs(30),
            Duration::from_secs(10),
            Some("in_progress"),
        );
        assert!(matches!(d, PollDecision::Stalled { .. }));
    }

    #[test]
    fn classify_poll_stalls_when_parse_failed_and_quiescent() {
        // File exists but parse failed (transient or malformed): treat like
        // in_progress (not Done) so the watchdog can still notice a true stall.
        let now = t0();
        let d = classify_poll(
            now,
            now + Duration::from_secs(60),
            true,
            Duration::from_secs(30),
            Duration::from_secs(10),
            None,
        );
        assert!(matches!(d, PollDecision::Stalled { .. }));
    }

    #[test]
    fn classify_poll_ignores_deadline_while_heartbeat_alive() {
        // Past the overall deadline, but file exists and recent mtime + terminal
        // status — caller should see Done, not DeadlineWithoutFile.
        let now = t0();
        let d = classify_poll(
            now + Duration::from_secs(120),
            now + Duration::from_secs(60),
            true,
            Duration::from_secs(2),
            Duration::from_secs(10),
            Some("completed"),
        );
        assert_eq!(d, PollDecision::Done);
    }

    #[test]
    fn compute_heartbeat_uses_quarter_of_timeout() {
        assert_eq!(compute_heartbeat(7200, None), Duration::from_secs(1800));
    }

    #[test]
    fn compute_heartbeat_clamps_low() {
        assert_eq!(compute_heartbeat(100, None), Duration::from_secs(60));
    }

    #[test]
    fn compute_heartbeat_clamps_high() {
        // 86400 / 4 = 21600 > 3600 → clamped to 3600
        assert_eq!(compute_heartbeat(86_400, None), Duration::from_secs(3600));
    }

    #[test]
    fn compute_heartbeat_honors_override() {
        assert_eq!(compute_heartbeat(7200, Some(120)), Duration::from_secs(120));
    }

    #[test]
    fn compute_heartbeat_minimum_is_one_second() {
        // override = 0 would yield a zero-duration window; clamp to 1s so the
        // loop still terminates rather than panicking on `Duration::ZERO`.
        assert_eq!(compute_heartbeat(7200, Some(0)), Duration::from_secs(1));
    }

    #[test]
    fn build_halt_aggregate_with_on_disk_carries_evidence() {
        let on_disk = serde_json::json!({
            "status": "in_progress",
            "current_phase": "9_phase2_dispatch",
            "updated_at": "2026-05-14T04:25:00Z",
            "task_status": { "T-P0": "COMPLETED" },
            "conductors_spawned": ["pcb", "rtl", "fpga"],
            "phases_completed": ["1", "2"],
        });
        let v = build_halt_aggregate("run-1", "inst", "/tmp/x.json", 1200, Some(&on_disk));
        assert_eq!(v["status"], "error");
        assert_eq!(v["halted_reason"], "timeout");
        assert_eq!(v["aborted_reason"], "timeout");
        assert_eq!(v["synthesized_by"], "hestia-ai-cli");
        assert_eq!(v["current_phase"], "9_phase2_dispatch");
        assert_eq!(v["updated_at"], "2026-05-14T04:25:00Z");
        assert!(v["task_status"].is_object());
        assert!(v["conductors_spawned"].is_array());
        assert!(v["phases_completed"].is_array());

        let msg = v["halt_message"].as_str().unwrap();
        assert!(msg.contains("status:in_progress"), "halt_message={msg}");
        assert!(msg.contains("9_phase2_dispatch"), "halt_message={msg}");
        assert!(
            !msg.contains("max_iterations"),
            "halt_message must not blame max_iterations: {msg}"
        );
    }

    #[test]
    fn build_halt_aggregate_without_on_disk_degrades_gracefully() {
        let v = build_halt_aggregate("run-2", "inst", "/tmp/x.json", 7200, None);
        let msg = v["halt_message"].as_str().unwrap();
        assert!(msg.contains("does not exist"), "halt_message={msg}");
        assert!(
            !msg.contains("max_iterations"),
            "halt_message must not blame max_iterations: {msg}"
        );
        assert!(v.get("current_phase").is_none());
        assert!(v.get("updated_at").is_none());
        assert!(v.get("task_status").is_none());
        assert_eq!(v["status"], "error");
        assert_eq!(v["synthesized_by"], "hestia-ai-cli");
    }

    #[test]
    fn build_halt_aggregate_missing_fields_in_on_disk() {
        // Conductor wrote partial JSON missing current_phase / updated_at —
        // halt_message should still be well-formed.
        let on_disk = serde_json::json!({ "status": "in_progress" });
        let v = build_halt_aggregate("run-3", "inst", "/tmp/x.json", 1200, Some(&on_disk));
        let msg = v["halt_message"].as_str().unwrap();
        assert!(msg.contains("status:in_progress"));
        assert!(msg.contains("(none recorded)"));
        assert!(msg.contains("(no updated_at)"));
    }
}