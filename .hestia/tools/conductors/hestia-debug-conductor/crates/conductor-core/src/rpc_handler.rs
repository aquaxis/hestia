//! RPC handler for the debug conductor daemon

use conductor_sdk::message::Request;
use conductor_sdk::message::Response;
use conductor_sdk::message::SuccessResponse;

use crate::session::SessionStateMachine;

/// RPC handler that dispatches incoming debug-conductor requests.
pub struct RpcHandler {
    state_machine: SessionStateMachine,
}

impl RpcHandler {
    /// Create a new RPC handler with a fresh session state machine.
    pub fn new() -> Self {
        Self {
            state_machine: SessionStateMachine::new(),
        }
    }

    /// Handle an incoming RPC request.
    pub async fn handle(&mut self, request: &Request) -> Response {
        let result = match request.method.as_str() {
            "debug.session.connect" => self.handle_connect(&request.params),
            "debug.session.disconnect" => self.handle_disconnect(),
            "debug.session.pause" => self.handle_pause(),
            "debug.session.resume" => self.handle_resume(),
            "debug.session.reset" => self.handle_reset(&request.params),
            "debug.session.status" => self.handle_status(),
            _ => Err(format!("Unknown method: {}", request.method)),
        };

        match result {
            Ok(value) => Response::Success(SuccessResponse {
                result: value,
                id: request.id.clone(),
                trace_id: request.trace_id.clone(),
            }),
            Err(msg) => {
                let err = conductor_sdk::error::ErrorResponse {
                    code: conductor_sdk::error::error_code::DEBUG_START,
                    message: msg,
                    data: None,
                };
                Response::Error(conductor_sdk::message::ErrorResultResponse {
                    error: err,
                    id: request.id.clone(),
                    trace_id: request.trace_id.clone(),
                })
            }
        }
    }

    fn handle_connect(&mut self, _params: &serde_json::Value) -> Result<serde_json::Value, String> {
        self.state_machine
            .transition(crate::session::SessionState::Connecting)
            .map_err(|s| format!("Invalid transition from {s} to Connecting"))?;
        // Simulate connection establishment
        self.state_machine
            .transition(crate::session::SessionState::Connected)
            .map_err(|s| format!("Invalid transition from {s} to Connected"))?;
        Ok(serde_json::json!({"status": "connected"}))
    }

    fn handle_disconnect(&mut self) -> Result<serde_json::Value, String> {
        self.state_machine
            .transition(crate::session::SessionState::Disconnected)
            .map_err(|s| format!("Invalid transition from {s} to Disconnected"))?;
        Ok(serde_json::json!({"status": "disconnected"}))
    }

    fn handle_pause(&mut self) -> Result<serde_json::Value, String> {
        self.state_machine
            .transition(crate::session::SessionState::Paused)
            .map_err(|s| format!("Invalid transition from {s} to Paused"))?;
        Ok(serde_json::json!({"status": "paused"}))
    }

    fn handle_resume(&mut self) -> Result<serde_json::Value, String> {
        self.state_machine
            .transition(crate::session::SessionState::Running)
            .map_err(|s| format!("Invalid transition from {s} to Running"))?;
        Ok(serde_json::json!({"status": "running"}))
    }

    fn handle_reset(&mut self, _params: &serde_json::Value) -> Result<serde_json::Value, String> {
        // Reset returns to Connected state
        self.state_machine
            .transition(crate::session::SessionState::Connected)
            .map_err(|s| format!("Invalid transition from {s} to Connected"))?;
        Ok(serde_json::json!({"status": "connected", "reset": true}))
    }

    fn handle_status(&self) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "state": self.state_machine.state().to_string()
        }))
    }
}

impl Default for RpcHandler {
    fn default() -> Self {
        Self::new()
    }
}