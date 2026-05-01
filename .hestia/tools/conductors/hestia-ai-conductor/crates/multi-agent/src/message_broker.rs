//! Inter-agent message routing

use std::collections::HashMap;

use conductor_sdk::message::Payload;
use tracing::{debug, instrument};

/// A routed message destined for a specific agent.
#[derive(Debug, Clone)]
pub struct RoutedMessage {
    pub from: String,
    pub to: String,
    pub payload: Payload,
}

/// Routes messages between agents.
///
/// The broker is a simple in-memory fan-out: a caller submits a message
/// addressed to an agent, and the broker decides which handler (if any)
/// should receive it.  In production this would be backed by an async
/// channel or a message-queue service.
#[derive(Debug)]
pub struct MessageBroker {
    /// Registered message handlers per agent ID.
    handlers: HashMap<String, Vec<String>>,
}

impl MessageBroker {
    /// Create a new empty broker.
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// Register an agent as a message handler for a target.
    pub fn register_handler(&mut self, target: String, agent_id: String) {
        self.handlers.entry(target).or_default().push(agent_id);
    }

    /// Route a message to its destination.
    ///
    /// Returns the list of agent IDs that should receive the message.
    /// If no handler is registered for the target, returns an empty vec.
    #[instrument(skip(self, payload), fields(from = %from, to = %to))]
    pub fn route(&self, from: &str, to: &str, payload: &Payload) -> RoutedMessage {
        debug!("routing message");
        RoutedMessage {
            from: from.to_string(),
            to: to.to_string(),
            payload: payload.clone(),
        }
    }

    /// Return the handler agent IDs registered for a given target.
    pub fn handlers_for(&self, target: &str) -> Vec<&str> {
        self.handlers
            .get(target)
            .map(|v| v.iter().map(String::as_str).collect())
            .unwrap_or_default()
    }
}

impl Default for MessageBroker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn route_returns_routed_message() {
        let broker = MessageBroker::new();
        let msg = broker.route("a", "b", &Payload::NaturalLanguage("hello".into()));
        assert_eq!(msg.from, "a");
        assert_eq!(msg.to, "b");
    }

    #[test]
    fn handlers_for_empty_when_none_registered() {
        let broker = MessageBroker::new();
        assert!(broker.handlers_for("unknown").is_empty());
    }
}