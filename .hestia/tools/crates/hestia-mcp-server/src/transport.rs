use crate::tools::ToolDefinition;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRequest {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse {
    pub jsonrpc: String,
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<McpResponseError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponseError {
    pub code: i32,
    pub message: String,
}

pub struct McpTransport;

impl McpTransport {
    pub fn handle_request(request: McpRequest) -> McpResponse {
        match request.method.as_str() {
            "tools/list" => McpResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(serde_json::to_value(ToolListResult {
                    tools: ToolDefinition::get_tool_definitions(),
                })
                .unwrap_or_default()),
                error: None,
            },
            "tools/call" => McpResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(Value::String("tool execution dispatched via agent-cli".to_string())),
                error: None,
            },
            "initialize" => McpResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(serde_json::json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": { "tools": {} },
                    "serverInfo": { "name": "hestia-mcp-server", "version": "0.1.0" }
                })),
                error: None,
            },
            _ => McpResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(McpResponseError {
                    code: -32601,
                    message: format!("method not found: {}", request.method),
                }),
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ToolListResult {
    tools: Vec<ToolDefinition>,
}

impl ToolDefinition {
    fn get_tool_definitions() -> Vec<ToolDefinition> {
        crate::tools::get_tool_definitions()
    }
}