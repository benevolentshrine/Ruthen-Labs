use crate::DaemonAction;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};
use tracing::{error, info};

#[derive(Serialize, Deserialize, Debug)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: serde_json::Value,
    id: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonRpcResponse {
    jsonrpc: String,
    result: Option<serde_json::Value>,
    error: Option<serde_json::Value>,
    id: u64,
}

// ── Directory & file paths ────────────────────────────────────────────────────

fn get_yomi_dir() -> PathBuf {
    if let Ok(env_dir) = std::env::var("YOMI_DATA_DIR") {
        let dir = PathBuf::from(env_dir);
        if !dir.exists() {
            let _ = std::fs::create_dir_all(&dir);
        }
        return dir;
    }
    if let Some(proj_dirs) = ProjectDirs::from("com", "sumi", "yomi") {
        let data_dir = proj_dirs.data_dir();
        if !data_dir.exists() {
            let _ = std::fs::create_dir_all(data_dir);
        }
        data_dir.to_path_buf()
    } else {
        PathBuf::from(".")
    }
}

fn get_port_file() -> PathBuf  { get_yomi_dir().join("port")       }
fn get_token_file() -> PathBuf { get_yomi_dir().join("auth_token") }
fn get_pid_file() -> PathBuf   { get_yomi_dir().join("daemon.pid") }
fn get_log_file() -> PathBuf   { get_yomi_dir().join("daemon.log") }

// ── PID helpers ───────────────────────────────────────────────────────────────

/// Write the current process PID to the PID file.
fn write_pid_file() -> std::io::Result<()> {
    let pid = std::process::id();
    let mut f = File::create(get_pid_file())?;
    write!(f, "{}", pid)?;
    Ok(())
}

/// Read the PID stored in the PID file. Returns None if file is absent or unparseable.
fn read_pid_file() -> Option<u32> {
    let mut f = File::open(get_pid_file()).ok()?;
    let mut s = String::new();
    f.read_to_string(&mut s).ok()?;
    s.trim().parse().ok()
}

/// Check whether a process with the given PID is still alive.
/// On Unix we send signal 0; on Windows we use OpenProcess.
#[cfg(unix)]
fn pid_is_alive(pid: u32) -> bool {
    use nix::sys::signal::{kill, Signal};
    use nix::unistd::Pid;
    kill(Pid::from_raw(pid as i32), Signal::SIGUSR1).is_ok()
}

#[cfg(not(unix))]
fn pid_is_alive(pid: u32) -> bool {
    // On Windows, try to open the process; if it succeeds it is still alive.
    use std::process::Command;
    Command::new("tasklist")
        .args(["/FI", &format!("PID eq {}", pid), "/NH"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains(&pid.to_string()))
        .unwrap_or(false)
}

/// Remove all runtime state files (port, token, pid).
fn cleanup_state_files() {
    let _ = std::fs::remove_file(get_port_file());
    let _ = std::fs::remove_file(get_token_file());
    let _ = std::fs::remove_file(get_pid_file());
}

// ── CLI action entry-point ────────────────────────────────────────────────────

pub async fn handle_daemon_action(action: &DaemonAction) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        DaemonAction::Start => {
            // ── Internal run mode (already detached child) ──────────────────
            if std::env::var("YOMI_DAEMON_INTERNAL").is_ok() {
                run_daemon_server().await?;
                return Ok(());
            }

            // ── Check for a running daemon (stale-state-aware) ──────────────
            if let Some(pid) = read_pid_file() {
                if pid_is_alive(pid) {
                    info!("Daemon is already running (PID {}).", pid);
                    return Ok(());
                }
                info!("Stale PID file found (PID {} is dead). Cleaning up and restarting.", pid);
                cleanup_state_files();
            }

            // ── Spawn detached background process ───────────────────────────
            let exe = std::env::current_exe()?;
            let log_path = get_log_file();
            let log_file = File::create(&log_path)?;

            #[cfg(unix)]
            {
                use std::os::unix::process::CommandExt;
                let _child = std::process::Command::new(&exe)
                    .args(["daemon", "start"])
                    .env("YOMI_DAEMON_INTERNAL", "1")
                    .stdin(std::process::Stdio::null())
                    .stdout(log_file.try_clone()?)
                    .stderr(log_file)
                    // setsid() detaches from the controlling TTY
                    .process_group(0)
                    .spawn()?;
            }
            #[cfg(not(unix))]
            {
                // Windows: CREATE_NO_WINDOW flag via `creation_flags`
                use std::os::windows::process::CommandExt;
                const CREATE_NO_WINDOW: u32 = 0x08000000;
                let _child = std::process::Command::new(&exe)
                    .args(["daemon", "start"])
                    .env("YOMI_DAEMON_INTERNAL", "1")
                    .stdin(std::process::Stdio::null())
                    .stdout(log_file.try_clone()?)
                    .stderr(log_file)
                    .creation_flags(CREATE_NO_WINDOW)
                    .spawn()?;
            }

            info!("Daemon launched in background. Logs → {:?}", log_path);
        }

        DaemonAction::Status => {
            // ── Stale-state detection before any TCP call ───────────────────
            match read_pid_file() {
                None => {
                    info!("Daemon is not running (no PID file).");
                    return Ok(());
                }
                Some(pid) if !pid_is_alive(pid) => {
                    info!("Daemon is not running (PID {} is dead). Cleaning up stale files.", pid);
                    cleanup_state_files();
                    return Ok(());
                }
                Some(pid) => {
                    info!("PID {} appears alive; querying via TCP.", pid);
                }
            }
            match send_rpc("status", serde_json::json!({})).await {
                Ok(res) => info!("Daemon status: {:?}", res),
                Err(e)  => info!("Daemon unreachable: {}", e),
            }
        }

        DaemonAction::Stop => {
            let res = send_rpc("stop", serde_json::json!({})).await?;
            info!("Daemon stop response: {:?}", res);
        }
    }
    Ok(())
}

// ── Server loop ───────────────────────────────────────────────────────────────

async fn run_daemon_server() -> Result<(), Box<dyn std::error::Error>> {
    let socket_path = "/tmp/sumi/yomi.sock";
    
    // Ensure parent directory exists
    if let Some(parent) = std::path::Path::new(socket_path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    // Remove existing socket if it exists
    let _ = std::fs::remove_file(socket_path);

    let listener = UnixListener::bind(socket_path)?;
    info!("Yomi daemon listening on UDS: {} (PID {})", socket_path, std::process::id());

    write_pid_file()?;

    // Token is no longer used for UDS as socket permissions handle security
    let token = "uds-internal-trust".to_string();

    // ── Graceful SIGTERM / SIGINT via a shutdown channel ───────────────────
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    let shutdown_tx = std::sync::Arc::new(std::sync::Mutex::new(Some(shutdown_tx)));

    #[cfg(unix)]
    {
        let tx_clone = shutdown_tx.clone();
        tokio::spawn(async move {
            use tokio::signal::unix::{signal, SignalKind};
            let mut sigterm = signal(SignalKind::terminate()).unwrap();
            let mut sigint  = signal(SignalKind::interrupt()).unwrap();
            tokio::select! {
                _ = sigterm.recv() => info!("Received SIGTERM"),
                _ = sigint.recv()  => info!("Received SIGINT"),
            }
            if let Ok(mut lock) = tx_clone.lock() {
                if let Some(tx) = lock.take() {
                    let _ = tx.send(());
                }
            }
        });
    }
    #[cfg(not(unix))]
    {
        let tx_clone = shutdown_tx.clone();
        tokio::spawn(async move {
            let _ = tokio::signal::ctrl_c().await;
            info!("Received Ctrl-C");
            if let Ok(mut lock) = tx_clone.lock() {
                if let Some(tx) = lock.take() {
                    let _ = tx.send(());
                }
            }
        });
    }

    let index_dir = crate::file_ops::get_index_dir();
    let storage = match crate::index::storage::Storage::open(&index_dir) {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to open index storage for daemon: {}", e);
            return Err(e.into());
        }
    };

    // ── Accept loop ────────────────────────────────────────────────────────
    loop {
        tokio::select! {
            accept = listener.accept() => {
                let (mut socket, _) = accept?;
                let token_ref = token.clone();
                let tx_clone  = shutdown_tx.clone();
                let storage_clone = storage.clone();

                tokio::spawn(async move {
                    let mut buf = vec![0u8; 10 * 1024 * 1024]; // 10 MB cap
                    match tokio::time::timeout(
                        std::time::Duration::from_secs(60),
                        socket.read(&mut buf),
                    ).await {
                        Ok(Ok(n)) if n > 0 => {
                            match serde_json::from_slice::<JsonRpcRequest>(&buf[..n]) {
                                Ok(req) => {
                                    let provided = req.params
                                        .get("token")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("");

                                    if provided != token_ref {
                                        error!("Invalid token provided"); // token value never logged
                                        let res = serde_json::json!({
                                            "jsonrpc": "2.0",
                                            "error": {
                                                "code": -32001,
                                                "message": "Invalid or missing auth token"
                                            },
                                            "id": req.id
                                        });
                                        let _ = socket.write_all(&serde_json::to_vec(&res).unwrap()).await;
                                        return;
                                    }

                                    match req.method.as_str() {
                                        "status" => {
                                            let res = JsonRpcResponse {
                                                jsonrpc: "2.0".to_string(),
                                                result: Some(serde_json::json!({"status": "running"})),
                                                error: None,
                                                id: req.id,
                                            };
                                            let _ = socket.write_all(&serde_json::to_vec(&res).unwrap()).await;
                                        }
                                        "stop" => {
                                            info!("Stop requested via RPC, shutting down gracefully.");
                                            let res = JsonRpcResponse {
                                                jsonrpc: "2.0".to_string(),
                                                result: Some(serde_json::json!({"status": "stopping"})),
                                                error: None,
                                                id: req.id,
                                            };
                                            let _ = socket.write_all(&serde_json::to_vec(&res).unwrap()).await;
                                            // Trigger graceful shutdown
                                            if let Ok(mut lock) = tx_clone.lock() {
                                                if let Some(tx) = lock.take() {
                                                    let _ = tx.send(());
                                                }
                                            }
                                        }
                                        "search" => {
                                            let query = req.params.get("query").and_then(|v| v.as_str()).unwrap_or("");
                                            let limit = req.params.get("limit").and_then(|v| v.as_u64()).unwrap_or(20) as usize;
                                            let engine = crate::index::query::QueryEngine::new(storage_clone.clone());
                                            
                                            match engine.execute(query, None, None, limit, 0) {
                                                Ok(results) => {
                                                    let res = JsonRpcResponse {
                                                        jsonrpc: "2.0".to_string(),
                                                        result: Some(serde_json::json!(results)),
                                                        error: None,
                                                        id: req.id,
                                                    };
                                                    let _ = socket.write_all(&serde_json::to_vec(&res).unwrap()).await;
                                                }
                                                Err(e) => {
                                                    let res = JsonRpcResponse {
                                                        jsonrpc: "2.0".to_string(),
                                                        result: None,
                                                        error: Some(serde_json::json!({ "code": -32000, "message": e.to_string() })),
                                                        id: req.id,
                                                    };
                                                    let _ = socket.write_all(&serde_json::to_vec(&res).unwrap()).await;
                                                }
                                            }
                                        }
                                        "read" => {
                                            let path_str = req.params.get("path").and_then(|v| v.as_str()).unwrap_or("");
                                            if path_str.contains("..") {
                                                let res = JsonRpcResponse {
                                                    jsonrpc: "2.0".to_string(),
                                                    result: None,
                                                    error: Some(serde_json::json!({ "code": -32000, "message": "Invalid path" })),
                                                    id: req.id,
                                                };
                                                let _ = socket.write_all(&serde_json::to_vec(&res).unwrap()).await;
                                            } else {
                                                match std::fs::read_to_string(path_str) {
                                                    Ok(content) => {
                                                        let res = JsonRpcResponse {
                                                            jsonrpc: "2.0".to_string(),
                                                            result: Some(serde_json::json!({ "content": content })),
                                                            error: None,
                                                            id: req.id,
                                                        };
                                                        let _ = socket.write_all(&serde_json::to_vec(&res).unwrap()).await;
                                                    }
                                                    Err(e) => {
                                                        let res = JsonRpcResponse {
                                                            jsonrpc: "2.0".to_string(),
                                                            result: None,
                                                            error: Some(serde_json::json!({ "code": -32000, "message": e.to_string() })),
                                                            id: req.id,
                                                        };
                                                        let _ = socket.write_all(&serde_json::to_vec(&res).unwrap()).await;
                                                    }
                                                }
                                            }
                                        }
                                        _ => {
                                            let res = JsonRpcResponse {
                                                jsonrpc: "2.0".to_string(),
                                                result: None,
                                                error: Some(serde_json::json!({
                                                    "code": -32601,
                                                    "message": "Method not found"
                                                })),
                                                id: req.id,
                                            };
                                            let _ = socket.write_all(&serde_json::to_vec(&res).unwrap()).await;
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("JSON parse error: {}", e);
                                    let res = serde_json::json!({
                                        "jsonrpc": "2.0",
                                        "error": { "code": -32700, "message": "Parse error" },
                                        "id": serde_json::Value::Null
                                    });
                                    let _ = socket.write_all(&serde_json::to_vec(&res).unwrap()).await;
                                }
                            }
                        }
                        Ok(Ok(_)) => { /* 0 bytes, client closed */ }
                        Ok(Err(e)) => { error!("Socket read error: {}", e); }
                        Err(_) => { info!("Connection idle timeout, dropping socket"); }
                    }
                });
            }

            _ = &mut shutdown_rx => {
                // Graceful shutdown: clean up state files so status detects it immediately
                info!("Graceful shutdown: removing state files and exiting.");
                cleanup_state_files();
                // Allow in-flight tokio tasks a moment to finish writes
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                std::process::exit(0);
            }
        }
    }
}

// ── Client helpers ─────────────────────────────────────────────────────────────

async fn send_rpc(method: &str, mut params: serde_json::Value) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let socket_path = "/tmp/sumi/yomi.sock";
    
    if let Some(obj) = params.as_object_mut() {
        obj.insert("token".to_string(), serde_json::Value::String("uds-internal-trust".to_string()));
    }

    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: method.to_string(),
        params,
        id: 1,
    };

    let mut stream = UnixStream::connect(socket_path).await?;
    stream.write_all(&serde_json::to_vec(&req)?).await?;

    let mut buf = vec![0u8; 4096];
    let n = stream.read(&mut buf).await?;

    let res: JsonRpcResponse = serde_json::from_slice(&buf[..n])?;
    Ok(res.result.unwrap_or(serde_json::json!({})))
}
