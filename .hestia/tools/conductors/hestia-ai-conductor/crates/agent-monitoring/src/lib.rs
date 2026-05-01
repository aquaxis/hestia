//! agent-monitoring -- Live observation and health checking for agents

pub mod health_check;
pub mod live_view;

pub use health_check::AgentHealthChecker;
pub use live_view::LiveView;