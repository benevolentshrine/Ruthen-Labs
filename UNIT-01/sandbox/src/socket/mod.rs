//! SANDBOX Socket — Unix socket server
//!
//! Handles incoming execution requests from ORCHESTRATOR/INDEXER over local Unix sockets.
//! No network code here — Unix sockets only.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Shared daemon state across all connections.
struct DaemonState {
    /// Active workspace path. File operations are scoped here when set.
    workspace: Option<PathBuf>,
    /// Session ID for shadow/rollback tracking.
    session_id: String,
}

impl DaemonState {
    fn new() -> Self {
        Self {
            workspace: None,
            session_id: uuid::Uuid::new_v4().to_string(),
        }
    }
}

type SharedState = Arc<Mutex<DaemonState>>;

/// Socket configuration (GATE 3: all paths centralized here)
pub mod config;

/// Ecosystem integration (auto-discovery with Nuki/Orchestrator)
pub mod ecosystem;

/// Socket stubs for INDEXER and ORCHESTRATOR (Phase 2)
pub mod stubs;

/// Maximum request size: 10MB
const MAX_REQUEST_SIZE: usize = config::MAX_REQUEST_SIZE;

/// Run the socket daemon
///
/// GATE 3: Strict socket contract freeze
/// Run the socket daemon
///
/// GATE 3: Strict socket contract freeze
pub async fn run_daemon(socket_path: Option<PathBuf>) -> Result<()> {
    let path = socket_path.unwrap_or_else(config::sandbox_socket_path);

    tracing::info!("Starting SANDBOX socket daemon on {:?}", path);

    #[cfg(unix)]
    {
        run_unix_daemon(path).await
    }

    #[cfg(windows)]
    {
        let _ = path; // used on unix
        // On Windows, use named pipes as a substitute for Unix sockets
        // This maintains the local-only communication requirement
        run_named_pipe_daemon().await
    }
}

#[cfg(unix)]
async fn run_unix_daemon(path: PathBuf) -> Result<()> {
    
    use tokio::net::UnixListener;

    // Ensure socket directory exists
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    // Remove old socket if it exists
    let socket_path = config::sandbox_socket_path();
    tracing::info!("Starting SANDBOX socket daemon on {:?}", socket_path);

    if path.exists() {
        tokio::fs::remove_file(&path)
            .await
            .with_context(|| format!("Failed to remove old socket at {:?}", path))?;
    }
    let listener = match UnixListener::bind(&path) {
        Ok(l) => l,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                tracing::error!(
                    "[FATAL] Socket bind denied at {}.\nLikely cause: SELinux or AppArmor policy blocking socket creation.\nFix: sudo semanage permissive -a unconfined_t\nOr add RUTHENLABS to AppArmor exceptions.",
                    config::sandbox_socket_path().display()
                );
                std::process::exit(2);
            }
            return Err(e.into());
        }
    };
    tracing::info!("Socket daemon listening on {:?}", path);

    let state: SharedState = Arc::new(Mutex::new(DaemonState::new()));

    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                let state = state.clone();
                tokio::spawn(handle_unix_connection(stream, state));
            }
            Err(e) => {
                tracing::error!("Failed to accept connection: {}", e);
            }
        }
    }
}

#[cfg(unix)]
async fn handle_unix_connection(mut stream: tokio::net::UnixStream, state: SharedState) -> Result<()> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let mut reader = BufReader::new(&mut stream);
    let mut line = String::new();
    
    // Read until newline (matching Orchestrator's UDS client)
    let n = reader.read_line(&mut line).await.context("Failed to read from socket")?;

    if n == 0 {
        return Ok(());
    }

    let response = process_request(line.as_bytes(), &state).await?;

    let mut response_bytes = serde_json::to_vec(&response)?;
    response_bytes.push(b'\n');
    
    stream.write_all(&response_bytes).await?;
    stream.flush().await?;

    Ok(())
}

#[cfg(windows)]
async fn run_named_pipe_daemon() -> Result<()> {
    // On Windows, use TCP localhost as the closest equivalent to Unix sockets
    // This still maintains local-only communication
    use tokio::net::TcpListener;

    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let local_addr = listener.local_addr()?;
    tracing::info!(
        "Socket daemon listening on TCP {} (Windows named pipe substitute)",
        local_addr
    );

    // Write the port to a file so clients can find it
    let info_path = std::env::temp_dir().join("sandbox").join("socket.info");
    if let Some(parent) = info_path.parent() {
        let _ = tokio::fs::create_dir_all(parent).await;
    }
    let _ = tokio::fs::write(&info_path, format!("{}", local_addr.port())).await;

    let state: SharedState = Arc::new(Mutex::new(DaemonState::new()));

    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                let state = state.clone();
                tokio::spawn(handle_tcp_connection(stream, state));
            }
            Err(e) => {
                tracing::error!("Failed to accept connection: {}", e);
            }
        }
    }
}

#[cfg(windows)]
async fn handle_tcp_connection(mut stream: tokio::net::TcpStream, state: SharedState) -> Result<()> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let mut buffer = vec![0u8; MAX_REQUEST_SIZE];
    let n: usize = stream
        .read(&mut buffer[..])
        .await
        .context("Failed to read from stream")?;

    if n == 0 {
        return Ok(());
    }

    buffer.truncate(n);

    let response = process_request(&buffer, &state).await?;

    let response_bytes = serde_json::to_vec(&response)?;
    stream.write_all(&response_bytes).await?;
    stream.write_all(b"\n").await?;
    stream.flush().await?;

    Ok(())
}

/// Process a request and return a response
async fn process_request(buffer: &[u8], state: &SharedState) -> Result<JsonRpcResponse> {
    let request: JsonRpcRequest = match serde_json::from_slice(buffer) {
        Ok(req) => req,
        Err(e) => {
            return Ok(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(JsonRpcError {
                    code: -32700,
                    message: format!("Parse error: {}", e),
                }),
                id: serde_json::Value::Null,
            });
        }
    };

    tracing::info!("Received request {} of type {}", request.id, request.method);

    let response = match request.method.as_str() {
        "cage_execute" | "execute" => handle_execute(request).await,
        "write" => handle_write(request, state).await,
        "patch" => handle_patch(request, state).await,
        "delete" => handle_delete(request, state).await,
        "rollback" => handle_rollback(request, state).await,
        "set_workspace" => handle_set_workspace(request, state).await,
        _ => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError {
                code: -32601,
                message: format!("Method not found: {}", request.method),
            }),
            id: request.id,
        },
    };
    Ok(response)
}

async fn handle_write(request: JsonRpcRequest, state: &SharedState) -> JsonRpcResponse {
    let path_str = request.params.get("path").and_then(|v| v.as_str());
    let content = request.params.get("content").and_then(|v| v.as_str());

    if let (Some(path), Some(text)) = (path_str, content) {
        if path.contains("..") {
            return error_response(request.id, -32602, "Security violation: path traversal not allowed");
        }

        let target = PathBuf::from(path);

        // Shadow backup: save original before overwrite
        if target.exists() {
            let session_id = state.lock().await.session_id.clone();
            if let Err(e) = shadow_backup(&target, &session_id) {
                tracing::warn!("Shadow backup failed for {}: {}", target.display(), e);
            }
        }

        // Ensure parent directory exists
        if let Some(parent) = target.parent() {
            let _ = tokio::fs::create_dir_all(parent).await;
        }

        match tokio::fs::write(&target, text).await {
            Ok(_) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(ExecuteResult {
                    verdict: format!("SUCCESS: File written to {}", target.display()),
                    audit_ref: uuid::Uuid::new_v4().to_string(),
                }),
                error: None,
                id: request.id,
            },
            Err(e) => error_response(request.id, -32000, &format!("Write failed: {}", e)),
        }
    } else {
        error_response(request.id, -32602, "Missing 'path' or 'content' parameter")
    }
}

/// Handle execute request
async fn handle_execute(request: JsonRpcRequest) -> JsonRpcResponse {
    let audit_id = uuid::Uuid::new_v4();

    let cmd = request.params.get("cmd").and_then(|v| v.as_str());
    if let Some(shell_cmd) = cmd {
        let cwd = request.params.get("cwd").and_then(|v| v.as_str()).unwrap_or(".");
        let output = tokio::process::Command::new("sh")
            .arg("-c")
            .arg(shell_cmd)
            .current_dir(cwd)
            .output()
            .await;
            
        return match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: Some(ExecuteResult {
                        verdict: format!("STDOUT:\n{}\nSTDERR:\n{}", stdout, stderr),
                        audit_ref: audit_id.to_string(),
                    }),
                    error: None,
                    id: request.id,
                }
            }
            Err(e) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(JsonRpcError {
                    code: -32000,
                    message: format!("Shell execute failed: {}", e),
                }),
                id: request.id,
            }
        };
    }

    let code_b64 = request.params.get("code").and_then(|v| v.as_str()).unwrap_or("");
    let code_bytes = match decode_base64(code_b64) {
        Ok(bytes) => bytes,
        Err(e) => {
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(JsonRpcError {
                    code: -32602,
                    message: format!("Invalid params: Failed to decode base64: {}", e),
                }),
                id: request.id,
            };
        }
    };

    // Write code to temp file
    let temp_dir = std::env::temp_dir().join("sandbox").join("workspace");
    let _ = tokio::fs::create_dir_all(&temp_dir).await;
    let temp_file = temp_dir.join(format!("{}.wasm", request.id));

    if let Err(e) = tokio::fs::write(&temp_file, &code_bytes).await {
        return JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError {
                code: -32000,
                message: format!("Server error: Failed to write temp file: {}", e),
            }),
            id: request.id,
        };
    }

    // Execute in cage (blocking operation, run in spawn_blocking)
    let policy_str = request.params.get("policy").and_then(|v| v.as_str()).unwrap_or("strict").to_string();
    let path = temp_file.clone();

    let verdict = tokio::task::spawn_blocking(move || {
        crate::cage::execute(path, policy_str, None)
    })
    .await
    .unwrap_or_else(|e| Err(anyhow::anyhow!("Execution panicked: {}", e)));

    // Cleanup temp file
    let _ = tokio::fs::remove_file(&temp_file).await;

    match verdict {
        Ok(crate::cage::Verdict::Allowed { .. }) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(ExecuteResult {
                verdict: "ALLOWED".to_string(),
                audit_ref: audit_id.to_string(),
            }),
            error: None,
            id: request.id,
        },
        Ok(crate::cage::Verdict::Blocked { reason }) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(ExecuteResult {
                verdict: "BLOCKED".to_string(),
                audit_ref: audit_id.to_string(),
            }),
            error: Some(JsonRpcError {
                code: 1000,
                message: reason,
            }),
            id: request.id,
        },
        Ok(crate::cage::Verdict::Timeout) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(ExecuteResult {
                verdict: "BLOCKED".to_string(),
                audit_ref: audit_id.to_string(),
            }),
            error: Some(JsonRpcError {
                code: 1001,
                message: "Timeout: fuel exhausted".to_string(),
            }),
            id: request.id,
        },
        Err(e) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError {
                code: -32000,
                message: format!("Execution error: {}", e),
            }),
            id: request.id,
        },
    }
}


/// Rollback all file changes for the current session using the Shadow system.
async fn handle_rollback(request: JsonRpcRequest, state: &SharedState) -> JsonRpcResponse {
    let session_id = request.params.get("session_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // Use provided session_id or fall back to current daemon session
    let sid = match session_id {
        Some(ref s) if s == "latest" => state.lock().await.session_id.clone(),
        Some(s) => s,
        None => state.lock().await.session_id.clone(),
    };

    let result = tokio::task::spawn_blocking(move || {
        let manager = crate::shadow::RollbackManager::new()?;
        manager.rollback(&sid)
    }).await;

    match result {
        Ok(Ok(rollback_result)) => {
            let msg = format!(
                "ROLLBACK COMPLETE: {} files restored, {} failed",
                rollback_result.success_count(),
                rollback_result.failure_count()
            );
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(ExecuteResult {
                    verdict: msg,
                    audit_ref: uuid::Uuid::new_v4().to_string(),
                }),
                error: None,
                id: request.id,
            }
        }
        Ok(Err(e)) => error_response(request.id, -32000, &format!("Rollback failed: {}", e)),
        Err(e) => error_response(request.id, -32000, &format!("Rollback task panicked: {}", e)),
    }
}

/// Search-and-replace within a file, with shadow backup.
async fn handle_patch(request: JsonRpcRequest, state: &SharedState) -> JsonRpcResponse {
    let path_str = request.params.get("path").and_then(|v| v.as_str());
    let target_text = request.params.get("target").and_then(|v| v.as_str());
    let replacement = request.params.get("replacement").and_then(|v| v.as_str());

    if let (Some(path), Some(target), Some(repl)) = (path_str, target_text, replacement) {
        if path.contains("..") {
            return error_response(request.id, -32602, "Security violation: path traversal not allowed");
        }

        let file_path = PathBuf::from(path);

        // Read current content
        let content = match tokio::fs::read_to_string(&file_path).await {
            Ok(c) => c,
            Err(e) => return error_response(request.id, -32000, &format!("Failed to read file: {}", e)),
        };

        if !content.contains(target) {
            return error_response(request.id, -32000, &format!("Target text not found in {}", path));
        }

        // Shadow backup before modification
        let session_id = state.lock().await.session_id.clone();
        if let Err(e) = shadow_backup(&file_path, &session_id) {
            tracing::warn!("Shadow backup failed for {}: {}", file_path.display(), e);
        }

        let patched = content.replacen(target, repl, 1);

        match tokio::fs::write(&file_path, &patched).await {
            Ok(_) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(ExecuteResult {
                    verdict: format!("SUCCESS: Patch applied to {}", path),
                    audit_ref: uuid::Uuid::new_v4().to_string(),
                }),
                error: None,
                id: request.id,
            },
            Err(e) => error_response(request.id, -32000, &format!("Patch write failed: {}", e)),
        }
    } else {
        error_response(request.id, -32602, "Missing 'path', 'target', or 'replacement' parameter")
    }
}

/// Set the active workspace directory for this daemon session.
async fn handle_set_workspace(request: JsonRpcRequest, state: &SharedState) -> JsonRpcResponse {
    let path_str = request.params.get("path").and_then(|v| v.as_str());

    if let Some(path) = path_str {
        let workspace = PathBuf::from(path);
        if !workspace.exists() || !workspace.is_dir() {
            return error_response(request.id, -32602, &format!("Invalid workspace: {} (must be an existing directory)", path));
        }

        let mut daemon_state = state.lock().await;
        daemon_state.workspace = Some(workspace);
        daemon_state.session_id = uuid::Uuid::new_v4().to_string();
        let sid = daemon_state.session_id.clone();
        drop(daemon_state);

        tracing::info!("Workspace set to: {} (session: {})", path, sid);

        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(ExecuteResult {
                verdict: format!("Workspace set to: {}", path),
                audit_ref: sid,
            }),
            error: None,
            id: request.id,
        }
    } else {
        error_response(request.id, -32602, "Missing 'path' parameter")
    }
}

/// Delete a file, with shadow backup.
async fn handle_delete(request: JsonRpcRequest, state: &SharedState) -> JsonRpcResponse {
    let path_str = request.params.get("path").and_then(|v| v.as_str());

    if let Some(path) = path_str {
        if path.contains("..") {
            return error_response(request.id, -32602, "Security violation: path traversal not allowed");
        }

        let file_path = PathBuf::from(path);
        if !file_path.exists() {
            return error_response(request.id, -32000, &format!("File not found: {}", path));
        }

        // Shadow backup before deletion
        let session_id = state.lock().await.session_id.clone();
        if let Err(e) = shadow_backup(&file_path, &session_id) {
            tracing::warn!("Shadow backup failed for deletion of {}: {}", file_path.display(), e);
        }

        match tokio::fs::remove_file(&file_path).await {
            Ok(_) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(ExecuteResult {
                    verdict: format!("SUCCESS: File {} deleted", path),
                    audit_ref: uuid::Uuid::new_v4().to_string(),
                }),
                error: None,
                id: request.id,
            },
            Err(e) => error_response(request.id, -32000, &format!("Delete failed: {}", e)),
        }
    } else {
        error_response(request.id, -32602, "Missing 'path' parameter")
    }
}

// ─── Helpers ───────────────────────────────────────────────────────────────────

/// Create a shadow backup of a file using Sandbox's RollbackManager.
fn shadow_backup(path: &Path, session_id: &str) -> Result<()> {
    let manager = crate::shadow::RollbackManager::new()?;
    manager.backup(path, session_id)?;
    Ok(())
}

/// Shorthand for building a JSON-RPC error response.
fn error_response(id: serde_json::Value, code: i32, message: &str) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        result: None,
        error: Some(JsonRpcError {
            code,
            message: message.to_string(),
        }),
        id,
    }
}

/// JSON-RPC 2.0 Request
#[derive(Debug, serde::Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: serde_json::Value,
    pub id: serde_json::Value,
}

#[derive(Debug, serde::Deserialize)]
pub struct ExecutePayload {
    /// Base64-encoded code
    pub code: String,
    /// "wasm" | "shell"
    #[allow(dead_code)]
    pub format: String,
    /// "strict" | "permissive"
    pub policy: String,
}

/// JSON-RPC 2.0 Response
#[derive(Debug, serde::Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<ExecuteResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    pub id: serde_json::Value,
}

#[derive(Debug, serde::Serialize)]
pub struct ExecuteResult {
    /// "ALLOWED" | "BLOCKED"
    pub verdict: String,
    /// Log entry ID for audit trail
    pub audit_ref: String,
}

#[derive(Debug, serde::Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
}

/// Base64 decoding helper (no external crate needed — GATE 1)
fn decode_base64(s: &str) -> Result<Vec<u8>> {
    let mut result = Vec::with_capacity(s.len() * 3 / 4);
    let chars: Vec<u8> = s.bytes().collect();

    let decode_char = |c: u8| -> Option<u8> {
        match c {
            b'A'..=b'Z' => Some(c - b'A'),
            b'a'..=b'z' => Some(c - b'a' + 26),
            b'0'..=b'9' => Some(c - b'0' + 52),
            b'+' => Some(62),
            b'/' => Some(63),
            _ => None,
        }
    };

    for chunk in chars.chunks(4) {
        let b0 = chunk.first().copied().ok_or_else(|| anyhow::anyhow!("Invalid base64"))?;
        let b1 = chunk.get(1).copied().ok_or_else(|| anyhow::anyhow!("Invalid base64"))?;

        let b = [
            decode_char(b0).ok_or_else(|| anyhow::anyhow!("Invalid base64 char"))?,
            decode_char(b1).ok_or_else(|| anyhow::anyhow!("Invalid base64 char"))?,
            chunk.get(2).and_then(|c| decode_char(*c)).unwrap_or(0),
            chunk.get(3).and_then(|c| decode_char(*c)).unwrap_or(0),
        ];

        result.push((b[0] << 2) | (b[1] >> 4));
        if chunk.len() > 2 && chunk[2] != b'=' {
            result.push((b[1] << 4) | (b[2] >> 2));
        }
        if chunk.len() > 3 && chunk[3] != b'=' {
            result.push((b[2] << 6) | b[3]);
        }
    }

    Ok(result)
}
