//! BORU Socket — Unix socket server
//!
//! Handles incoming execution requests from SUJI/YOMI over local Unix sockets.
//! No network code here — Unix sockets only.

use anyhow::{Context, Result};
use std::path::PathBuf;

/// Socket configuration (GATE 3: all paths centralized here)
pub mod config;

/// Ecosystem integration (auto-discovery with Nuki/Suji)
pub mod ecosystem;

/// Socket stubs for YOMI and SUJI (Phase 2)
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
    let path = socket_path.unwrap_or_else(config::boru_socket_path);

    tracing::info!("Starting BORU socket daemon on {:?}", path);

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
    let socket_path = config::boru_socket_path();
    tracing::info!("Starting BORU socket daemon on {:?}", socket_path);

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
                    "[FATAL] Socket bind denied at {}.\nLikely cause: SELinux or AppArmor policy blocking socket creation.\nFix: sudo semanage permissive -a unconfined_t\nOr add MOMO to AppArmor exceptions.",
                    config::boru_socket_path().display()
                );
                std::process::exit(2);
            }
            return Err(e.into());
        }
    };
    tracing::info!("Socket daemon listening on {:?}", path);

    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                tokio::spawn(handle_unix_connection(stream));
            }
            Err(e) => {
                tracing::error!("Failed to accept connection: {}", e);
            }
        }
    }
}

#[cfg(unix)]
async fn handle_unix_connection(mut stream: tokio::net::UnixStream) -> Result<()> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    // Read request (JSON NDJSON style - newline delimited)
    let mut buffer = vec![0u8; MAX_REQUEST_SIZE];
    let n: usize = stream
        .read(&mut buffer[..])
        .await
        .context("Failed to read from socket")?;

    if n == 0 {
        return Ok(());
    }

    buffer.truncate(n);

    let response = process_request(&buffer).await?;

    // Send response
    let response_bytes = serde_json::to_vec(&response)?;
    stream.write_all(&response_bytes).await?;
    stream.write_all(b"\n").await?;
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
    let info_path = std::env::temp_dir().join("boru").join("socket.info");
    if let Some(parent) = info_path.parent() {
        let _ = tokio::fs::create_dir_all(parent).await;
    }
    let _ = tokio::fs::write(&info_path, format!("{}", local_addr.port())).await;

    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                tokio::spawn(handle_tcp_connection(stream));
            }
            Err(e) => {
                tracing::error!("Failed to accept connection: {}", e);
            }
        }
    }
}

#[cfg(windows)]
async fn handle_tcp_connection(mut stream: tokio::net::TcpStream) -> Result<()> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    // Read request
    let mut buffer = vec![0u8; MAX_REQUEST_SIZE];
    let n: usize = stream
        .read(&mut buffer[..])
        .await
        .context("Failed to read from stream")?;

    if n == 0 {
        return Ok(());
    }

    buffer.truncate(n);

    let response = process_request(&buffer).await?;

    // Send response
    let response_bytes = serde_json::to_vec(&response)?;
    stream.write_all(&response_bytes).await?;
    stream.write_all(b"\n").await?;
    stream.flush().await?;

    Ok(())
}

/// Process a request and return a response
async fn process_request(buffer: &[u8]) -> Result<JsonRpcResponse> {
    // Parse request
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
                id: "null".to_string(),
            });
        }
    };

    tracing::info!(
        "Received request {} of type {}",
        request.id,
        request.method
    );

    // Process based on request type
    let response = match request.method.as_str() {
        "cage_execute" => handle_execute(request).await,
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

/// Handle execute request
async fn handle_execute(request: JsonRpcRequest) -> JsonRpcResponse {
    let audit_id = uuid::Uuid::new_v4();

    // Decode base64 code
    let code_bytes = match decode_base64(&request.params.code) {
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
    let temp_dir = std::env::temp_dir().join("boru").join("workspace");
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
    let policy = request.params.policy.clone();
    let path = temp_file.clone();

    let verdict = tokio::task::spawn_blocking(move || {
        crate::cage::execute(path, policy, None)
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

/// JSON-RPC 2.0 Request
#[derive(Debug, serde::Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: ExecutePayload,
    pub id: String,
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
    pub id: String,
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
