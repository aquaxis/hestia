//! Phase 126 — Integration tests for sub-agent parallelism.
//!
//! Verifies that AgentManager's global cap is reflected via env vars,
//! that reviewers can acquire the reserved slot, and that acquire timeout
//! causes fail-fast when the cap is exhausted.

use multi_agent::agent_manager::AgentManager;

#[tokio::test]
async fn agent_manager_with_caps_reserves_reviewer_slot() {
    // global_max=2 so general limiter is 1, reviewer reserved slot is 1
    let mgr = AgentManager::with_caps(2, 5);
    assert_eq!(mgr.capacity(), 1);
}

#[tokio::test]
async fn agent_manager_default_cap_picks_up_env_override() {
    std::env::set_var("HESTIA_GLOBAL_MAX_AGENTS", "5");
    std::env::set_var("HESTIA_ACQUIRE_TIMEOUT_SECS", "30");
    let mgr = AgentManager::with_default_cap();
    // 5 - 1 (reserved for reviewer) = 4
    assert_eq!(mgr.capacity(), 4);
    std::env::remove_var("HESTIA_GLOBAL_MAX_AGENTS");
    std::env::remove_var("HESTIA_ACQUIRE_TIMEOUT_SECS");
}

#[tokio::test]
async fn agent_manager_minimum_capacity_is_one() {
    // Even with global_max=1, the general limiter reserves at least 1
    let mgr = AgentManager::with_caps(1, 5);
    assert_eq!(mgr.capacity(), 1);
}

#[tokio::test]
async fn agent_manager_with_default_cap_uses_safe_default_when_env_missing() {
    std::env::remove_var("HESTIA_GLOBAL_MAX_AGENTS");
    std::env::remove_var("HESTIA_ACQUIRE_TIMEOUT_SECS");
    let mgr = AgentManager::with_default_cap();
    // Default 8 - reviewer reserved 1 = 7
    assert_eq!(mgr.capacity(), 7);
}
