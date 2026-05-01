//! agent-communication -- Message types and communication bus for inter-agent routing
//!
//! Re-exports the core message types from `conductor_sdk::message` and
//! provides a [`CommunicationBus`] that routes messages between agents.

// Re-export message types from the SDK so consumers only need this crate.
pub use conductor_sdk::message::{
    BatchRequest, MessageId, MethodName, Notification, Payload, Request, Response,
    SuccessResponse, TraceId,
};

use std::collections::HashMap;

use tracing::{debug, instrument};

/// A directed message on the communication bus.
#[derive(Debug, Clone)]
pub struct BusMessage {
    pub from_agent: String,
    pub to_agent: String,
    pub payload: Payload,
    pub trace_id: Option<TraceId>,
}

/// Central communication bus that routes messages between agents.
///
/// Agents register with the bus; when a message is routed the bus determines
/// the destination based on the `to_agent` field.  In a production setting
/// this would be backed by async channels or an external message broker.
#[derive(Debug)]
pub struct CommunicationBus {
    /// Agents known to the bus.
    registered: HashMap<String, Vec<BusMessage>>,
}

impl CommunicationBus {
    /// Create a new empty communication bus.
    pub fn new() -> Self {
        Self {
            registered: HashMap::new(),
        }
    }

    /// Register an agent on the bus.
    pub fn register(&mut self, agent_id: String) {
        self.registered.entry(agent_id.clone()).or_default();
        debug!(agent = %agent_id, "agent registered on communication bus");
    }

    /// Check whether an agent is registered.
    pub fn is_registered(&self, agent_id: &str) -> bool {
        self.registered.contains_key(agent_id)
    }

    /// Route a message from one agent to another.
    ///
    /// If the target agent is not registered the message is dropped and `false`
    /// is returned.  Otherwise the message is appended to the target's inbound
    /// queue and `true` is returned.
    #[instrument(skip(self, payload))]
    pub fn route(
        &mut self,
        from_agent: &str,
        to_agent: &str,
        payload: Payload,
        trace_id: Option<TraceId>,
    ) -> bool {
        if !self.registered.contains_key(to_agent) {
            debug!(to = %to_agent, "target agent not registered, dropping message");
            return false;
        }
        let msg = BusMessage {
            from_agent: from_agent.to_string(),
            to_agent: to_agent.to_string(),
            payload,
            trace_id,
        };
        self.registered
            .get_mut(to_agent)
            .expect("checked above")
            .push(msg);
        debug!(from = %from_agent, to = %to_agent, "message routed");
        true
    }

    /// Drain all pending messages for an agent.
    pub fn drain(&mut self, agent_id: &str) -> Vec<BusMessage> {
        self.registered
            .get_mut(agent_id)
            .map(|q| std::mem::take(q))
            .unwrap_or_default()
    }
}

impl Default for CommunicationBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_and_route() {
        let mut bus = CommunicationBus::new();
        bus.register("agent-a".into());
        bus.register("agent-b".into());

        let sent = bus.route(
            "agent-a",
            "agent-b",
            Payload::NaturalLanguage("hello".into()),
            None,
        );
        assert!(sent);

        let msgs = bus.drain("agent-b");
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].from_agent, "agent-a");
    }

    #[test]
    fn route_to_unknown_drops() {
        let mut bus = CommunicationBus::new();
        bus.register("agent-a".into());
        let sent = bus.route(
            "agent-a",
            "unknown",
            Payload::NaturalLanguage("hello".into()),
            None,
        );
        assert!(!sent);
    }

    #[test]
    fn drain_empty_returns_nothing() {
        let mut bus = CommunicationBus::new();
        bus.register("agent-a".into());
        assert!(bus.drain("agent-a").is_empty());
    }
}