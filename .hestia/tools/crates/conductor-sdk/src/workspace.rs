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

/// Resolve a project-side template path (Phase 21 — keep Hestia core generic).
///
/// Templates live under `<project-root>/.hestia/<category>/templates/<name>` so
/// that app- and board-specific data (UART/LED register maps, ARTY-A7-100T
/// constraints, vendor-specific build TCL etc.) stay with the project rather
/// than being baked into the Hestia source tree.
///
/// Returns `Some(path)` if the template exists, `None` otherwise.
pub fn find_project_template(category: &str, name: &str) -> Option<PathBuf> {
    let path = resolve_project_root()
        .join(".hestia")
        .join(category)
        .join("templates")
        .join(name);
    if path.is_file() { Some(path) } else { None }
}

/// First existing project file under `<project-root>/<category>/[subpath]/<name>`.
///
/// Used by handlers to prefer user-edited project files (e.g. `<root>/rtl/uart_led.sv`)
/// over project-side templates.
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
