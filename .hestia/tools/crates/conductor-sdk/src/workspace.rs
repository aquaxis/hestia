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
/// shared registry by invoking `agent-cli list` and grepping its stdout.
///
/// Returns:
/// - `true`  if `peer_name` appears as a live peer
/// - `false` if `agent-cli` is not on PATH, fails, times out, or the peer is absent
///
/// Test override: setting `HESTIA_PEER_ALIVE_FORCE=<peer1>,<peer2>,...` causes
/// the listed peers to be reported as alive without invoking `agent-cli`.
/// This keeps unit tests deterministic in environments without a running
/// `agent-cli` process.
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

    let output = std::process::Command::new("agent-cli")
        .arg("list")
        .output();
    let Ok(out) = output else {
        return false;
    };
    if !out.status.success() {
        return false;
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    // agent-cli list output starts each peer line with the peer name.
    stdout.lines().any(|line| {
        let trimmed = line.trim_start();
        trimmed.starts_with(&format!("{peer_name} "))
            || trimmed == peer_name
            || trimmed.starts_with(&format!("{peer_name}\t"))
    })
}

/// Phase 80 — dispatch_*.v1 完了後に ai-reviewer を auto-spawn する汎用ヘルパ。
///
/// 各 conductor の `<domain>.dispatch_*.v1` メソッド末尾から呼出可能で、ai-reviewer に
/// 「dispatch スコープのレビューを依頼」する prompt を送信する fire-and-forget 経路。
///
/// 引数:
/// - `parent_conductor`: 親 conductor の peer 名（例 "rtl"）
/// - `dispatch_method`: 実行された dispatch メソッド名（例 "rtl.dispatch_coders.v1"）
/// - `spawned_count`: 動的 spawn された sub-agent 数
///
/// 返却: dispatch 成功なら true、失敗（hestia 不在 / agent-cli 不在等）なら false。
/// 失敗は warn ログのみで dispatch 全体に影響なし。
///
/// env override `HESTIA_DISABLE_AUTO_REVIEW=1` で無効化可能（Phase 77 と共通）。
pub fn auto_review_after_dispatch(
    parent_conductor: &str,
    dispatch_method: &str,
    spawned_count: usize,
) -> bool {
    if std::env::var("HESTIA_DISABLE_AUTO_REVIEW").as_deref() == Ok("1") {
        return false;
    }
    if spawned_count == 0 {
        // spawn 0 件なら review する対象がないため skip
        return false;
    }
    // hestia spawn-subagent --persona ai-reviewer --name ai-reviewer
    let spawn_result = std::process::Command::new("hestia")
        .args(["spawn-subagent", "--persona", "ai-reviewer", "--name", "ai-reviewer"])
        .output();
    if !matches!(&spawn_result, Ok(o) if o.status.success()) {
        return false;
    }
    let prompt = format!(
        "[{dispatch_method} auto-review] parent={parent_conductor} spawned_count={spawned_count}. Review the dynamic sub-agent outputs and write `<root>/.aiprj/REVIEW_REPORT_dispatch.md`."
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
    let output = std::process::Command::new("agent-cli")
        .arg("send")
        .arg(peer_name)
        .arg(text)
        .output()
        .map_err(|e| format!("agent-cli send {peer_name}: spawn failed: {e}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "agent-cli send {peer_name}: exit {} stderr={}",
            output.status, stderr.trim()
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
