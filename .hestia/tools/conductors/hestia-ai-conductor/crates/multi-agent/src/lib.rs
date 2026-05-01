//! multi-agent -- Agent lifecycle, inter-agent messaging, and session tracking

pub mod agent_manager;
pub mod message_broker;
pub mod session;

pub use agent_manager::AgentManager;
pub use message_broker::MessageBroker;
pub use session::Session;