//! Workspace output directory helpers for conductor handlers.
//!
//! Phase 19: each handler should produce real artifact files under
//! `.hestia/workspaces/<domain>/output/<run-id>/`. This module provides the
//! shared run-id resolution and directory creation logic so that handlers
//! across all conductors emit artifacts in a uniform layout.
//!
//! # Run-id resolution order
//! 1. `HESTIA_RUN_ID` environment variable (set by the AI orchestrator when
//!    invoking domain CLIs from `shell` tool calls)
//! 2. Fallback: `<UTC ISO8601 compact>-adhoc` timestamp string

use std::path::PathBuf;

/// Resolve the active run-id (env var first, fallback to timestamp).
pub fn resolve_run_id() -> String {
    if let Ok(value) = std::env::var("HESTIA_RUN_ID") {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    let now = chrono::Utc::now();
    format!("{}-adhoc", now.format("%Y%m%dT%H%M%SZ"))
}

/// Locate the active project root by walking up from the current working
/// directory looking for a `.hestia/` subdirectory. Falls back to the current
/// working directory when no such ancestor is found.
///
/// This avoids CWD-relative path nesting when handlers are invoked from a
/// conductor workspace such as `<root>/.hestia/workspaces/ai/` — without it
/// the relative path `.hestia/workspaces/<domain>/output/...` would be
/// resolved against `<root>/.hestia/workspaces/ai/` and produce a doubly
/// nested directory.
pub fn resolve_project_root() -> PathBuf {
    if let Ok(value) = std::env::var("HESTIA_PROJECT_ROOT") {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut cursor: Option<&std::path::Path> = Some(cwd.as_path());
    while let Some(dir) = cursor {
        if dir.join(".hestia").is_dir() {
            return dir.to_path_buf();
        }
        cursor = dir.parent();
    }
    cwd
}

/// Resolve the workspace output directory for a domain and the active run-id,
/// creating the directory if it does not exist.
///
/// Phase 20: this layout is now **internal only**. Project-facing artifacts
/// should be written under [`ensure_artifact_dir`] instead.
///
/// Layout: `<project-root>/.hestia/workspaces/<domain>/output/<run-id>/`
pub fn ensure_output_dir(domain: &str) -> Result<(String, PathBuf), String> {
    let run_id = resolve_run_id();
    let dir = resolve_project_root()
        .join(".hestia")
        .join("workspaces")
        .join(domain)
        .join("output")
        .join(&run_id);
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("ensure_output_dir({domain}, {run_id}): {e}"))?;
    Ok((run_id, dir))
}

/// Resolve a project-facing artifact directory for the given category and
/// optional subpath, creating it if it does not exist.
///
/// Phase 20 layout: `<project-root>/<category>/[subpath]/`
///
/// Examples:
/// - `ensure_artifact_dir("rtl", None)` → `<root>/rtl/`
/// - `ensure_artifact_dir("fpga", Some("constraints"))` → `<root>/fpga/constraints/`
/// - `ensure_artifact_dir("fpga", Some("scripts"))` → `<root>/fpga/scripts/`
///
/// Unlike [`ensure_output_dir`], this layout has **no run-id segment** —
/// project artifacts represent the current state of the project and are
/// overwritten on each run. Run-level history lives in
/// `<root>/.hestia/run_log/<run-id>.json` instead.
pub fn ensure_artifact_dir(category: &str, subpath: Option<&str>) -> Result<PathBuf, String> {
    let mut dir = resolve_project_root().join(category);
    if let Some(sp) = subpath {
        for segment in sp.split('/').filter(|s| !s.is_empty()) {
            dir = dir.join(segment);
        }
    }
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("ensure_artifact_dir({category}, {subpath:?}): {e}"))?;
    Ok(dir)
}

/// Phase 55b — Check whether an agent-cli peer is currently alive in the
/// shared registry by invoking `agent-cli list` and parsing its stdout.
///
/// Returns:
/// - `true`  if `peer_name` appears as a live peer
/// - `false` if `agent-cli` is not on PATH, fails, times out, or the peer is absent
///
/// Test override: setting `HESTIA_PEER_ALIVE_FORCE=<peer1>,<peer2>,...` causes
/// the listed peers to be reported as alive without invoking `agent-cli`.
/// This keeps unit tests deterministic in environments without a running
/// `agent-cli` process.
///
/// # Phase 87 — H11 root cause fix (agent-cli list output parsing fix)
///
/// The actual output format of `agent-cli list` is `<ID>  <NAME>  <PROVIDER>  <MODEL>  <ROLE>  <SKILLS>`,
/// where each line starts with **agent-id (`agent-01KXX...`)** and the peer name (NAME) appears
/// in the **2nd column**. The old Phase 55b implementation expected the peer name at the line start,
/// so it returned false even when a peer was definitely registered in `agent-cli list`
/// (the root cause of the ai-conductor ~80% takeover observed in Phase 83 = H11).
///
/// This fix changes the comparison to use the 2nd whitespace-separated column (NAME column)
/// as the peer name. The header line (`ID NAME PROVIDER ...`) will not match unless
/// `peer_name == "NAME"` (a peer named NAME effectively never exists).
///
/// Phase 113 — Resolve engine binary name.
///
/// The hestia binary sets env variable `HESTIA_ENGINE_BINARY` when spawning subprocesses,
/// so conductor-sdk reads its value to follow the engine abstraction.
/// Defaults to `agent-cli` when unset (backward compatible).
pub(crate) fn engine_binary() -> String {
    std::env::var("HESTIA_ENGINE_BINARY").unwrap_or_else(|_| "agent-cli".to_string())
}

pub fn agent_cli_peer_alive(peer_name: &str) -> bool {
    if let Ok(force) = std::env::var("HESTIA_PEER_ALIVE_FORCE") {
        if force
            .split(',')
            .map(|s| s.trim())
            .any(|p| p == peer_name)
        {
            return true;
        }
        // explicit override present but peer not listed — treat as offline
        return false;
    }

    let output = std::process::Command::new(engine_binary())
        .arg("list")
        .output();
    let Ok(out) = output else {
        return false;
    };
    if !out.status.success() {
        return false;
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Phase 87 H11 fix — parse the NAME column (whitespace-separated 2nd field)
    // instead of expecting peer name at line start. The first column is the
    // agent-id (e.g. `agent-01KQSXX...`), the second is the peer name.
    stdout.lines().any(|line| {
        let mut fields = line.split_whitespace();
        // skip the ID column
        let _id = fields.next();
        match fields.next() {
            Some(name) => name == peer_name,
            None => false,
        }
    })
}

/// Phase 129 — Scan the NAME column of `engine_binary() list` output and return
/// the number of peers starting with `prefix`. Returns 0 on registry query failure
/// (conservative fallback to avoid excessive cap suppression).
///
/// Usage: rtl-conductor / apps-conductor obtain the current alive coder count when
/// calling `dispatch_coders.v1` and enforce `per_conductor_max` as the **alive cap**
/// (suppressing cumulative alive count across multiple dispatch calls).
///
/// Follows engine abstraction and works with both agent-cli and claude-cli-shim
/// via `engine_binary()`.
///
/// Test compatibility: Like the existing `agent_cli_peer_alive`, if `HESTIA_PEER_ALIVE_FORCE`
/// env is set, returns the count of entries starting with `prefix` from its comma-separated
/// list (skipping actual engine query).
pub fn count_alive_peers_with_prefix(prefix: &str) -> usize {
    if let Ok(force) = std::env::var("HESTIA_PEER_ALIVE_FORCE") {
        return force
            .split(',')
            .map(|s| s.trim())
            .filter(|p| p.starts_with(prefix))
            .count();
    }

    let output = std::process::Command::new(engine_binary())
        .arg("list")
        .output();
    let Ok(out) = output else {
        return 0;
    };
    if !out.status.success() {
        return 0;
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    count_prefix_matches_in_listing(&stdout, prefix)
}

/// Phase 129 — Pure parse function for `count_alive_peers_with_prefix`.
/// Separated from I/O for unit testability.
pub(crate) fn count_prefix_matches_in_listing(stdout: &str, prefix: &str) -> usize {
    stdout
        .lines()
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            let _id = fields.next();
            fields.next()
        })
        .filter(|name| name.starts_with(prefix))
        .count()
}

/// Phase 84f / Phase 88 — Strict subagent mode determination.
///
/// Determine the behavior of each conductor's `<domain>.design.v1` handler
/// based on the value of env `HESTIA_STRICT_SUBAGENT`:
/// - `=1` or `=true` or **unset** → strict (`subagent_unavailable` + halt)
/// - `=0` or `=false` → non-strict (Phase 55b compatible `phase55b-fallback`)
///
/// **Changed default to strict ON in Phase 88 (1.7.0)**. This structurally resolves
/// the issue observed three consecutive times in Phase 83/84/87 where
/// "fallback became steady-state → sub-agent hierarchy permanently at 0% function".
/// Silent dependency on fallback (persona mislearning the fallback path as normal flow)
/// now requires explicit opt-out (`HESTIA_STRICT_SUBAGENT=0`).
pub fn strict_subagent_enabled() -> bool {
    match std::env::var("HESTIA_STRICT_SUBAGENT") {
        Ok(v) if v == "0" || v.eq_ignore_ascii_case("false") => false,
        // Phase 88: default → strict ON (old Phase 84f had default OFF)
        _ => true,
    }
}

/// Phase 84 — Wait until registry registration is confirmed.
///
/// Polls `agent-cli list` and waits up to `timeout_ms` milliseconds for `peer_name`
/// to be registered in the registry. Returns:
/// - `true`  ... registration confirmed within deadline
/// - `false` ... timeout or agent-cli not available
///
/// Usage: Called from `spawn_agent_cli` immediately after spawning a child process,
/// blocking until agent-cli becomes IPC ready, ensuring that subsequent design.v1 / dispatch
/// calls can immediately delegate to the peer.
///
/// Test override: If `HESTIA_PEER_ALIVE_FORCE` is set, use `agent_cli_peer_alive`'s
/// determination directly (test compatibility).
pub fn wait_for_registry(peer_name: &str, timeout_ms: u64) -> bool {
    let deadline = std::time::Instant::now()
        + std::time::Duration::from_millis(timeout_ms);
    let interval = std::time::Duration::from_millis(200);
    loop {
        if agent_cli_peer_alive(peer_name) {
            return true;
        }
        if std::time::Instant::now() >= deadline {
            return false;
        }
        std::thread::sleep(interval);
    }
}

/// Phase 80 — Generic helper to auto-spawn ai-reviewer after dispatch_*.v1 completion.
///
/// Can be called from the end of each conductor's `<domain>.dispatch_*.v1` method,
/// sending a fire-and-forget prompt to ai-reviewer requesting
/// "review of the dispatch scope".
///
/// Arguments:
/// - `parent_conductor`: parent conductor's peer name (e.g. "rtl")
/// - `dispatch_method`: executed dispatch method name (e.g. "rtl.dispatch_coders.v1")
/// - `spawned_count`: number of dynamically spawned sub-agents
///
/// Returns: true on dispatch success, false on failure (hestia not present / agent-cli not present, etc.).
/// Failure only logs a warning and does not affect the overall dispatch.
///
/// Can be disabled with env override `HESTIA_DISABLE_AUTO_REVIEW=1` (shared with Phase 77).
pub fn auto_review_after_dispatch(
    parent_conductor: &str,
    dispatch_method: &str,
    spawned_count: usize,
) -> bool {
    if std::env::var("HESTIA_DISABLE_AUTO_REVIEW").as_deref() == Ok("1") {
        return false;
    }
    if spawned_count == 0 {
        // If 0 spawns, there is nothing to review, so skip
        return false;
    }
    // hestia spawn-subagent --persona ai-reviewer --name ai-reviewer
    let spawn_result = std::process::Command::new("hestia")
        .args(["spawn-subagent", "--persona", "ai-reviewer", "--name", "ai-reviewer"])
        .output();
    if !matches!(&spawn_result, Ok(o) if o.status.success()) {
        return false;
    }
    // Phase 102 — Review artifacts are written directly under the project root (`.aiprj/` is the project-managed AI-exclusive area).
    let prompt = format!(
        "[{dispatch_method} auto-review] parent={parent_conductor} spawned_count={spawned_count}. Review the dynamic sub-agent outputs and write `<root>/REVIEW_REPORT_dispatch.md`."
    );
    agent_cli_send("ai-reviewer", &prompt).is_ok()
}

/// Phase 55c — Best-effort fire-and-forget message dispatch via `agent-cli send`.
/// Returns `Ok(())` if the send subprocess exited 0, `Err(message)` otherwise.
///
/// Used by handler `<domain>.design.v1` paths to dispatch a delegation prompt
/// to the corresponding `<domain>-designer` sub-agent without blocking on the
/// LLM inference loop. The actual artifact production happens asynchronously
/// in the designer's agent-cli process; the orchestrator (ai-conductor LLM)
/// observes completion by checking for the `expected_artifacts` files.
///
/// Test override: setting `HESTIA_PEER_SEND_NOOP=1` makes this a no-op that
/// returns Ok(()) without invoking agent-cli — used in unit tests where no
/// agent-cli registry is available.
pub fn agent_cli_send(peer_name: &str, text: &str) -> Result<(), String> {
    if std::env::var("HESTIA_PEER_SEND_NOOP").as_deref() == Ok("1") {
        return Ok(());
    }
    let bin = engine_binary();
    let output = std::process::Command::new(&bin)
        .arg("send")
        .arg(peer_name)
        .arg(text)
        .output()
        .map_err(|e| format!("{bin} send {peer_name}: spawn failed: {e}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "{bin} send {peer_name}: exit {} stderr={}",
            output.status, stderr.trim()
        ));
    }
    Ok(())
}

/// Phase 93 — Spawn a domain conductor on-demand via `hestia start <domain>`.
///
/// Part of the Phase 93 startup model redesign: `hestia start` (no arguments)
/// starts only the ai-conductor, and domain conductors
/// (rtl/fpga/asic/pcb/hal/apps/debug/rag) are started on-demand by the
/// ai-conductor at dispatch time using this helper.
///
/// Behavior:
/// 1. Check liveness with `agent_cli_peer_alive(domain)`
/// 2. If absent, spawn `hestia start <domain>` (detached / fire-and-forget)
/// 3. Wait for registry registration with `wait_for_registry(domain, 5000ms)`
/// 4. Return value: `Ok(())` on successful startup, `Err` on registry registration failure
///
/// No-op for already-live peers (returns Ok).
///
/// Environment variables:
/// - `HESTIA_PEER_SEND_NOOP=1` — skip spawn for testing and return Ok
pub fn spawn_conductor_on_demand(domain: &str) -> Result<(), String> {
    if std::env::var("HESTIA_PEER_SEND_NOOP").as_deref() == Ok("1") {
        return Ok(());
    }
    if agent_cli_peer_alive(domain) {
        return Ok(()); // Already running
    }
    // detached spawn: start `hestia start <domain>` in background
    let result = std::process::Command::new("hestia")
        .arg("start")
        .arg(domain)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .stdin(std::process::Stdio::null())
        .spawn();
    if let Err(e) = result {
        return Err(format!(
            "spawn_conductor_on_demand({domain}): hestia start spawn failed: {e}"
        ));
    }
    // Wait for registry registration (max 5 seconds)
    if !wait_for_registry(domain, 5000) {
        return Err(format!(
            "spawn_conductor_on_demand({domain}): registry timeout 5000ms"
        ));
    }
    Ok(())
}

/// Look up an executable in `PATH` without taking on a `which` crate dependency.
pub fn find_in_path(name: &str) -> Option<PathBuf> {
    if let Some(slash) = name.find('/') {
        let _ = slash;
        let p = PathBuf::from(name);
        if p.is_file() {
            return Some(p);
        }
    }
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

// Phase 42: `find_project_template` was removed. Hestia is an agent-driven
// hardware development environment — artifacts (RTL, register maps, XDC,
// TCL, etc.) must be designed and emitted by the AI orchestrator via
// `fs_write` to the project root, not loaded from pre-placed templates.
// Allowing a template fallback degraded the system into a template-substitution
// engine and caused the AI persona to tell users to "go place a template
// then re-run", which is exactly the opposite of an AI-driven workflow.
// Handlers now resolve inputs only via params and existing project files.

/// First existing project file under `<project-root>/<category>/[subpath]/<name>`.
///
/// Used by handlers to consume artifacts that the AI orchestrator generated
/// for this run (e.g. `<root>/rtl/uart_led.sv` written by `fs_write` before
/// the lint step).
pub fn find_project_file(category: &str, subpath: Option<&str>, name: &str) -> Option<PathBuf> {
    let mut path = resolve_project_root().join(category);
    if let Some(sp) = subpath {
        for segment in sp.split('/').filter(|s| !s.is_empty()) {
            path = path.join(segment);
        }
    }
    let path = path.join(name);
    if path.is_file() { Some(path) } else { None }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Phase 129 — Pure parse test for `count_prefix_matches_in_listing`.
    /// Verify that it works with the same parsing convention (NAME column = 2nd whitespace token)
    /// for both agent-cli and claude-cli-shim list output.
    #[test]
    fn count_prefix_matches_zero_when_empty() {
        assert_eq!(count_prefix_matches_in_listing("", "rtl-coder-"), 0);
    }

    #[test]
    fn count_prefix_matches_skips_header() {
        // Typically the first line is a header with "NAME" etc. in the NAME column. Filtering by prefix yields 0.
        let stdout = "ID  NAME  PROVIDER\n";
        assert_eq!(count_prefix_matches_in_listing(stdout, "rtl-coder-"), 0);
    }

    #[test]
    fn count_prefix_matches_counts_rtl_coders() {
        let stdout = "\
ID                                NAME                       PROVIDER  MODEL
shim-aaa-1                        ai                         claude    claude-opus-4-7
shim-bbb-2                        ai-designer                claude    claude-opus-4-7
shim-ccc-3                        rtl-coder-axi              claude    claude-opus-4-7
shim-ddd-4                        rtl-coder-bootrom          claude    claude-opus-4-7
shim-eee-5                        rtl                        claude    claude-opus-4-7
shim-fff-6                        apps-coder-uart            claude    claude-opus-4-7
";
        assert_eq!(count_prefix_matches_in_listing(stdout, "rtl-coder-"), 2);
        assert_eq!(count_prefix_matches_in_listing(stdout, "apps-coder-"), 1);
        assert_eq!(count_prefix_matches_in_listing(stdout, "ai-"), 1); // ai-designer only
        assert_eq!(count_prefix_matches_in_listing(stdout, "rtl"), 3); // rtl, rtl-coder-axi, rtl-coder-bootrom
    }

    #[test]
    fn count_prefix_matches_handles_agent_cli_format() {
        // Can also parse agent-cli's `agent-` prefix format
        let stdout = "\
agent-01KQX72WJY3Z59RN77YXB9Z02P  rtl-coder-i2c   ollama  glm-5.1:cloud
agent-01KQX72WT3DY2GKWSDDXA9QK0K  rtl-coder-spi   ollama  glm-5.1:cloud
";
        assert_eq!(count_prefix_matches_in_listing(stdout, "rtl-coder-"), 2);
    }
}