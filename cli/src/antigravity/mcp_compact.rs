//! Compact MCP facade for the Antigravity connector. Implemented operations
//! call the authenticated native bridge; unfinished operations fail honestly.

use serde::Deserialize;
use serde_json::{json, Value};
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::path::PathBuf;
use std::time::Duration;

const PROTOCOL_VERSION: &str = "1.0";
const BRIDGE_TIMEOUT: Duration = Duration::from_secs(20);
const MAX_RESPONSE_BYTES: usize = 1024 * 1024;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BridgeState {
    protocol_version: String,
    port: u16,
    auth_token: String,
}

fn response(is_error: bool, text: String, structured: Value) -> Value {
    json!({
        "isError": is_error,
        "content": [
            { "type": "text", "text": text }
        ],
        "structuredContent": structured
    })
}

fn error(code: &str, message: impl Into<String>) -> Value {
    let message = message.into();
    response(
        true,
        format!("{code}: {message}"),
        json!({ "error": { "code": code, "message": message } }),
    )
}

fn bridge_state_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("agent-browser")
        .join("extension-bridge.json")
}

fn call_bridge(request: Value) -> Result<Value, String> {
    let path = bridge_state_path();
    let bytes = fs::read(&path).map_err(|_| {
        "Chrome connector is not running. Install the native host and enable the extension."
            .to_string()
    })?;
    let state: BridgeState =
        serde_json::from_slice(&bytes).map_err(|error| format!("invalid bridge state: {error}"))?;
    if state.protocol_version != PROTOCOL_VERSION {
        return Err(format!(
            "unsupported bridge protocol {}, expected {}",
            state.protocol_version, PROTOCOL_VERSION
        ));
    }

    let address = ("127.0.0.1", state.port)
        .to_socket_addrs()
        .map_err(|error| error.to_string())?
        .next()
        .ok_or_else(|| "failed to resolve bridge address".to_string())?;
    let mut stream =
        TcpStream::connect_timeout(&address, BRIDGE_TIMEOUT).map_err(|error| error.to_string())?;
    stream
        .set_read_timeout(Some(BRIDGE_TIMEOUT))
        .map_err(|error| error.to_string())?;
    stream
        .set_write_timeout(Some(BRIDGE_TIMEOUT))
        .map_err(|error| error.to_string())?;

    let envelope = json!({
        "authToken": state.auth_token,
        "request": request
    });
    let mut payload = serde_json::to_vec(&envelope).map_err(|error| error.to_string())?;
    payload.push(b'\n');
    stream
        .write_all(&payload)
        .and_then(|_| stream.flush())
        .map_err(|error| error.to_string())?;

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    let read = reader
        .by_ref()
        .take((MAX_RESPONSE_BYTES + 1) as u64)
        .read_line(&mut line)
        .map_err(|error| error.to_string())?;
    if read == 0 {
        return Err("bridge closed before returning a response".to_string());
    }
    if read > MAX_RESPONSE_BYTES {
        return Err("bridge response exceeded the size limit".to_string());
    }
    serde_json::from_str(line.trim()).map_err(|error| format!("invalid bridge response: {error}"))
}

fn request_id(args: &Value) -> String {
    args.get("requestId")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string())
}

fn tab_id(args: &Value) -> Option<u64> {
    args.get("tabId").and_then(Value::as_u64)
}

fn bridge_result(result: Result<Value, String>) -> Value {
    match result {
        Ok(value) => {
            let is_error = value
                .get("isError")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            response(
                is_error,
                serde_json::to_string_pretty(&value).unwrap_or_else(|_| value.to_string()),
                value,
            )
        }
        Err(message) => error("bridge_unavailable", message),
    }
}

pub fn work_session_start(args: &Value) -> Value {
    bridge_result(call_bridge(json!({
        "protocolVersion": PROTOCOL_VERSION,
        "id": request_id(args),
        "action": "tabs.authorized"
    })))
}

pub fn work_observe(args: &Value) -> Value {
    bridge_result(call_bridge(json!({
        "protocolVersion": PROTOCOL_VERSION,
        "id": request_id(args),
        "action": "observe",
        "tabId": tab_id(args)
    })))
}

pub fn work_execute(args: &Value) -> Value {
    let action = match args.get("action").and_then(Value::as_str) {
        Some(action @ ("click" | "fill" | "type" | "focus" | "get_text")) => action,
        Some(action) => {
            return error(
                "unsupported_action",
                format!("action {action} is not supported by the connector"),
            )
        }
        None => return error("invalid_request", "action is required"),
    };

    let target = args.get("target").cloned().unwrap_or_else(|| json!({}));
    if !target.is_object() {
        return error("invalid_request", "target must be an object");
    }

    bridge_result(call_bridge(json!({
        "protocolVersion": PROTOCOL_VERSION,
        "id": request_id(args),
        "action": action,
        "tabId": tab_id(args),
        "target": target,
        "text": args.get("text")
    })))
}

pub fn work_verify(_args: &Value) -> Value {
    error(
        "not_implemented",
        "Generic postcondition verification is not implemented yet",
    )
}

pub fn work_checkpoint(_args: &Value) -> Value {
    error(
        "not_implemented",
        "Durable work checkpoints are not implemented yet",
    )
}

pub fn work_resume(_args: &Value) -> Value {
    error(
        "not_implemented",
        "Durable work resume is not implemented yet",
    )
}

pub fn work_export(_args: &Value) -> Value {
    error(
        "not_implemented",
        "Work artifact export is not implemented yet",
    )
}

pub fn work_request_approval(_args: &Value) -> Value {
    error(
        "not_implemented",
        "Bound approval receipts are not implemented yet",
    )
}

pub fn work_status(args: &Value) -> Value {
    work_session_start(args)
}

pub fn work_cancel(_args: &Value) -> Value {
    error(
        "not_implemented",
        "Work cancellation is not implemented yet",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsupported_action_is_an_error() {
        let result = work_execute(&json!({"action": "publish"}));
        assert_eq!(result["isError"], true);
        assert_eq!(
            result["structuredContent"]["error"]["code"],
            "unsupported_action"
        );
    }

    #[test]
    fn unimplemented_operations_do_not_claim_success() {
        for result in [
            work_verify(&json!({})),
            work_checkpoint(&json!({})),
            work_resume(&json!({})),
            work_export(&json!({})),
            work_request_approval(&json!({})),
            work_cancel(&json!({})),
        ] {
            assert_eq!(result["isError"], true);
        }
    }
}
