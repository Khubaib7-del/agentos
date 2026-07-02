//! Minimal MCP (Model Context Protocol) server over stdio: newline-delimited
//! JSON-RPC 2.0. Hand-rolled instead of an SDK to keep the dependency tree
//! tiny (security finding 4) and the input handling fully under our control
//! (finding 6): strict shapes, no shell, tool output is data only.

use agentos_core::Store;
use anyhow::Result;
use serde_json::{json, Value};
use std::io::{BufRead, Write};
use std::path::Path;

type RpcResult = std::result::Result<Value, (i64, String)>;

pub fn serve(root: &Path) -> Result<()> {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        let Ok(msg) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        // Requests without an id are notifications — never answered.
        let Some(id) = msg.get("id").cloned() else {
            continue;
        };
        let method = msg.get("method").and_then(Value::as_str).unwrap_or("");
        let params = msg.get("params").cloned().unwrap_or_else(|| json!({}));
        let payload = match dispatch(root, method, &params) {
            Ok(result) => json!({ "jsonrpc": "2.0", "id": id, "result": result }),
            Err((code, message)) => json!({
                "jsonrpc": "2.0", "id": id,
                "error": { "code": code, "message": message }
            }),
        };
        writeln!(out, "{payload}")?;
        out.flush()?;
    }
    Ok(())
}

fn dispatch(root: &Path, method: &str, params: &Value) -> RpcResult {
    match method {
        "initialize" => Ok(initialize_result(params)),
        "ping" => Ok(json!({})),
        "tools/list" => Ok(tools_list()),
        "tools/call" => Ok(tools_call(root, params)),
        _ => Err((-32601, format!("method not found: {method}"))),
    }
}

fn initialize_result(params: &Value) -> Value {
    let version = params
        .get("protocolVersion")
        .and_then(Value::as_str)
        .unwrap_or("2025-06-18");
    json!({
        "protocolVersion": version,
        "capabilities": { "tools": {} },
        "serverInfo": { "name": "agentos", "version": env!("CARGO_PKG_VERSION") }
    })
}

fn tools_list() -> Value {
    json!({ "tools": [
        {
            "name": "get_decisions",
            "description": "Read the project's recorded decisions. Locked decisions are commitments: if a plan conflicts with one, surface the conflict to the user instead of silently deviating.",
            "inputSchema": { "type": "object", "properties": {}, "additionalProperties": false }
        },
        {
            "name": "log_decision",
            "description": "Record a project decision the user has made or approved. Use lock=true only when the user explicitly confirms it is final.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "text": { "type": "string", "description": "The decision, e.g. 'DB: PostgreSQL'" },
                    "why": { "type": "string", "description": "Rationale" },
                    "lock": { "type": "boolean", "description": "Mark as locked (user-confirmed final)" }
                },
                "required": ["text"],
                "additionalProperties": false
            }
        },
        {
            "name": "get_review_queue",
            "description": "Read the user's queued review notes — ideas they had while you were working. Address pending notes like code-review comments when you finish a task.",
            "inputSchema": { "type": "object", "properties": {}, "additionalProperties": false }
        },
        {
            "name": "resolve_review_note",
            "description": "Mark a review note as resolved after you have addressed it (or explained why not).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": { "type": "integer", "description": "The note id from get_review_queue" }
                },
                "required": ["id"],
                "additionalProperties": false
            }
        }
    ] })
}

fn tools_call(root: &Path, params: &Value) -> Value {
    let name = params.get("name").and_then(Value::as_str).unwrap_or("");
    let args = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));
    match run_tool(root, name, &args) {
        Ok(text) => json!({ "content": [{ "type": "text", "text": text }], "isError": false }),
        Err(text) => json!({ "content": [{ "type": "text", "text": text }], "isError": true }),
    }
}

fn run_tool(root: &Path, name: &str, args: &Value) -> std::result::Result<String, String> {
    let store = Store::open(root).map_err(|_| {
        "agentos is not initialized in this project — run `agentos init` first".to_string()
    })?;
    match name {
        "get_decisions" => {
            let decisions = store.decisions().map_err(|e| e.to_string())?;
            if decisions.is_empty() {
                return Ok("no decisions recorded yet".into());
            }
            let mut s = String::new();
            for d in &decisions {
                let lock = if d.locked { " [locked]" } else { "" };
                s.push_str(&format!("#{} {}{}", d.id, d.text, lock));
                if let Some(why) = &d.why {
                    s.push_str(&format!(" — why: {why}"));
                }
                s.push('\n');
            }
            Ok(s)
        }
        "log_decision" => {
            let text = args
                .get("text")
                .and_then(Value::as_str)
                .filter(|t| !t.trim().is_empty())
                .ok_or("missing required argument: text")?;
            let why = args.get("why").and_then(Value::as_str);
            let lock = args.get("lock").and_then(Value::as_bool).unwrap_or(false);
            let d = store
                .add_decision(text, why, lock)
                .map_err(|e| e.to_string())?;
            let suffix = if d.locked { " (locked)" } else { "" };
            Ok(format!("decision #{} recorded{suffix}", d.id))
        }
        "get_review_queue" => {
            let notes = store.notes().map_err(|e| e.to_string())?;
            let open: Vec<_> = notes
                .iter()
                .filter(|n| n.status != agentos_core::NoteStatus::Resolved)
                .collect();
            if open.is_empty() {
                return Ok("review queue is empty".into());
            }
            let mut s = String::new();
            for n in open {
                let status = match n.status {
                    agentos_core::NoteStatus::Pending => "pending",
                    agentos_core::NoteStatus::Delivered => "delivered",
                    agentos_core::NoteStatus::Resolved => unreachable!(),
                };
                s.push_str(&format!("#{} [{}] {}\n", n.id, status, n.text));
            }
            Ok(s)
        }
        "resolve_review_note" => {
            let id = args
                .get("id")
                .and_then(Value::as_u64)
                .ok_or("missing required argument: id")?;
            if store.resolve_note(id).map_err(|e| e.to_string())? {
                Ok(format!("note #{id} resolved"))
            } else {
                Err(format!("no note with id {id}"))
            }
        }
        other => Err(format!("unknown tool: {other}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialize_echoes_protocol_version() {
        let r = initialize_result(&json!({ "protocolVersion": "2025-03-26" }));
        assert_eq!(r["protocolVersion"], "2025-03-26");
        assert_eq!(r["serverInfo"]["name"], "agentos");
    }

    #[test]
    fn tools_list_has_four_tools() {
        let tools = tools_list();
        assert_eq!(tools["tools"].as_array().unwrap().len(), 4);
    }

    #[test]
    fn unknown_method_is_rpc_error() {
        let dir = tempfile::tempdir().unwrap();
        let err = dispatch(dir.path(), "resources/list", &json!({})).unwrap_err();
        assert_eq!(err.0, -32601);
    }

    #[test]
    fn tool_call_lifecycle_against_real_store() {
        let dir = tempfile::tempdir().unwrap();
        Store::init(dir.path()).unwrap();

        let r = tools_call(
            dir.path(),
            &json!({ "name": "log_decision", "arguments": { "text": "DB: PostgreSQL", "lock": true } }),
        );
        assert_eq!(r["isError"], false);

        let r = tools_call(dir.path(), &json!({ "name": "get_decisions" }));
        let text = r["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("DB: PostgreSQL"));
        assert!(text.contains("[locked]"));

        let store = Store::open(dir.path()).unwrap();
        let n = store.add_note("try caching").unwrap();
        let r = tools_call(
            dir.path(),
            &json!({ "name": "resolve_review_note", "arguments": { "id": n.id } }),
        );
        assert_eq!(r["isError"], false);
        assert!(store.pending_notes().unwrap().is_empty());
    }

    #[test]
    fn uninitialized_project_is_tool_error_not_crash() {
        let dir = tempfile::tempdir().unwrap();
        let r = tools_call(dir.path(), &json!({ "name": "get_decisions" }));
        assert_eq!(r["isError"], true);
        assert!(r["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("agentos init"));
    }
}
