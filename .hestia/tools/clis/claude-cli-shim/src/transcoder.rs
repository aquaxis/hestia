//! Conversion from Claude Code stream-json events to agent-cli compatible JSONL.
//!
//! Claude Code's `--output-format stream-json` outputs in the following format:
//! ```jsonl
//! {"type":"system","subtype":"init",...}
//! {"type":"assistant","message":{"content":[{"type":"thinking","thinking":"..."}]}}
//! {"type":"assistant","message":{"content":[{"type":"text","text":"..."}]}}
//! {"type":"assistant","message":{"content":[{"type":"tool_use","name":"...","input":{...}}]}}
//! {"type":"user","message":{"content":[{"type":"tool_result","content":"...","is_error":false}]}}
//! {"type":"result",...}
//! ```
//!
//! `transcode_event`, which converts to the agent-cli format, returns 0..N agent-cli
//! compatible events per input line (because an assistant message may contain
//! both thinking + tool_use blocks).

use serde_json::{json, Value};

/// Convert a single Claude stream-json line into a sequence of agent-cli compatible events.
///
/// Unrecognized event types are ignored (returns an empty Vec).
pub fn transcode_event(claude_event: &Value) -> Vec<Value> {
    let ts = chrono::Utc::now().to_rfc3339();
    let event_type = claude_event.get("type").and_then(|v| v.as_str()).unwrap_or("");
    match event_type {
        "assistant" => transcode_assistant(claude_event, &ts),
        "user" => transcode_user(claude_event, &ts),
        // system / result / error have no agent-cli mapping (ignored)
        _ => Vec::new(),
    }
}

fn transcode_assistant(ev: &Value, ts: &str) -> Vec<Value> {
    let Some(content) = ev
        .get("message")
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_array())
    else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for block in content {
        let block_type = block.get("type").and_then(|v| v.as_str()).unwrap_or("");
        match block_type {
            "thinking" => {
                let text = block
                    .get("thinking")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                out.push(json!({
                    "kind": "thinking",
                    "ts": ts,
                    "text": text,
                }));
            }
            "text" => {
                let text = block.get("text").and_then(|v| v.as_str()).unwrap_or("");
                out.push(json!({
                    "kind": "assistant",
                    "ts": ts,
                    "text": text,
                }));
            }
            "tool_use" => {
                let tool = block.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let args = block.get("input").cloned().unwrap_or(Value::Null);
                out.push(json!({
                    "kind": "tool_call",
                    "ts": ts,
                    "tool": tool,
                    "args": args,
                }));
            }
            _ => {}
        }
    }
    out
}

fn transcode_user(ev: &Value, ts: &str) -> Vec<Value> {
    let Some(content) = ev
        .get("message")
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_array())
    else {
        // When the user message is just a string
        if let Some(text) = ev.get("message").and_then(|v| v.as_str()) {
            return vec![json!({
                "kind": "user",
                "ts": ts,
                "text": text,
            })];
        }
        return Vec::new();
    };
    let mut out = Vec::new();
    for block in content {
        let block_type = block.get("type").and_then(|v| v.as_str()).unwrap_or("");
        match block_type {
            "tool_result" => {
                let tool = block
                    .get("tool_use_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let is_error = block
                    .get("is_error")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let result = block
                    .get("content")
                    .cloned()
                    .unwrap_or(Value::Null);
                out.push(json!({
                    "kind": "tool_result",
                    "ts": ts,
                    "tool": tool,
                    "ok": !is_error,
                    "result": result,
                }));
            }
            "text" => {
                let text = block.get("text").and_then(|v| v.as_str()).unwrap_or("");
                out.push(json!({
                    "kind": "user",
                    "ts": ts,
                    "text": text,
                }));
            }
            _ => {}
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn assistant_thinking_becomes_thinking_kind() {
        let ev = json!({
            "type": "assistant",
            "message": {
                "content": [
                    {"type": "thinking", "thinking": "Thinking..."},
                ]
            }
        });
        let out = transcode_event(&ev);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0]["kind"], "thinking");
        assert_eq!(out[0]["text"], "Thinking...");
    }

    #[test]
    fn assistant_text_becomes_assistant_kind() {
        let ev = json!({
            "type": "assistant",
            "message": {
                "content": [
                    {"type": "text", "text": "Here is the answer"},
                ]
            }
        });
        let out = transcode_event(&ev);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0]["kind"], "assistant");
        assert_eq!(out[0]["text"], "Here is the answer");
    }

    #[test]
    fn assistant_tool_use_becomes_tool_call() {
        let ev = json!({
            "type": "assistant",
            "message": {
                "content": [
                    {"type": "tool_use", "name": "Read", "input": {"path": "/tmp/x"}},
                ]
            }
        });
        let out = transcode_event(&ev);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0]["kind"], "tool_call");
        assert_eq!(out[0]["tool"], "Read");
        assert_eq!(out[0]["args"]["path"], "/tmp/x");
    }

    #[test]
    fn user_tool_result_ok_when_not_error() {
        let ev = json!({
            "type": "user",
            "message": {
                "content": [
                    {"type": "tool_result", "tool_use_id": "Read", "content": "data", "is_error": false},
                ]
            }
        });
        let out = transcode_event(&ev);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0]["kind"], "tool_result");
        assert_eq!(out[0]["ok"], true);
    }

    #[test]
    fn user_tool_result_ok_false_when_error() {
        let ev = json!({
            "type": "user",
            "message": {
                "content": [
                    {"type": "tool_result", "tool_use_id": "Read", "content": "fail", "is_error": true},
                ]
            }
        });
        let out = transcode_event(&ev);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0]["ok"], false);
    }

    #[test]
    fn unknown_event_type_returns_empty() {
        let ev = json!({"type": "system", "subtype": "init"});
        let out = transcode_event(&ev);
        assert!(out.is_empty());
    }

    #[test]
    fn assistant_with_multiple_blocks() {
        let ev = json!({
            "type": "assistant",
            "message": {
                "content": [
                    {"type": "thinking", "thinking": "Thinking"},
                    {"type": "text", "text": "Answering"},
                    {"type": "tool_use", "name": "Bash", "input": {"cmd": "ls"}},
                ]
            }
        });
        let out = transcode_event(&ev);
        assert_eq!(out.len(), 3);
        assert_eq!(out[0]["kind"], "thinking");
        assert_eq!(out[1]["kind"], "assistant");
        assert_eq!(out[2]["kind"], "tool_call");
    }
}