//! Workspace helper integration tests (Phase 23).
//!
//! `find_in_path` must locate executables present on PATH.  The verilator
//! regression test (`HESTIA_TOOLS_REQUIRED` opt-in) asserts the developer
//! environment still has the tool available; CI environments without
//! verilator simply skip the assertion.

use conductor_sdk::workspace;

#[test]
fn find_in_path_locates_sh() {
    // /bin/sh exists on every reasonable Unix; this is a smoke test.
    assert!(
        workspace::find_in_path("sh").is_some(),
        "find_in_path failed to locate /bin/sh — basic PATH walk regression"
    );
}

#[test]
fn find_in_path_returns_none_for_missing_binary() {
    assert!(
        workspace::find_in_path("definitely-not-a-real-binary-name-xyzzy").is_none(),
        "find_in_path returned Some for a clearly missing binary"
    );
}

#[test]
fn find_in_path_resolves_required_tools_when_opted_in() {
    // Opt-in regression test: only fires when HESTIA_TOOLS_REQUIRED is set
    // (e.g. `HESTIA_TOOLS_REQUIRED=verilator,vivado cargo test`).  Skipped
    // silently otherwise so CI without the toolchain still passes.
    let Ok(required) = std::env::var("HESTIA_TOOLS_REQUIRED") else {
        return;
    };
    for tool in required.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        assert!(
            workspace::find_in_path(tool).is_some(),
            "HESTIA_TOOLS_REQUIRED listed `{tool}` but it is not on PATH"
        );
    }
}

#[test]
fn resolve_run_id_honors_env_var() {
    let prior = std::env::var("HESTIA_RUN_ID").ok();
    std::env::set_var("HESTIA_RUN_ID", "20260503T010203Z-test1234");
    let id = workspace::resolve_run_id();
    assert_eq!(id, "20260503T010203Z-test1234");

    match prior {
        Some(v) => std::env::set_var("HESTIA_RUN_ID", v),
        None => std::env::remove_var("HESTIA_RUN_ID"),
    }
}

#[test]
fn resolve_run_id_falls_back_to_timestamp_when_env_empty() {
    let prior = std::env::var("HESTIA_RUN_ID").ok();
    std::env::remove_var("HESTIA_RUN_ID");
    let id = workspace::resolve_run_id();
    assert!(
        id.ends_with("-adhoc"),
        "fallback run_id should end with -adhoc, got {id}"
    );

    if let Some(v) = prior {
        std::env::set_var("HESTIA_RUN_ID", v);
    }
}
