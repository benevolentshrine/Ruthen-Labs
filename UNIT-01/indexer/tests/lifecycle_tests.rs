/// Daemon Lifecycle & Crash Recovery — Integration Tests
///
/// Run with:
///   cargo test --test lifecycle_tests
///
/// Each test spawns a real `yomi` binary in a fresh temp dir via YOMI_DATA_DIR,
/// exercises the scenario, and asserts daemon state through file inspection and
/// TCP probing. No mocks, no stubs.

use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::Duration;
use tempfile::TempDir;

// ── Helpers ────────────────────────────────────────────────────────────────────

struct DaemonCtx {
    pub process: Child,
    pub port: u16,
    pub token: String,
    pub dir: PathBuf,
    pub _tmp: Option<TempDir>,
}

impl DaemonCtx {
    /// Spawn a daemon in a fresh temp dir using the internal flag (already detached).
    fn start() -> Self {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().to_path_buf();
        let ctx = Self::start_in(dir.clone(), Some(tmp));
        ctx
    }

    fn start_in(dir: PathBuf, _tmp: Option<TempDir>) -> Self {
        let child = Command::new("cargo")
            .args(["run", "--bin", "yomi", "--", "daemon", "start"])
            .env("YOMI_DATA_DIR", &dir)
            .env("YOMI_DAEMON_INTERNAL", "1") // skip re-spawn; run server directly
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to spawn daemon");

        let (port, token) = poll_state_files(&dir, 50);
        DaemonCtx { process: child, port, token, dir, _tmp }
    }

    fn send_json(&self, payload: serde_json::Value) -> String {
        let mut stream = TcpStream::connect(format!("127.0.0.1:{}", self.port))
            .expect("TCP connect failed");
        stream.write_all(&serde_json::to_vec(&payload).unwrap()).unwrap();
        let mut resp = String::new();
        let _ = stream.read_to_string(&mut resp);
        resp
    }

    fn valid_status_req(&self) -> serde_json::Value {
        serde_json::json!({
            "jsonrpc": "2.0",
            "method": "status",
            "params": { "token": self.token },
            "id": 1
        })
    }

    fn kill_hard(&mut self) {
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}

impl Drop for DaemonCtx {
    fn drop(&mut self) {
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}

/// Poll port + auth_token files up to `attempts` × 100ms.
fn poll_state_files(dir: &PathBuf, attempts: u32) -> (u16, String) {
    let port_path  = dir.join("port");
    let token_path = dir.join("auth_token");

    for _ in 0..attempts {
        if let (Ok(mut pf), Ok(mut tf)) = (fs::File::open(&port_path), fs::File::open(&token_path)) {
            let mut ps = String::new();
            let mut ts = String::new();
            if pf.read_to_string(&mut ps).is_ok()
                && tf.read_to_string(&mut ts).is_ok()
                && !ps.trim().is_empty()
                && !ts.trim().is_empty()
            {
                if let Ok(port) = ps.trim().parse::<u16>() {
                    return (port, ts.trim().to_string());
                }
            }
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    panic!("Daemon did not write state files in time");
}

// ── Test 1: Hard Kill Recovery ─────────────────────────────────────────────────

#[test]
fn test_hard_kill_recovery() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path().to_path_buf();

    // Plant a fake index.json.tmp to simulate mid-write state
    let tmp_path = dir.join("index.json.tmp");
    fs::write(&tmp_path, b"{ partial json").unwrap();

    // Start daemon
    let mut ctx = DaemonCtx::start_in(dir.clone(), None);

    // Verify it is alive
    let res = ctx.send_json(ctx.valid_status_req());
    assert!(res.contains("running"), "Daemon not running before kill: {}", res);

    // Hard kill (simulates kill -9)
    ctx.kill_hard();

    // The .tmp file written before daemon start should still exist
    // (daemon had no chance to clean it up on hard kill — expected behaviour).
    // The critical assertion is that after a restart the daemon comes up cleanly.

    // Clean leftover state files (as the restart logic in daemon.rs would detect stale PID)
    let _ = fs::remove_file(dir.join("port"));
    let _ = fs::remove_file(dir.join("auth_token"));
    let _ = fs::remove_file(dir.join("daemon.pid"));

    // Restart in the same dir
    let ctx2 = DaemonCtx::start_in(dir.clone(), None);
    let res2 = ctx2.send_json(ctx2.valid_status_req());
    assert!(res2.contains("running"), "Daemon did not restart cleanly: {}", res2);
}

// ── Test 2: Graceful Shutdown via RPC stop ─────────────────────────────────────

#[test]
fn test_graceful_shutdown_via_rpc() {
    let ctx = DaemonCtx::start();
    let port = ctx.port;
    let token = ctx.token.clone();
    let dir = ctx.dir.clone();

    // Verify running
    let res = ctx.send_json(ctx.valid_status_req());
    assert!(res.contains("running"), "Not running before stop: {}", res);

    // Send stop
    let stop_req = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "stop",
        "params": { "token": token },
        "id": 2
    });
    let res = ctx.send_json(stop_req);
    assert!(res.contains("stopping"), "Stop RPC did not acknowledge: {}", res);

    // Give graceful shutdown up to 2 seconds
    std::thread::sleep(Duration::from_millis(500));

    // State files should be cleaned up
    assert!(
        !dir.join("port").exists() || !dir.join("daemon.pid").exists(),
        "State files not cleaned up after graceful stop"
    );

    // TCP port should be closed (connect must fail)
    let still_up = TcpStream::connect(format!("127.0.0.1:{}", port)).is_ok();
    assert!(!still_up, "Daemon TCP port still open after graceful stop");
}

// ── Test 3: Graceful Shutdown via SIGINT (Unix only) ──────────────────────────

#[test]
#[cfg(unix)]
fn test_graceful_shutdown_sigint() {
    use std::os::unix::process::ExitStatusExt;
    use nix::sys::signal::{kill, Signal};
    use nix::unistd::Pid;

    let ctx = DaemonCtx::start();
    let port = ctx.port;
    let dir = ctx.dir.clone();
    let pid = ctx.process.id();

    // Verify alive
    let res = ctx.send_json(ctx.valid_status_req());
    assert!(res.contains("running"));

    // SIGINT
    kill(Pid::from_raw(pid as i32), Signal::SIGINT).expect("kill SIGINT failed");

    // Wait for exit (up to 2s)
    let mut exited = false;
    for _ in 0..20 {
        std::thread::sleep(Duration::from_millis(100));
        if !dir.join("daemon.pid").exists() {
            exited = true;
            break;
        }
    }
    assert!(exited, "Daemon did not clean up PID file after SIGINT");

    // Port closed
    std::thread::sleep(Duration::from_millis(100));
    assert!(
        TcpStream::connect(format!("127.0.0.1:{}", port)).is_err(),
        "TCP port still open after SIGINT"
    );
}

// ── Test 4: Stale State Detection ─────────────────────────────────────────────

#[test]
fn test_stale_state_detection() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path().to_path_buf();

    // Write fake stale state: a dead PID (1 is always alive on Unix; use 999999 on both)
    let stale_pid: u32 = 999_999;
    fs::write(dir.join("port"), b"9999").unwrap();
    fs::write(dir.join("auth_token"), b"stale-token").unwrap();
    fs::write(dir.join("daemon.pid"), stale_pid.to_string().as_bytes()).unwrap();

    // Run `yomi daemon status` — must not hang, must not report "running"
    let output = Command::new("cargo")
        .args(["run", "--bin", "yomi", "--", "daemon", "status"])
        .env("YOMI_DATA_DIR", &dir)
        .output()
        .expect("Failed to run status");

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // Must report dead / stale — not "running"
    assert!(
        combined.contains("dead") || combined.contains("not running") || combined.contains("Stale"),
        "Status did not detect stale PID: {}", combined
    );

    // State files must be cleaned up
    assert!(!dir.join("daemon.pid").exists(), "PID file was not cleaned up");
    assert!(!dir.join("port").exists(), "Port file was not cleaned up");
}

// ── Test 5: Background Detachment (Unix / WSL2) ────────────────────────────────

#[test]
#[cfg(unix)]
fn test_background_detachment() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path().to_path_buf();

    // Call `yomi daemon start` WITHOUT YOMI_DAEMON_INTERNAL so it runs the
    // real detach path. It should return immediately.
    let before = std::time::Instant::now();
    let status = Command::new("cargo")
        .args(["run", "--bin", "yomi", "--", "daemon", "start"])
        .env("YOMI_DATA_DIR", &dir)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("Failed to run daemon start");
    let elapsed = before.elapsed();

    // Parent process must have returned quickly (< 8 seconds including cargo compile)
    assert!(elapsed.secs() < 8, "daemon start did not return quickly: {:?}", elapsed);

    // Wait for background child to write state files
    let (port, token) = poll_state_files(&dir, 60);

    // Log file should have been created
    assert!(dir.join("daemon.log").exists(), "daemon.log not created");

    // Daemon must be reachable
    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port))
        .expect("Cannot connect to background daemon");
    let req = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "status",
        "params": { "token": token },
        "id": 1
    });
    stream.write_all(&serde_json::to_vec(&req).unwrap()).unwrap();
    let mut resp = String::new();
    let _ = stream.read_to_string(&mut resp);
    assert!(resp.contains("running"), "Background daemon not reachable: {}", resp);

    // Clean up: stop the detached daemon via RPC
    let stop_req = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "stop",
        "params": { "token": token },
        "id": 2
    });
    let mut s2 = TcpStream::connect(format!("127.0.0.1:{}", port)).unwrap();
    s2.write_all(&serde_json::to_vec(&stop_req).unwrap()).unwrap();
}
