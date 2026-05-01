use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

pub fn get_tool_definitions() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "hestia_rag_search".to_string(),
            description: "Search the Hestia knowledge base via RAG".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Search query" },
                    "top_k": { "type": "integer", "description": "Number of results", "default": 5 }
                },
                "required": ["query"]
            }),
        },
        ToolDefinition {
            name: "hestia_lsp_diagnostics".to_string(),
            description: "Get HDL diagnostics from LSP broker".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "file_path": { "type": "string", "description": "HDL file path" }
                },
                "required": ["file_path"]
            }),
        },
        ToolDefinition {
            name: "hestia_constraint_convert".to_string(),
            description: "Convert constraint files between formats".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "from_format": { "type": "string" },
                    "to_format": { "type": "string" },
                    "content": { "type": "string" }
                },
                "required": ["from_format", "to_format", "content"]
            }),
        },
        ToolDefinition {
            name: "hestia_ip_resolve".to_string(),
            description: "Resolve IP core dependencies".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "ip_id": { "type": "string", "description": "IP core ID (e.g. com.xilinx.fifo)" }
                },
                "required": ["ip_id"]
            }),
        },
        ToolDefinition {
            name: "hestia_pipeline_run".to_string(),
            description: "Run a CI/CD pipeline".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "pipeline_name": { "type": "string" },
                    "backend": { "type": "string", "enum": ["github_actions", "gitlab_ci", "local"] }
                },
                "required": ["pipeline_name"]
            }),
        },
        ToolDefinition {
            name: "hestia_health_check".to_string(),
            description: "Check health status of Hestia components".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "conductor": { "type": "string", "description": "Conductor name (optional)" }
                }
            }),
        },
    ]
}