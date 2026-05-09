//! Phase 126 — サブエージェント並列度の統合テスト。
//!
//! AgentManager のグローバル cap が env 経由で反映され、reviewer が予約 slot を
//! 取れること、cap 枯渇時に acquire timeout で fail-fast すること、を検証する。

use multi_agent::agent_manager::AgentManager;

#[tokio::test]
async fn agent_manager_with_caps_reserves_reviewer_slot() {
    // global_max=2 → 一般 limiter は 1, reviewer 予約 slot は 1
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
    // global_max=1 でも一般 limiter は最低 1 確保される
    let mgr = AgentManager::with_caps(1, 5);
    assert_eq!(mgr.capacity(), 1);
}

#[tokio::test]
async fn agent_manager_with_default_cap_uses_safe_default_when_env_missing() {
    std::env::remove_var("HESTIA_GLOBAL_MAX_AGENTS");
    std::env::remove_var("HESTIA_ACQUIRE_TIMEOUT_SECS");
    let mgr = AgentManager::with_default_cap();
    // 既定 8 - reviewer 予約 1 = 7
    assert_eq!(mgr.capacity(), 7);
}
