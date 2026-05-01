//! conductor-client -- Thin wrapper around the SDK transport layer
//!
//! Re-exports the SDK's [`AgentCliClient`] and [`HestiaClientConfig`] and
//! provides a [`ConductorClient`] convenience struct.

pub use conductor_sdk::transport::AgentCliClient;
pub use conductor_sdk::config::HestiaClientConfig;

use conductor_sdk::agent::ConductorId;
use conductor_sdk::error::HestiaError;
use conductor_sdk::message::Payload;

/// High-level client for communicating with Hestia conductors.
///
/// Wraps [`AgentCliClient`] and provides ergonomic methods tailored to the
/// ai-conductor domain.
pub struct ConductorClient {
    inner: AgentCliClient,
}

impl ConductorClient {
    /// Create a new conductor client from the given configuration.
    pub fn new(config: HestiaClientConfig) -> Result<Self, HestiaError> {
        let inner = AgentCliClient::new(config)?;
        Ok(Self { inner })
    }

    /// List all currently connected peers.
    pub async fn list_peers(&self) -> Result<Vec<String>, HestiaError> {
        self.inner.list_peers().await
    }

    /// Send a payload to a specific conductor.
    pub async fn send_to_conductor(
        &self,
        conductor: ConductorId,
        payload: &Payload,
    ) -> Result<String, HestiaError> {
        self.inner.send_to_conductor(conductor, payload).await
    }

    /// Send a payload to an arbitrary peer by name.
    pub async fn send(&self, peer: &str, payload: &Payload) -> Result<String, HestiaError> {
        self.inner.send(peer, payload).await
    }

    /// Access the underlying SDK client.
    pub fn inner(&self) -> &AgentCliClient {
        &self.inner
    }
}