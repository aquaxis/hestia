//! Conductor registration and status tracking

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use tracing::{info, warn};

use conductor_sdk::agent::{ConductorId, ConductorInfo, ConductorStatus};

/// Manages the set of known conductors and their current state.
#[derive(Debug)]
pub struct ConductorManager {
    conductors: Arc<RwLock<HashMap<ConductorId, ConductorInfo>>>,
}

impl ConductorManager {
    /// Create an empty conductor manager.
    pub fn new() -> Self {
        Self {
            conductors: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register (or re-register) a conductor.
    ///
    /// If the conductor was already known its info is replaced; otherwise a new
    /// entry is inserted with `ConductorStatus::Online`.
    pub async fn register(&self, id: ConductorId, info: ConductorInfo) {
        let mut map = self.conductors.write().await;
        info!(conductor = %id, "registering conductor");
        map.insert(id, info);
    }

    /// Unregister a conductor, removing it from the active set.
    ///
    /// Returns `true` when the conductor was present and removed.
    pub async fn unregister(&self, id: ConductorId) -> bool {
        let mut map = self.conductors.write().await;
        if map.remove(&id).is_some() {
            info!(conductor = %id, "unregistered conductor");
            true
        } else {
            warn!(conductor = %id, "attempted to unregister unknown conductor");
            false
        }
    }

    /// Query the current status of a specific conductor.
    pub async fn get_status(&self, id: ConductorId) -> Option<ConductorStatus> {
        let map = self.conductors.read().await;
        map.get(&id).map(|info| info.status)
    }

    /// Return a snapshot of all registered conductors.
    pub async fn list_conductors(&self) -> Vec<ConductorInfo> {
        let map = self.conductors.read().await;
        map.values().cloned().collect()
    }
}

impl Default for ConductorManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn register_and_list() {
        let mgr = ConductorManager::new();
        let info = ConductorInfo {
            id: ConductorId::Ai,
            status: ConductorStatus::Online,
            version: "0.1.0".into(),
            uptime_secs: 42,
        };
        mgr.register(ConductorId::Ai, info).await;
        let list = mgr.list_conductors().await;
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, ConductorId::Ai);
    }

    #[tokio::test]
    async fn unregister_returns_false_when_unknown() {
        let mgr = ConductorManager::new();
        assert!(!mgr.unregister(ConductorId::Rtl).await);
    }
}