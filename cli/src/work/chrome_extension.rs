//! Native Messaging host and authenticated loopback bridge for the optional
//! Antigravity Chrome extension. Chrome owns the stdio side of the connection;
//! local MCP calls use a short-lived token and correlated JSON messages.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

const DEFAULT_BRIDGE_PORT: u16 = 4850;
const MAX_MESSAGE_BYTES: usize = 1024 * 1024;
const PROTOCOL_VERSION: &str = "1.0";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionCommand {
    pub target_extension_id: String,
    pub payload: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionMessage {
    pub from_extension_id: String,
    pub payload: Value,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BridgeState {
    protocol_version: String,
    port: u16,
    auth_token: String,
    pid: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LocalRequest {
    auth_token: String,
    request: Value,
}

fn bridge_state_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("agent-browser")
        .join("extension-bridge.json")
}

fn random_token() -> io::Result<String> {
    let mut bytes = [0_u8; 32];
    getrandom::getrandom(&mut bytes)
        .map_err(|error| io::Error::other(format!("failed to generate bridge token: {error}")))?;
    Ok(hex::encode(bytes))
}

fn read_native_message<R: Read>(input: &mut R) -> io::Result<Option<Value>> {
    let mut len_bytes = [0_u8; 4];
    match input.read_exact(&mut len_bytes) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(error) => return Err(error),
    }

    let len = u32::from_ne_bytes(len_bytes) as usize;
    if len == 0 || len > MAX_MESSAGE_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid native message size: {len}"),
        ));
    }

    let mut buffer = vec![0_u8; len];
    input.read_exact(&mut buffer)?;
    serde_json::from_slice(&buffer)
        .map(Some)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

fn write_native_message<W: Write>(output: &mut W, value: &Value) -> io::Result<()> {
    let bytes = serde_json::to_vec(value)?;
    if bytes.len() > MAX_MESSAGE_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "native message exceeds size limit",
        ));
    }
    output.write_all(&(bytes.len() as u32).to_ne_bytes())?;
    output.write_all(&bytes)?;
    output.flush()
}

fn write_json_line(stream: &mut TcpStream, value: &Value) -> io::Result<()> {
    let mut bytes = serde_json::to_vec(value)?;
    bytes.push(b'\n');
    stream.write_all(&bytes)?;
    stream.flush()
}

fn deliver_response(pending: &Arc<Mutex<HashMap<String, TcpStream>>>, message: &Value) {
    let Some(reference) = message.get("ref").and_then(Value::as_str) else {
        return;
    };
    if let Ok(mut pending) = pending.lock() {
        if let Some(mut stream) = pending.remove(reference) {
            let _ = write_json_line(&mut stream, message);
        }
    }
}

fn handle_local_client(
    stream: TcpStream,
    auth_token: String,
    extension_tx: mpsc::Sender<Value>,
    pending: Arc<Mutex<HashMap<String, TcpStream>>>,
) {
    let peer = stream.peer_addr().ok();
    if peer.is_some_and(|address| !address.ip().is_loopback()) {
        return;
    }

    let writer = match stream.try_clone() {
        Ok(writer) => writer,
        Err(_) => return,
    };
    let mut reader = BufReader::new(stream);
    let mut line = String::new();

    loop {
        line.clear();
        let read = match reader.read_line(&mut line) {
            Ok(read) => read,
            Err(_) => return,
        };
        if read == 0 || read > MAX_MESSAGE_BYTES {
            return;
        }

        let request = match serde_json::from_str::<LocalRequest>(line.trim()) {
            Ok(request) => request,
            Err(error) => {
                let mut writer = match writer.try_clone() {
                    Ok(writer) => writer,
                    Err(_) => return,
                };
                let _ = write_json_line(
                    &mut writer,
                    &json!({"isError": true, "error": {"code": "invalid_request", "message": error.to_string()}}),
                );
                continue;
            }
        };

        if request.auth_token != auth_token {
            let mut writer = match writer.try_clone() {
                Ok(writer) => writer,
                Err(_) => return,
            };
            let _ = write_json_line(
                &mut writer,
                &json!({"isError": true, "error": {"code": "unauthorized", "message": "invalid bridge token"}}),
            );
            return;
        }

        let Some(request_id) = request.request.get("id").and_then(Value::as_str) else {
            let mut writer = match writer.try_clone() {
                Ok(writer) => writer,
                Err(_) => return,
            };
            let _ = write_json_line(
                &mut writer,
                &json!({"isError": true, "error": {"code": "invalid_request", "message": "request.id is required"}}),
            );
            continue;
        };
        if let Ok(mut pending) = pending.lock() {
            if let Ok(client) = writer.try_clone() {
                pending.insert(request_id.to_string(), client);
            }
        }

        if extension_tx.send(request.request).is_err() {
            return;
        }
    }
}

fn start_local_bridge(
    extension_tx: mpsc::Sender<Value>,
    pending: Arc<Mutex<HashMap<String, TcpStream>>>,
) -> io::Result<(TcpListener, BridgeState)> {
    let listener = TcpListener::bind(("127.0.0.1", DEFAULT_BRIDGE_PORT))
        .or_else(|_| TcpListener::bind(("127.0.0.1", 0)))?;
    let state = BridgeState {
        protocol_version: PROTOCOL_VERSION.to_string(),
        port: listener.local_addr()?.port(),
        auth_token: random_token()?,
        pid: std::process::id(),
    };

    let state_path = bridge_state_path();
    if let Some(parent) = state_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&state_path, serde_json::to_vec_pretty(&state)?)?;

    let token = state.auth_token.clone();
    let listener_thread = listener.try_clone()?;
    thread::spawn(move || {
        for stream in listener_thread.incoming().flatten() {
            let tx = extension_tx.clone();
            let token = token.clone();
            let pending = Arc::clone(&pending);
            thread::spawn(move || handle_local_client(stream, token, tx, pending));
        }
    });

    Ok((listener, state))
}

struct BridgeStateGuard(PathBuf);

impl Drop for BridgeStateGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

pub fn run_native_host_loop() {
    let (extension_tx, extension_rx) = mpsc::channel::<Value>();
    let pending = Arc::new(Mutex::new(HashMap::<String, TcpStream>::new()));
    let (_listener, _state) = match start_local_bridge(extension_tx, Arc::clone(&pending)) {
        Ok(result) => result,
        Err(error) => {
            let _ = write_native_message(
                &mut io::stdout(),
                &json!({"isError": true, "error": {"code": "bridge_start_failed", "message": error.to_string()}}),
            );
            return;
        }
    };
    let _state_guard = BridgeStateGuard(bridge_state_path());

    thread::spawn(move || {
        let mut stdout = io::stdout();
        for message in extension_rx {
            if write_native_message(&mut stdout, &message).is_err() {
                break;
            }
        }
    });

    let mut stdin = io::stdin();
    loop {
        match read_native_message(&mut stdin) {
            Ok(Some(message)) => deliver_response(&pending, &message),
            Ok(None) | Err(_) => break,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_message_round_trip() {
        let expected = json!({"id": "1", "action": "observe"});
        let mut bytes = Vec::new();
        write_native_message(&mut bytes, &expected).unwrap();
        let actual = read_native_message(&mut bytes.as_slice()).unwrap().unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn native_message_rejects_oversized_payload() {
        let mut bytes = ((MAX_MESSAGE_BYTES as u32) + 1).to_ne_bytes().to_vec();
        bytes.extend_from_slice(b"{}");
        assert!(read_native_message(&mut bytes.as_slice()).is_err());
    }
}
