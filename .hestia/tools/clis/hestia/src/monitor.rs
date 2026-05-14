//! Phase 108 — Hestia system health monitor + status viewer.
//!
//! - `run_monitor_daemon()`: Long-running loop spawned as a child process from
//!   `hestia start ai`. Every 30 seconds it polls the status of sub-agents
//!   (ai-designer / ai-reviewer) and running domain conductors, and issues
//!   resume instructions via `agent-cli send` only when all agents have stopped
//!   with pending tasks remaining.
//! - `run_monitor()`: Human-facing `hestia monitor` subcommand.
//!   Periodically refreshes the existing `show_status` output.
//!
//! See `.aiprj/AI_PRJ_DESIGN.md` §3 / §4 for design details.

use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::process::Command;

use super::{
    collect_agent_statuses, is_engine_peer_id, show_status, transform_status_listing, AgentStatus,
};

/// Monitor target kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MonitorKind {
    /// Resident sub-agents such as ai-designer / ai-reviewer, or dynamic sub-agents
    /// (`<domain>-coder-*` etc.).
    Subagent,
    /// Currently running domain conductors among rtl / fpga / asic / pcb / hal / apps / debug / rag.
    DomainConductor,
    /// (Phase 110) ai-conductor (peer "ai") exclusively. Excluded from Phase 109 auto-termination,
    /// but included in Phase 108 batch-resume and Phase 110 rescue targets.
    AiConductor,
}

/// Resolved result for a single monitor target.
#[derive(Debug, Clone)]
pub(crate) struct MonitorTarget {
    pub agent_id: String,
    pub peer: String,
    /// Kind (read-only, used for log formatting and future extensions).
    #[allow(dead_code)]
    pub kind: MonitorKind,
    /// Phase 109 — Parent domain conductor name for dynamic sub-agents (`<domain>-coder-*` etc.).
    /// Even for `Subagent`, distinguished as `parent_conductor = Some("ai")` (resident sub-agent) or
    /// `Some("rtl")` (dynamic sub-agent).
    /// `None` for `DomainConductor`.
    pub parent_conductor: Option<String>,
}

/// One row from `task_status.md`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TaskStatusEntry {
    pub task_id: String,
    pub state: String,
}

/// Monitored peer names. `ai` itself is excluded (no self-monitoring).
const SUBAGENT_PEERS: &[&str] = &["ai-designer", "ai-reviewer"];
const DOMAIN_CONDUCTOR_PEERS: &[&str] =
    &["rtl", "fpga", "asic", "pcb", "hal", "apps", "debug", "rag"];

/// Monitor interval in seconds. Overridden by `HESTIA_MONITOR_INTERVAL_SECS`, clamped to 1..=600.
/// The instruction requires 1 Hz cadence so the STARTING-stall detector can observe
/// at least 300 ticks before firing at the 5-minute threshold.
pub(crate) fn monitor_interval_secs() -> u64 {
    std::env::var("HESTIA_MONITOR_INTERVAL_SECS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .map(clamp_monitor_interval)
        .unwrap_or(1)
}

/// Set `HESTIA_MONITOR_DISABLED=1` to completely disable the monitoring loop.
pub(crate) fn monitor_disabled() -> bool {
    matches!(
        std::env::var("HESTIA_MONITOR_DISABLED").ok().as_deref(),
        Some("1") | Some("true") | Some("yes")
    )
}

pub(crate) fn clamp_monitor_interval(s: u64) -> u64 {
    s.clamp(1, 600)
}

/// Clamp the refresh interval in seconds for `hestia monitor`. 1..=60.
pub(crate) fn clamp_view_interval(s: u64) -> u64 {
    s.clamp(1, 60)
}

/// Default seconds before a peer stuck in STARTING is treated as a startup
/// failure. The user instruction is "STARTING for 5 minutes".
pub(crate) const STARTING_STALL_SECS_DEFAULT: u64 = 300;

/// STARTING-stall threshold in seconds. Overridden by
/// `HESTIA_MONITOR_STARTING_STALL_SECS`, clamped to 60..=1800.
pub(crate) fn starting_stall_secs() -> u64 {
    std::env::var("HESTIA_MONITOR_STARTING_STALL_SECS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .map(clamp_starting_stall)
        .unwrap_or(STARTING_STALL_SECS_DEFAULT)
}

pub(crate) fn clamp_starting_stall(s: u64) -> u64 {
    s.clamp(60, 1800)
}

pub(crate) fn starting_stall_duration() -> Duration {
    Duration::from_secs(starting_stall_secs())
}

/// Returns `.hestia/workspaces/` under the project root.
pub(crate) fn workspaces_root() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".hestia/workspaces")
}

/// Phase 109 / 110 — Pure function that classifies a peer name into a monitor kind and parent conductor.
///
/// Returns: `Some((kind, parent_conductor))` or `None` (not a monitor target).
/// - `ai` -> `(AiConductor, None)` (added as monitor target in Phase 110)
/// - `ai-designer` / `ai-reviewer` -> `(Subagent, Some("ai"))`
/// - `rtl` / `fpga` / ... -> `(DomainConductor, None)`
/// - `rtl-coder-uart` / `asic-signoff` etc. (`<domain>-*` pattern) -> `(Subagent, Some(<domain>))`
/// - unknown -> `None`
pub(crate) fn classify_peer(name: &str) -> Option<(MonitorKind, Option<String>)> {
    if name == "ai" {
        return Some((MonitorKind::AiConductor, None));
    }
    if SUBAGENT_PEERS.contains(&name) {
        return Some((MonitorKind::Subagent, Some("ai".to_string())));
    }
    if DOMAIN_CONDUCTOR_PEERS.contains(&name) {
        return Some((MonitorKind::DomainConductor, None));
    }
    for &d in DOMAIN_CONDUCTOR_PEERS {
        let prefix = format!("{d}-");
        if name.starts_with(&prefix) {
            return Some((MonitorKind::Subagent, Some(d.to_string())));
        }
    }
    None
}

/// Pure function that extracts monitor targets from `<engine> list` stdout.
///
/// Uses `classify_peer()` to determine the monitor kind; non-targets are excluded.
/// Phase 109 adds support for dynamic sub-agents (`<domain>-coder-*` etc.).
/// Phase 121: ID prefix detection uses `is_engine_peer_id` for engine abstraction
/// (accepts both `agent-` from agent-cli and `shim-` from claude_cli_shim).
pub(crate) fn resolve_monitor_targets(stdout: &str) -> Vec<MonitorTarget> {
    let mut out = Vec::new();
    for line in stdout.lines().skip(1) {
        // Column 1 = ID, Column 2 = NAME. Take the first 2 whitespace-separated tokens.
        let mut it = line.split_whitespace();
        let Some(id) = it.next() else { continue };
        let Some(name) = it.next() else { continue };
        if !is_engine_peer_id(id) {
            continue;
        }
        let Some((kind, parent_conductor)) = classify_peer(name) else {
            continue;
        };
        out.push(MonitorTarget {
            agent_id: id.to_string(),
            peer: name.to_string(),
            kind,
            parent_conductor,
        });
    }
    out
}

/// Pure function that parses the Markdown table in `task_status.md` and returns
/// entries for pending-task detection.
///
/// Expected format (`AI_PRJ_DESIGN.md` §4.2):
/// ```text
/// | Task ID  | Status     | Updated By | Updated At  | Notes |
/// |----------|------------|------------|-------------|-------|
/// | T-001    | Complete   | foo        | ...         | -     |
/// ```
/// Header and separator rows are skipped. Only rows whose status column is one of
/// "Not started" / "In progress" / "Complete" / "Blocked" are included as entries.
pub(crate) fn parse_task_status(content: &str) -> Vec<TaskStatusEntry> {
    let mut out = Vec::new();
    for raw in content.lines() {
        let line = raw.trim();
        if !line.starts_with('|') {
            continue;
        }
        // Skip separator rows (|----|----|).
        if line.chars().all(|c| matches!(c, '|' | '-' | ':' | ' ')) {
            continue;
        }
        let cells: Vec<&str> = line
            .trim_matches('|')
            .split('|')
            .map(|s| s.trim())
            .collect();
        if cells.len() < 2 {
            continue;
        }
        let task_id = cells[0].to_string();
        let state = cells[1].to_string();
        // Skip header rows (containing "Status" or "Task ID" column labels).
        if state == "Status" || task_id == "Task ID" {
            continue;
        }
        // Skip rows whose state value is not in the expected set (excludes arbitrary note rows).
        if !matches!(state.as_str(), "Not started" | "In progress" | "Complete" | "Blocked") {
            continue;
        }
        out.push(TaskStatusEntry { task_id, state });
    }
    out
}

/// Phase 109 — Pure function that returns `true` when **all** tasks on
/// `<workspace>/<peer>/task_status.md` are "Complete". Zero entries or missing file returns `false`
/// (prevents false termination).
pub(crate) fn peer_tasks_all_complete(workspace_root: &Path, peer: &str) -> bool {
    let path = workspace_root.join(peer).join("task_status.md");
    let Ok(content) = std::fs::read_to_string(&path) else {
        return false;
    };
    let entries = parse_task_status(&content);
    !entries.is_empty() && entries.iter().all(|e| e.state == "Complete")
}

/// Phase 109 — Pure function that returns the peer names of domain conductors satisfying:
/// 1. All rows in `task_status.md` are "Complete"
/// 2. No sub-agents (`<domain>-*` peers) under that conductor are included in `targets`
///    (i.e., they have already disappeared from `agent-cli list`)
///
/// Ordering guarantee: A conductor is not returned while its sub-agents are still present.
pub(crate) fn conductors_ready_to_terminate(
    targets: &[MonitorTarget],
    workspace_root: &Path,
) -> Vec<String> {
    let mut out = Vec::new();
    for c in targets.iter() {
        if !matches!(c.kind, MonitorKind::DomainConductor) {
            continue;
        }
        if !peer_tasks_all_complete(workspace_root, &c.peer) {
            continue;
        }
        let has_subagent = targets.iter().any(|t| {
            matches!(t.kind, MonitorKind::Subagent)
                && t.parent_conductor.as_deref() == Some(c.peer.as_str())
        });
        if has_subagent {
            continue;
        }
        out.push(c.peer.clone());
    }
    out
}

/// Phase 109 — Pure function that checks whether a specific status from `agent-cli list`
/// (IDLE / Error / Unknown) is safe for termination. Used by the monitor loop to
/// determine "is this status safe to terminate?"
pub(crate) fn is_terminable_status(status: AgentStatus) -> bool {
    matches!(
        status,
        AgentStatus::Idle | AgentStatus::Error | AgentStatus::Unknown
    )
}

/// Phase 109 — Gracefully terminate `peer`.
///
/// 1. Extract associated PIDs via `pgrep -f "agent-cli run.*--name <peer>"`
/// 2. Send SIGTERM to each PID
/// 3. After a grace period (`HESTIA_MONITOR_TERMINATE_GRACE_SECS`, default 10s, clamped 0..=60),
///    escalate to SIGKILL for any still-alive PIDs
///
/// Even with duplicate peer states (pre-Phase 109 fix), all PIDs are captured and terminated.
async fn terminate_peer(peer: &str) -> anyhow::Result<()> {
    let pids = pgrep_agent_cli_pids(peer).await;
    if pids.is_empty() {
        return Ok(());
    }
    eprintln!(
        "[monitor] terminating peer '{peer}' (PIDs: {})",
        pids.iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );
    for &pid in &pids {
        unsafe {
            libc::kill(pid as i32, libc::SIGTERM);
        }
    }

    let grace = Duration::from_secs(terminate_grace_secs());
    tokio::time::sleep(grace).await;

    // Escalate to SIGKILL for PIDs that are still alive
    let still_alive = pgrep_agent_cli_pids(peer).await;
    for &pid in &still_alive {
        if pids.contains(&pid) {
            eprintln!("[monitor] SIGKILL escalation for '{peer}' PID {pid}");
            unsafe {
                libc::kill(pid as i32, libc::SIGKILL);
            }
        }
    }
    Ok(())
}

async fn pgrep_agent_cli_pids(peer: &str) -> Vec<u32> {
    let cfg = super::load_hestia_config();
    let bin_basename = cfg.engine.binary_basename();
    let pattern = format!("{bin_basename} run.*--name {peer}");
    let Ok(out) = Command::new("pgrep")
        .arg("-f")
        .arg(&pattern)
        .output()
        .await
    else {
        return Vec::new();
    };
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|l| l.trim().parse::<u32>().ok())
        .collect()
}

/// Phase 109 — Termination grace period in seconds (default 10, overridden by `HESTIA_MONITOR_TERMINATE_GRACE_SECS`, clamped 0..=60).
pub(crate) fn terminate_grace_secs() -> u64 {
    std::env::var("HESTIA_MONITOR_TERMINATE_GRACE_SECS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .map(clamp_terminate_grace)
        .unwrap_or(10)
}

pub(crate) fn clamp_terminate_grace(s: u64) -> u64 {
    s.clamp(0, 60)
}

// ─────────────────────────────────────────────────────────────────────
// Phase 110 — SIGKILL + re-spawn rescue path (now invoked only by the
// Phase 136 STARTING-stall detector; the Phase 108 batch-resume and
// idle-rescue paths were removed in the 2026-05-14 process-termination
// fix). See AI_PRJ_DESIGN.md §1.1 for the reasoning.
// ─────────────────────────────────────────────────────────────────────

/// Resolve the persona file name (without extension) from a peer name.
///
/// - `asic-signoff` -> `asic-signoff-checker` (known exception, HD-033)
/// - `<domain>-coder-<module>` -> `<domain>-coder`
/// - Others (`ai` / `ai-designer` / `rtl` / etc.) -> peer name as-is
pub(crate) fn resolve_persona_for_peer(peer: &str) -> Option<String> {
    if peer.is_empty() {
        return None;
    }
    if peer == "asic-signoff" {
        return Some("asic-signoff-checker".to_string());
    }
    for &d in DOMAIN_CONDUCTOR_PEERS {
        let prefix = format!("{d}-coder-");
        if peer.starts_with(&prefix) {
            return Some(format!("{d}-coder"));
        }
    }
    Some(peer.to_string())
}

/// Phase 110 — Pure function that generates the instruction message sent to a
/// re-spawned peer after rescue.
pub(crate) fn build_rescue_message(peer: &str) -> String {
    format!(
        "[hestia monitor / Phase 110 rescue] You ({p}) did not respond to previous \
         resume instructions, so the process was killed via SIGKILL and restarted. \
         Read `.hestia/rules/update_project.md` from the project root via fs_read, \
         and follow its AI Update Guidelines and Update Details to resume work. \
         Also read `<workspace>/{p}/tasks.md` and `<workspace>/{p}/task_status.md` \
         via fs_read, and continue from pending tasks (Not started / In progress / Blocked).",
        p = peer
    )
}

/// Async helper that sends an immediate SIGKILL to the peer's agent-cli process.
/// Unlike `terminate_peer` (SIGTERM -> grace -> SIGKILL), this only sends SIGKILL immediately.
async fn kill_peer_now(peer: &str) -> Vec<u32> {
    let pids = pgrep_agent_cli_pids(peer).await;
    for &pid in &pids {
        unsafe {
            libc::kill(pid as i32, libc::SIGKILL);
        }
    }
    pids
}

/// Phase 110 — After kill, polls until the peer disappears from `agent-cli list`.
/// On timeout, still proceeds to the next step (re-spawn) with a warning only.
async fn wait_for_deregistration(peer: &str, max_wait: Duration) -> bool {
    let start = Instant::now();
    while start.elapsed() < max_wait {
        let stdout = run_agent_cli_list().await.unwrap_or_default();
        if !crate::registered_peer_names(&stdout).contains(peer) {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    false
}

/// Phase 110 — Rescue path main body.
///
/// 1. Immediately SIGKILL the peer's agent-cli process
/// 2. Poll until the peer disappears from `agent-cli list` (max 10 seconds)
/// 3. Resolve the persona file name from the peer name and verify it exists
/// 4. Re-spawn via `spawn_agent_cli` (passes duplicate check)
/// 5. Wait for registry registration (max 15 seconds)
/// 6. Send `update_project.md` read + pending task resume instruction via `agent-cli send`
pub(crate) async fn rescue_peer(peer: &str) -> anyhow::Result<()> {
    let killed = kill_peer_now(peer).await;
    eprintln!(
        "[monitor/rescue] killed '{peer}' (PIDs: {})",
        killed
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );

    let deregistered = wait_for_deregistration(peer, Duration::from_secs(10)).await;
    if !deregistered {
        eprintln!("[monitor/rescue] '{peer}' did not deregister within 10s — proceeding anyway");
    }

    let Some(persona_root) = resolve_persona_for_peer(peer) else {
        anyhow::bail!("could not resolve persona for peer '{peer}'");
    };
    let persona_path = std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".hestia/personas")
        .join(format!("{persona_root}.md"));
    if !persona_path.exists() {
        anyhow::bail!(
            "persona file '{}' not found for peer '{peer}'",
            persona_path.display()
        );
    }

    crate::spawn_agent_cli(&persona_root, peer).await?;
    let _ = conductor_sdk::workspace::wait_for_registry(peer, 15_000);

    let msg = build_rescue_message(peer);
    let cfg = super::load_hestia_config();
    let engine_bin = cfg.engine.binary_name();
    let status = Command::new(engine_bin)
        .arg("send")
        .arg(peer)
        .arg(&msg)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await
        .map_err(|e| anyhow::anyhow!("{engine_bin} send {peer} (rescue) failed: {e}"))?;
    if !status.success() {
        anyhow::bail!("{engine_bin} send {peer} (rescue message) exited with {status}");
    }
    eprintln!("[monitor/rescue] '{peer}' rescued successfully");
    Ok(())
}

/// Pure decision helper: should we trigger a STARTING-stall rescue for a peer
/// whose first STARTING observation happened at `first_seen`?
/// Returns `true` when `now - first_seen >= threshold`.
pub(crate) fn should_starting_stall_rescue(
    first_seen: Instant,
    now: Instant,
    threshold: Duration,
) -> bool {
    now.saturating_duration_since(first_seen) >= threshold
}

/// Pure message builder: the text the monitor sends to the parent conductor
/// after killing + re-spawning an agent that was stuck in STARTING.
pub(crate) fn build_starting_failure_message(peer: &str, threshold_secs: u64) -> String {
    format!(
        "[hestia monitor] Agent '{peer}' was killed because it stayed in \
         STARTING for more than {secs}s without producing any activity log. \
         The agent has been re-spawned. Please re-issue your last instruction \
         to '{peer}' so it can resume from a fresh state.",
        secs = threshold_secs,
    )
}

/// Send the STARTING-failure notification to the parent conductor via
/// `<engine> send <parent_conductor>`. Errors are returned (not silently swallowed)
/// so the caller can log them.
async fn notify_conductor_of_starting_failure(
    peer: &str,
    parent: &str,
    threshold_secs: u64,
) -> anyhow::Result<()> {
    let msg = build_starting_failure_message(peer, threshold_secs);
    let cfg = super::load_hestia_config();
    let engine_bin = cfg.engine.binary_name();
    let status = Command::new(engine_bin)
        .arg("send")
        .arg(parent)
        .arg(&msg)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await
        .map_err(|e| anyhow::anyhow!("{engine_bin} send {parent}: {e}"))?;
    if !status.success() {
        anyhow::bail!("{engine_bin} send {parent} exited with {status}");
    }
    Ok(())
}

/// STARTING-stall rescue: SIGKILL + re-spawn the agent (via the existing Phase 110
/// rescue path), then notify the parent conductor so it can re-issue the request.
pub(crate) async fn starting_stall_rescue(
    peer: &str,
    parent_conductor: Option<&str>,
    threshold_secs: u64,
) -> anyhow::Result<()> {
    rescue_peer(peer).await?;
    if let Some(parent) = parent_conductor {
        notify_conductor_of_starting_failure(peer, parent, threshold_secs).await?;
    } else {
        eprintln!(
            "[monitor/starting-stall] '{peer}' has no parent conductor — \
             skipping notification (peer is 'ai' or unknown)"
        );
    }
    Ok(())
}

/// Status summary for `hestia monitor` header display.
///
/// Phase 110: Added `think` field.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct StatusSummary {
    pub running: usize,
    pub busy: usize,
    pub think: usize,
    pub idle: usize,
    pub waiting: usize,
    pub error: usize,
    pub starting: usize,
    pub unknown: usize,
}

pub(crate) fn summarize_statuses(statuses: &HashMap<String, AgentStatus>) -> StatusSummary {
    let mut s = StatusSummary::default();
    s.running = statuses.len();
    for v in statuses.values() {
        match v {
            AgentStatus::Busy => s.busy += 1,
            AgentStatus::Think => s.think += 1,
            AgentStatus::Idle => s.idle += 1,
            AgentStatus::Waiting => s.waiting += 1,
            AgentStatus::Error => s.error += 1,
            AgentStatus::Starting => s.starting += 1,
            AgentStatus::Unknown => s.unknown += 1,
        }
    }
    s
}

pub(crate) fn build_monitor_header(
    summary: &StatusSummary,
    interval: u64,
    timestamp: &str,
) -> String {
    format!(
        "[Hestia Monitor]  refreshed: {ts}  (every {iv}s, Ctrl+C to exit)\n\
         Conductors: {run} running   BUSY: {b}   THINKING: {th}   IDLE: {i}   WAIT: {w}   ERROR: {e}   STARTING: {st}   UNKNOWN: {u}\n\n",
        ts = timestamp,
        iv = interval,
        run = summary.running,
        b = summary.busy,
        th = summary.think,
        i = summary.idle,
        w = summary.waiting,
        e = summary.error,
        st = summary.starting,
        u = summary.unknown,
    )
}

async fn run_agent_cli_list() -> Result<String> {
    let cfg = super::load_hestia_config();
    let engine_bin = cfg.engine.binary_name();
    let out = Command::new(engine_bin)
        .arg("list")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .await
        .map_err(|e| anyhow::anyhow!("failed to run {engine_bin} list: {e}"))?;
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

fn install_signal_handler() -> Arc<AtomicBool> {
    let stop = Arc::new(AtomicBool::new(false));
    let s_int = stop.clone();
    let s_term = stop.clone();
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            s_int.store(true, Ordering::SeqCst);
        }
    });
    tokio::spawn(async move {
        use tokio::signal::unix::{signal, SignalKind};
        if let Ok(mut sig) = signal(SignalKind::terminate()) {
            sig.recv().await;
            s_term.store(true, Ordering::SeqCst);
        }
    });
    stop
}

/// SIGKILL every `agent-cli run …` (or shim equivalent) child process the
/// daemon was monitoring. Called by `run_monitor_daemon` after a SIGINT/SIGTERM
/// breaks the main loop. We intentionally leave `hestia mirror` helpers and the
/// daemon's own PID alone — mirrors die naturally with their parent agent-cli,
/// and the daemon exits by itself once this function returns.
async fn shutdown_all_agents() {
    eprintln!("[monitor] shutdown requested — SIGKILLing agent-cli children");
    let cfg = super::load_hestia_config();
    let patterns = super::engine_kill_patterns(&cfg);
    let self_pid = std::process::id();
    let mut killed: usize = 0;
    for pat in patterns
        .iter()
        .filter(|p| !p.contains("monitor-daemon") && !p.contains("hestia mirror"))
    {
        let Ok(out) = Command::new("pgrep")
            .arg("-f")
            .arg(pat)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .await
        else {
            continue;
        };
        for line in String::from_utf8_lossy(&out.stdout).lines() {
            let Ok(pid) = line.trim().parse::<u32>() else { continue };
            if pid == self_pid {
                continue;
            }
            unsafe {
                libc::kill(pid as i32, libc::SIGKILL);
            }
            killed += 1;
        }
    }
    eprintln!("[monitor] shutdown_all_agents complete — SIGKILLed {killed} process(es)");
}

fn now_iso8601() -> String {
    use std::time::UNIX_EPOCH;
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // Simplified ISO 8601 (chrono is a workspace dep for Hestia overall,
    // but not yet added to the hestia crate. Using format! for UTC display at second precision).
    let days = (secs / 86_400) as i64;
    let h = ((secs % 86_400) / 3_600) as u32;
    let m = ((secs % 3_600) / 60) as u32;
    let s = (secs % 60) as u32;
    let (y, mo, d) = ymd_from_days_since_epoch(days);
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{m:02}:{s:02}Z")
}

/// Simple implementation that converts days since 1970-01-01 to YYYY-MM-DD.
/// Only considers 4/100/400 leap year rules. UTC timezone only (for second-precision display).
fn ymd_from_days_since_epoch(mut days: i64) -> (i32, u32, u32) {
    let mut year: i32 = 1970;
    loop {
        let yd = if is_leap(year) { 366 } else { 365 };
        if days < yd {
            break;
        }
        days -= yd;
        year += 1;
    }
    let mlen = if is_leap(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut month: u32 = 1;
    for &len in &mlen {
        if days < len as i64 {
            break;
        }
        days -= len as i64;
        month += 1;
    }
    (year, month, (days + 1) as u32)
}

fn is_leap(y: i32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0)
}

/// `hestia monitor-daemon` main body (launched from `Commands::MonitorDaemon`).
pub(crate) async fn run_monitor_daemon() -> Result<()> {
    if monitor_disabled() {
        eprintln!("[monitor] HESTIA_MONITOR_DISABLED=1 — daemon exiting without monitoring");
        return Ok(());
    }

    let interval = Duration::from_secs(monitor_interval_secs());
    let stop = install_signal_handler();
    // STARTING-stall detector — per-peer first-observation timestamp.
    let mut starting_observed_at: HashMap<String, Instant> = HashMap::new();
    let starting_stall = starting_stall_duration();
    let starting_stall_secs_val = starting_stall.as_secs();

    eprintln!(
        "[monitor] daemon started (interval={}s, starting_stall={}s)",
        interval.as_secs(),
        starting_stall_secs_val,
    );

    while !stop.load(Ordering::SeqCst) {
        tokio::select! {
            _ = tokio::time::sleep(interval) => {}
            _ = wait_for_stop(stop.clone()) => break,
        }
        if stop.load(Ordering::SeqCst) {
            break;
        }

        let stdout = match run_agent_cli_list().await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[monitor] agent-cli list failed: {e} — retrying next tick");
                continue;
            }
        };

        let statuses = collect_agent_statuses(&stdout, SystemTime::now());
        let targets = resolve_monitor_targets(&stdout);
        let workspace_root = workspaces_root();

        // ── STARTING-stall detector ─────────────────────────────────────
        // Update the per-peer first-STARTING-observation map, then fire
        // `starting_stall_rescue` for any peer past the threshold.
        let now_observed = Instant::now();
        for t in &targets {
            let status = statuses.get(&t.agent_id).copied().unwrap_or(AgentStatus::Unknown);
            if matches!(status, AgentStatus::Starting) {
                starting_observed_at.entry(t.peer.clone()).or_insert(now_observed);
            } else {
                starting_observed_at.remove(&t.peer);
            }
        }
        // Drop entries for peers that have disappeared from `agent-cli list`.
        let registered_peers = crate::registered_peer_names(&stdout);
        starting_observed_at.retain(|peer, _| registered_peers.contains(peer));

        let to_starting_rescue: Vec<(String, Option<String>)> = starting_observed_at
            .iter()
            .filter(|(_, first_seen)| {
                should_starting_stall_rescue(**first_seen, now_observed, starting_stall)
            })
            .filter_map(|(peer, _)| {
                let parent = targets
                    .iter()
                    .find(|t| t.peer == *peer)
                    .and_then(|t| t.parent_conductor.clone());
                Some((peer.clone(), parent))
            })
            .collect();

        for (peer, parent) in to_starting_rescue {
            eprintln!(
                "[monitor/starting-stall] '{peer}' has been STARTING for ≥{starting_stall_secs_val}s — \
                 SIGKILL + re-spawn + notify parent={:?}",
                parent
            );
            if let Err(e) =
                starting_stall_rescue(&peer, parent.as_deref(), starting_stall_secs_val).await
            {
                eprintln!("[monitor/starting-stall] '{peer}' rescue failed: {e}");
            }
            starting_observed_at.remove(&peer);
        }

        // ── Phase 109 (1) Gracefully terminate completed sub-agents ──
        for t in &targets {
            if !matches!(t.kind, MonitorKind::Subagent) {
                continue;
            }
            let Some(status) = statuses.get(&t.agent_id).copied() else {
                continue;
            };
            if !is_terminable_status(status) {
                continue;
            }
            if !peer_tasks_all_complete(&workspace_root, &t.peer) {
                continue;
            }
            if let Err(e) = terminate_peer(&t.peer).await {
                eprintln!("[monitor] terminate '{}' failed: {e}", t.peer);
            }
        }

        // ── Phase 109 (2) Terminate completed conductors with no remaining sub-agents ──
        // Note: sub-agents removed in step (1) will disappear from the next cycle's `agent-cli list`.
        // They are not simultaneously removed in the same cycle, so `targets` (= previous snapshot)
        // is used as the argument, and `conductors_ready_to_terminate` errs on the safe side
        // (subagent still present -> don't terminate conductor).
        for peer in conductors_ready_to_terminate(&targets, &workspace_root) {
            if let Err(e) = terminate_peer(&peer).await {
                eprintln!("[monitor] terminate conductor '{peer}' failed: {e}");
            }
        }

        // Phase 108 batch-resume and Phase 110 SIGKILL-on-idle rescue paths
        // were removed in the 2026-05-14 process-termination fix. The only
        // automatic actions on the loop body are now: STARTING-stall (above)
        // and Phase 109 graceful terminate-on-completed (above). Idle peers
        // are intentionally left alone — the user drives resumption via
        // `hestia ai run` / `hestia ai qa`, or aborts via `hestia stop` /
        // `hestia kill`. See report_stop.md §9 for context.
    }

    eprintln!("[monitor] daemon stopping");
    shutdown_all_agents().await;
    Ok(())
}

async fn wait_for_stop(stop: Arc<AtomicBool>) {
    loop {
        if stop.load(Ordering::SeqCst) {
            return;
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
}

/// `hestia monitor` main body (human-facing, periodic refresh display).
pub(crate) async fn run_monitor(interval: u64, once: bool, all: bool) -> Result<()> {
    if once {
        return show_status(all).await;
    }

    let interval = clamp_view_interval(interval);
    let stop = install_signal_handler();
    let is_tty = atty_stdout();

    while !stop.load(Ordering::SeqCst) {
        match build_monitor_frame(all, interval).await {
            Ok(frame) => {
                if is_tty {
                    // ANSI: cursor home + clear screen + clear scrollback.
                    print!("\x1b[H\x1b[2J\x1b[3J{frame}");
                } else {
                    print!("{frame}\n");
                }
                use std::io::Write;
                let _ = std::io::stdout().flush();
            }
            Err(e) => {
                eprintln!("[monitor] frame build failed: {e}");
            }
        }
        tokio::select! {
            _ = tokio::time::sleep(Duration::from_secs(interval)) => {}
            _ = wait_for_stop(stop.clone()) => break,
        }
    }

    if is_tty {
        // Move cursor to bottom-left on exit (avoid collision with next prompt).
        println!();
    }
    Ok(())
}

async fn build_monitor_frame(all: bool, interval: u64) -> Result<String> {
    let stdout = run_agent_cli_list().await?;
    let now = SystemTime::now();
    let statuses = collect_agent_statuses(&stdout, now);
    let summary = summarize_statuses(&statuses);
    let header = build_monitor_header(&summary, interval, &now_iso8601());
    let body = transform_status_listing(&stdout, all, &statuses);
    Ok(format!("{header}{body}"))
}

fn atty_stdout() -> bool {
    // Use libc::isatty(1) to determine if stdout is a terminal. On failure, treat as false (piped output).
    unsafe { libc::isatty(1) == 1 }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target_with_parent(
        id: &str,
        peer: &str,
        kind: MonitorKind,
        parent: Option<&str>,
    ) -> MonitorTarget {
        MonitorTarget {
            agent_id: id.to_string(),
            peer: peer.to_string(),
            kind,
            parent_conductor: parent.map(|s| s.to_string()),
        }
    }

    fn statuses(pairs: &[(&str, AgentStatus)]) -> HashMap<String, AgentStatus> {
        pairs.iter().map(|(k, v)| ((*k).to_string(), *v)).collect()
    }

    // ─── clamp helpers ────────────────────────────────────────────────────
    #[test]
    fn clamp_monitor_interval_bounds() {
        // Lower bound dropped from 5 → 1 (instruction requires 1 Hz cadence
        // so the STARTING-stall detector ticks at least 300 times before firing
        // at the 5-minute threshold).
        assert_eq!(clamp_monitor_interval(0), 1);
        assert_eq!(clamp_monitor_interval(1), 1);
        assert_eq!(clamp_monitor_interval(5), 5);
        assert_eq!(clamp_monitor_interval(30), 30);
        assert_eq!(clamp_monitor_interval(600), 600);
        assert_eq!(clamp_monitor_interval(601), 600);
        assert_eq!(clamp_monitor_interval(1_000_000), 600);
    }

    #[test]
    fn clamp_view_interval_bounds() {
        assert_eq!(clamp_view_interval(0), 1);
        assert_eq!(clamp_view_interval(1), 1);
        assert_eq!(clamp_view_interval(2), 2);
        assert_eq!(clamp_view_interval(60), 60);
        assert_eq!(clamp_view_interval(61), 60);
        assert_eq!(clamp_view_interval(10_000), 60);
    }

    // ─── STARTING-stall detector ──────────────────────────────────────────

    #[test]
    fn clamp_starting_stall_bounds() {
        assert_eq!(clamp_starting_stall(0), 60);
        assert_eq!(clamp_starting_stall(59), 60);
        assert_eq!(clamp_starting_stall(60), 60);
        assert_eq!(clamp_starting_stall(300), 300);
        assert_eq!(clamp_starting_stall(1800), 1800);
        assert_eq!(clamp_starting_stall(1801), 1800);
        assert_eq!(clamp_starting_stall(86_400), 1800);
    }

    #[test]
    fn should_starting_stall_rescue_at_or_past_threshold() {
        let first_seen = Instant::now();
        let threshold = Duration::from_secs(300);

        // Just before threshold → false.
        let now_before = first_seen + Duration::from_secs(299);
        assert!(!should_starting_stall_rescue(first_seen, now_before, threshold));

        // Exactly at threshold → true (>=).
        let now_at = first_seen + Duration::from_secs(300);
        assert!(should_starting_stall_rescue(first_seen, now_at, threshold));

        // Past threshold → true.
        let now_past = first_seen + Duration::from_secs(450);
        assert!(should_starting_stall_rescue(first_seen, now_past, threshold));
    }

    #[test]
    fn should_starting_stall_rescue_saturating() {
        // Argument order inversion (now < first_seen) → 0 elapsed → false.
        let first_seen = Instant::now() + Duration::from_secs(60);
        let now = Instant::now();
        let threshold = Duration::from_secs(10);
        assert!(!should_starting_stall_rescue(first_seen, now, threshold));
    }

    #[test]
    fn build_starting_failure_message_contains_required_substrings() {
        let m = build_starting_failure_message("rtl-coder-uart", 300);
        assert!(m.contains("rtl-coder-uart"), "missing peer name: {m}");
        assert!(m.contains("STARTING"), "missing STARTING token: {m}");
        assert!(m.contains("killed"), "missing 'killed': {m}");
        assert!(m.contains("re-spawned"), "missing 'restart' phrase: {m}");
        assert!(m.contains("300s"), "missing threshold: {m}");
    }

    // ─── resolve_monitor_targets ─────────────────────────────────────────
    const HEADER_LIST: &str =
        "ID                                NAME          PROVIDER  MODEL          ROLE\n";

    #[test]
    fn resolve_picks_subagents_and_domain_and_ai_phase110() {
        // Phase 110: ai is included as MonitorKind::AiConductor.
        let body = "agent-AAA  ai          ollama  glm  meta\n\
                    agent-BBB  ai-designer ollama  glm  designer\n\
                    agent-CCC  ai-reviewer ollama  glm  reviewer\n\
                    agent-DDD  rtl         ollama  glm  conductor\n\
                    agent-EEE  bogus       ollama  glm  ???\n";
        let input = format!("{HEADER_LIST}{body}");
        let got = resolve_monitor_targets(&input);
        let names: Vec<&str> = got.iter().map(|t| t.peer.as_str()).collect();
        assert_eq!(names, vec!["ai", "ai-designer", "ai-reviewer", "rtl"]);
        assert_eq!(got[0].kind, MonitorKind::AiConductor);
        assert_eq!(got[1].kind, MonitorKind::Subagent);
        assert_eq!(got[3].kind, MonitorKind::DomainConductor);
    }

    #[test]
    fn resolve_ignores_non_agent_rows() {
        let body = "0001       ai          ollama  glm  meta\n\
                    agent-BBB  ai-designer ollama  glm  designer\n";
        let input = format!("{HEADER_LIST}{body}");
        let got = resolve_monitor_targets(&input);
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].peer, "ai-designer");
    }

    #[test]
    fn resolve_handles_empty_and_header_only() {
        assert!(resolve_monitor_targets("").is_empty());
        assert!(resolve_monitor_targets(HEADER_LIST).is_empty());
    }

    #[test]
    fn resolve_picks_claude_cli_shim_peers_phase121() {
        // Phase 121: Also recognizes `shim-<UUID>` ID prefix from claude_cli_shim engine.
        let body = "shim-aaaa-1111  ai           claude  claude-opus-4-7  meta\n\
                    shim-bbbb-2222  ai-designer  claude  claude-opus-4-7  designer\n\
                    shim-cccc-3333  ai-reviewer  claude  claude-opus-4-7  reviewer\n\
                    shim-dddd-4444  rtl          claude  claude-opus-4-7  conductor\n";
        let input = format!("{HEADER_LIST}{body}");
        let got = resolve_monitor_targets(&input);
        let names: Vec<&str> = got.iter().map(|t| t.peer.as_str()).collect();
        assert_eq!(names, vec!["ai", "ai-designer", "ai-reviewer", "rtl"]);
        assert_eq!(got[0].kind, MonitorKind::AiConductor);
        assert_eq!(got[1].kind, MonitorKind::Subagent);
        assert_eq!(got[3].kind, MonitorKind::DomainConductor);
    }

    #[test]
    fn resolve_handles_mixed_engine_prefixes_phase121() {
        // During the transition period, both agent-cli and claude-cli-shim prefixes are accepted.
        let body = "agent-AAA       ai           ollama  glm              meta\n\
                    shim-bbbb-2222  ai-designer  claude  claude-opus-4-7  designer\n";
        let input = format!("{HEADER_LIST}{body}");
        let got = resolve_monitor_targets(&input);
        let names: Vec<&str> = got.iter().map(|t| t.peer.as_str()).collect();
        assert_eq!(names, vec!["ai", "ai-designer"]);
    }

    // ─── parse_task_status ───────────────────────────────────────────────
    #[test]
    fn parse_task_status_header_only_returns_empty() {
        let md = "# header\n\n| Task ID | Status | Updated By | Updated At | Notes |\n|----|----|----|----|----|\n";
        assert!(parse_task_status(md).is_empty());
    }

    #[test]
    fn parse_task_status_picks_pending_rows() {
        let md = "\
| Task ID | Status | Updated By | Updated At | Notes |
|----|----|----|----|----|
| T-001 | Complete | foo | t1 | - |
| T-002 | In progress | foo | t2 | - |
| T-003 | Not started | foo | t3 | - |
| T-004 | Blocked | foo | t4 | - |
";
        let entries = parse_task_status(md);
        assert_eq!(entries.len(), 4);
        assert_eq!(entries[0].state, "Complete");
        assert_eq!(entries[1].state, "In progress");
        assert_eq!(entries[2].state, "Not started");
        assert_eq!(entries[3].state, "Blocked");
    }

    #[test]
    fn parse_task_status_all_complete_no_pending() {
        let md = "\
| Task ID | Status | Updated By | Updated At | Notes |
|----|----|----|----|----|
| T-001 | Complete | foo | t1 | - |
| T-002 | Complete | foo | t2 | - |
";
        let entries = parse_task_status(md);
        assert_eq!(entries.len(), 2);
        assert!(entries.iter().all(|e| e.state == "Complete"));
    }

    #[test]
    fn parse_task_status_ignores_garbage() {
        let md = "Some text\nRandom line\n";
        assert!(parse_task_status(md).is_empty());
    }

    #[test]
    fn parse_task_status_ignores_unknown_state_values() {
        let md = "\
| Task ID | Status | Updated By |
|----|----|----|
| T-001 | XXX | foo |
| T-002 | In progress | foo |
";
        let entries = parse_task_status(md);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].task_id, "T-002");
    }

    // ─── summarize_statuses & build_monitor_header ───────────────────────
    #[test]
    fn summarize_counts_all_kinds() {
        // Phase 110: Extended to include Think, for a total of 7 variants.
        let s = statuses(&[
            ("agent-A", AgentStatus::Busy),
            ("agent-B", AgentStatus::Idle),
            ("agent-C", AgentStatus::Idle),
            ("agent-D", AgentStatus::Waiting),
            ("agent-E", AgentStatus::Error),
            ("agent-F", AgentStatus::Starting),
            ("agent-G", AgentStatus::Unknown),
            ("agent-H", AgentStatus::Think),
        ]);
        let sum = summarize_statuses(&s);
        assert_eq!(sum.running, 8);
        assert_eq!(sum.busy, 1);
        assert_eq!(sum.think, 1);
        assert_eq!(sum.idle, 2);
        assert_eq!(sum.waiting, 1);
        assert_eq!(sum.error, 1);
        assert_eq!(sum.starting, 1);
        assert_eq!(sum.unknown, 1);
    }

    #[test]
    fn summarize_empty_input() {
        let s: HashMap<String, AgentStatus> = HashMap::new();
        let sum = summarize_statuses(&s);
        assert_eq!(sum.running, 0);
        assert_eq!(sum.busy, 0);
    }

    #[test]
    fn header_contains_all_columns() {
        let summary = StatusSummary {
            running: 3,
            busy: 1,
            think: 0,
            idle: 1,
            waiting: 0,
            error: 0,
            starting: 1,
            unknown: 0,
        };
        let h = build_monitor_header(&summary, 2, "2026-05-08T12:00:00Z");
        assert!(h.contains("Hestia Monitor"));
        assert!(h.contains("refreshed: 2026-05-08T12:00:00Z"));
        assert!(h.contains("every 2s"));
        assert!(h.contains("3 running"));
        assert!(h.contains("BUSY: 1"));
        assert!(h.contains("THINKING: 0"));
        assert!(h.contains("IDLE: 1"));
        assert!(h.contains("STARTING: 1"));
    }

    // ─── ymd_from_days_since_epoch ───────────────────────────────────────
    #[test]
    fn ymd_epoch_origin() {
        assert_eq!(ymd_from_days_since_epoch(0), (1970, 1, 1));
    }

    #[test]
    fn ymd_known_date() {
        // 2026-01-01 is 20454 days after epoch.
        // Leap years from 1970..=2025: years divisible by 4 from 1972..=2024 (no 100/400 exceptions) = 14.
        // 365 * 56 + 14 = 20440 + 14 = 20454.
        assert_eq!(ymd_from_days_since_epoch(20454), (2026, 1, 1));
    }

    // ─── classify_peer (Phase 109) ─────────────────────────────────────
    #[test]
    fn classify_peer_resident_subagents() {
        let (k, p) = classify_peer("ai-designer").unwrap();
        assert_eq!(k, MonitorKind::Subagent);
        assert_eq!(p, Some("ai".to_string()));
        let (k, p) = classify_peer("ai-reviewer").unwrap();
        assert_eq!(k, MonitorKind::Subagent);
        assert_eq!(p, Some("ai".to_string()));
    }

    #[test]
    fn classify_peer_domain_conductors() {
        let (k, p) = classify_peer("rtl").unwrap();
        assert_eq!(k, MonitorKind::DomainConductor);
        assert_eq!(p, None);
        let (k, p) = classify_peer("asic").unwrap();
        assert_eq!(k, MonitorKind::DomainConductor);
        assert_eq!(p, None);
    }

    #[test]
    fn classify_peer_dynamic_subagents() {
        let (k, p) = classify_peer("rtl-coder-uart").unwrap();
        assert_eq!(k, MonitorKind::Subagent);
        assert_eq!(p, Some("rtl".to_string()));
        let (k, p) = classify_peer("asic-signoff").unwrap();
        assert_eq!(k, MonitorKind::Subagent);
        assert_eq!(p, Some("asic".to_string()));
        let (k, p) = classify_peer("hal-designer").unwrap();
        assert_eq!(k, MonitorKind::Subagent);
        assert_eq!(p, Some("hal".to_string()));
    }

    #[test]
    fn classify_peer_includes_ai_as_aiconductor_phase110() {
        // Phase 110: Include ai as a monitor target (in Phase 109, this returned None).
        let (k, p) = classify_peer("ai").unwrap();
        assert_eq!(k, MonitorKind::AiConductor);
        assert_eq!(p, None);
    }

    #[test]
    fn classify_peer_unknown_returns_none() {
        assert!(classify_peer("bogus").is_none());
        assert!(classify_peer("nonsense-xyz").is_none());
    }

    // ─── peer_tasks_all_complete (Phase 109) ────────────────────────────
    #[test]
    fn peer_tasks_all_complete_returns_true_when_all_complete() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("ai-designer");
        std::fs::create_dir_all(&p).unwrap();
        std::fs::write(
            p.join("task_status.md"),
            "| Task ID | Status |\n|----|----|\n| T-001 | Complete |\n| T-002 | Complete |\n",
        )
        .unwrap();
        assert!(peer_tasks_all_complete(dir.path(), "ai-designer"));
    }

    #[test]
    fn peer_tasks_all_complete_returns_false_when_pending() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("rtl");
        std::fs::create_dir_all(&p).unwrap();
        std::fs::write(
            p.join("task_status.md"),
            "| Task ID | Status |\n|----|----|\n| T-001 | Complete |\n| T-002 | In progress |\n",
        )
        .unwrap();
        assert!(!peer_tasks_all_complete(dir.path(), "rtl"));
    }

    #[test]
    fn peer_tasks_all_complete_returns_false_when_no_entries() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("ai-reviewer");
        std::fs::create_dir_all(&p).unwrap();
        // Only headers, 0 entries -> treated as no tasks defined, false (prevents false termination).
        std::fs::write(
            p.join("task_status.md"),
            "# header\n\n| Task ID | Status |\n|----|----|\n",
        )
        .unwrap();
        assert!(!peer_tasks_all_complete(dir.path(), "ai-reviewer"));
    }

    #[test]
    fn peer_tasks_all_complete_returns_false_when_file_missing() {
        let dir = tempfile::tempdir().unwrap();
        // Missing file -> false (prevents false termination).
        assert!(!peer_tasks_all_complete(dir.path(), "rtl"));
    }

    // ─── conductors_ready_to_terminate (Phase 109) ──────────────────────
    fn write_status(dir: &Path, peer: &str, body: &str) {
        let p = dir.join(peer);
        std::fs::create_dir_all(&p).unwrap();
        std::fs::write(p.join("task_status.md"), body).unwrap();
    }

    #[test]
    fn conductors_ready_when_no_subagent_and_complete() {
        let dir = tempfile::tempdir().unwrap();
        write_status(
            dir.path(),
            "rtl",
            "| Task ID | Status |\n|----|----|\n| T-001 | Complete |\n",
        );
        let targets = vec![target_with_parent(
            "agent-A",
            "rtl",
            MonitorKind::DomainConductor,
            None,
        )];
        let got = conductors_ready_to_terminate(&targets, dir.path());
        assert_eq!(got, vec!["rtl".to_string()]);
    }

    #[test]
    fn conductors_blocked_when_subagent_still_present() {
        let dir = tempfile::tempdir().unwrap();
        write_status(
            dir.path(),
            "rtl",
            "| Task ID | Status |\n|----|----|\n| T-001 | Complete |\n",
        );
        let targets = vec![
            target_with_parent("agent-A", "rtl", MonitorKind::DomainConductor, None),
            target_with_parent(
                "agent-B",
                "rtl-coder-uart",
                MonitorKind::Subagent,
                Some("rtl"),
            ),
        ];
        // Sub-agent still present -> withhold conductor termination
        assert!(conductors_ready_to_terminate(&targets, dir.path()).is_empty());
    }

    #[test]
    fn conductors_blocked_when_tasks_pending() {
        let dir = tempfile::tempdir().unwrap();
        write_status(
            dir.path(),
            "asic",
            "| Task ID | Status |\n|----|----|\n| T-001 | In progress |\n",
        );
        let targets = vec![target_with_parent(
            "agent-A",
            "asic",
            MonitorKind::DomainConductor,
            None,
        )];
        assert!(conductors_ready_to_terminate(&targets, dir.path()).is_empty());
    }

    #[test]
    fn conductors_ready_empty_targets_returns_empty() {
        let dir = tempfile::tempdir().unwrap();
        assert!(conductors_ready_to_terminate(&[], dir.path()).is_empty());
    }

    // ─── is_terminable_status / clamp_terminate_grace (Phase 109) ──────
    #[test]
    fn is_terminable_only_for_idle_error_unknown() {
        assert!(is_terminable_status(AgentStatus::Idle));
        assert!(is_terminable_status(AgentStatus::Error));
        assert!(is_terminable_status(AgentStatus::Unknown));
        assert!(!is_terminable_status(AgentStatus::Busy));
        assert!(!is_terminable_status(AgentStatus::Waiting));
        assert!(!is_terminable_status(AgentStatus::Starting));
    }

    #[test]
    fn clamp_terminate_grace_bounds() {
        assert_eq!(clamp_terminate_grace(0), 0);
        assert_eq!(clamp_terminate_grace(10), 10);
        assert_eq!(clamp_terminate_grace(60), 60);
        assert_eq!(clamp_terminate_grace(61), 60);
        assert_eq!(clamp_terminate_grace(10_000), 60);
    }

    // ─── resolve_monitor_targets (Phase 109 extension: dynamic sub-agent adoption) ──
    #[test]
    fn resolve_includes_dynamic_subagents() {
        // Phase 110: ai is included as AiConductor + dynamic sub-agent adoption.
        let body = "agent-AAA  ai          ollama  glm  meta\n\
                    agent-BBB  ai-designer ollama  glm  designer\n\
                    agent-CCC  rtl         ollama  glm  conductor\n\
                    agent-DDD  rtl-coder-uart ollama glm worker\n\
                    agent-EEE  asic-signoff   ollama glm worker\n\
                    agent-FFF  bogus       ollama  glm  ???\n";
        let input = format!("{HEADER_LIST}{body}");
        let got = resolve_monitor_targets(&input);
        let names: Vec<&str> = got.iter().map(|t| t.peer.as_str()).collect();
        assert_eq!(
            names,
            vec!["ai", "ai-designer", "rtl", "rtl-coder-uart", "asic-signoff"]
        );
        // Verify parent_conductor
        assert_eq!(got[0].parent_conductor, None);
        assert_eq!(got[0].kind, MonitorKind::AiConductor);
        assert_eq!(got[1].parent_conductor.as_deref(), Some("ai"));
        assert_eq!(got[2].parent_conductor, None);
        assert_eq!(got[3].parent_conductor.as_deref(), Some("rtl"));
        assert_eq!(got[4].parent_conductor.as_deref(), Some("asic"));
    }

    // ─── Phase 110 — classify_peer extension tests ──────────────────────────
    #[test]
    fn classify_peer_dynamic_phase110_unchanged() {
        // Existing (Phase 109) dynamic sub-agent resolution is preserved.
        let (k, p) = classify_peer("rtl-coder-uart").unwrap();
        assert_eq!(k, MonitorKind::Subagent);
        assert_eq!(p, Some("rtl".to_string()));
    }

    // ─── Phase 110 — resolve_persona_for_peer ─────────────────────────
    #[test]
    fn resolve_persona_known_static_peers() {
        assert_eq!(resolve_persona_for_peer("ai"), Some("ai".to_string()));
        assert_eq!(
            resolve_persona_for_peer("ai-designer"),
            Some("ai-designer".to_string())
        );
        assert_eq!(resolve_persona_for_peer("rtl"), Some("rtl".to_string()));
        assert_eq!(
            resolve_persona_for_peer("hal-coder"),
            Some("hal-coder".to_string())
        );
    }

    #[test]
    fn resolve_persona_known_exception_asic_signoff() {
        assert_eq!(
            resolve_persona_for_peer("asic-signoff"),
            Some("asic-signoff-checker".to_string())
        );
    }

    #[test]
    fn resolve_persona_dynamic_coder_subagents() {
        assert_eq!(
            resolve_persona_for_peer("rtl-coder-uart"),
            Some("rtl-coder".to_string())
        );
        assert_eq!(
            resolve_persona_for_peer("hal-coder-i2c"),
            Some("hal-coder".to_string())
        );
        assert_eq!(
            resolve_persona_for_peer("apps-coder-firmware"),
            Some("apps-coder".to_string())
        );
    }

    #[test]
    fn resolve_persona_unknown_returns_same_name() {
        // Unknown peer names are returned as-is (caller checks persona file existence).
        assert_eq!(
            resolve_persona_for_peer("bogus-name"),
            Some("bogus-name".to_string())
        );
        assert_eq!(resolve_persona_for_peer(""), None);
    }

    // ─── Phase 110 — build_rescue_message (still used by starting_stall_rescue) ──
    #[test]
    fn rescue_message_includes_peer_paths_and_update_project() {
        let m = build_rescue_message("ai-designer");
        assert!(m.contains("ai-designer"));
        assert!(m.contains("update_project.md"));
        assert!(m.contains("tasks.md"));
        assert!(m.contains("task_status.md"));
        assert!(m.contains("SIGKILL"));
        assert!(m.contains("restarted"));
        assert!(m.contains("pending"));
    }

    // ─── Phase 110 — is_terminable_status Think support ──────────────────
    #[test]
    fn is_terminable_status_excludes_think_phase110() {
        // Think is excluded from termination (treated as active, same as Busy / Waiting / Starting).
        assert!(!is_terminable_status(AgentStatus::Think));
    }

    // ─── Phase 110 — summarize_statuses Think counting ────────────────────
    #[test]
    fn summarize_counts_think_phase110() {
        let s = statuses(&[
            ("agent-A", AgentStatus::Busy),
            ("agent-B", AgentStatus::Think),
            ("agent-C", AgentStatus::Think),
            ("agent-D", AgentStatus::Idle),
            ("agent-E", AgentStatus::Waiting),
        ]);
        let sum = summarize_statuses(&s);
        assert_eq!(sum.running, 5);
        assert_eq!(sum.busy, 1);
        assert_eq!(sum.think, 2);
        assert_eq!(sum.idle, 1);
        assert_eq!(sum.waiting, 1);
    }

    #[test]
    fn header_includes_think_and_wait_phase110() {
        let summary = StatusSummary {
            running: 4,
            busy: 1,
            think: 1,
            idle: 1,
            waiting: 1,
            error: 0,
            starting: 0,
            unknown: 0,
        };
        let h = build_monitor_header(&summary, 2, "2026-05-08T12:00:00Z");
        assert!(h.contains("THINKING: 1"));
        assert!(h.contains("WAIT: 1"));
        assert!(h.contains("BUSY: 1"));
        assert!(h.contains("IDLE: 1"));
    }
}
