//! ASIC conductor RPC handler

use conductor_sdk::message::{Request, Response, SuccessResponse};

/// Handle an incoming RPC request for the ASIC conductor.
pub async fn handle_request(req: Request) -> Response {
    tracing::info!(method = %req.method, "handling ASIC RPC request");
    Response::Success(SuccessResponse {
        result: serde_json::json!({
            "status": "ok",
            "method": req.method,
        }),
        id: req.id,
        trace_id: req.trace_id,
    })
}