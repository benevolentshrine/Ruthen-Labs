use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::process::{Command, Child};
use std::time::Duration;
use tempfile::TempDir;

struct DaemonContext {
    process: Child,
    port: u16,
    _temp_dir: TempDir,
}

impl DaemonContext {
    fn start() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let dir_path = temp_dir.path().to_str().unwrap().to_string();

        let mut child = Command::new("cargo")
            .arg("run")
            .arg("--bin")
            .arg("indexer")
            .arg("--")
            .arg("daemon")
            .arg("start")
            .env("INDEXER_DATA_DIR", &dir_path)
            .spawn()
            .expect("Failed to spawn daemon");

        // Wait for port file to be created
        let port_file = temp_dir.path().join("port");
        let mut port = 0;
        for _ in 0..50 {
            if let Ok(mut file) = fs::File::open(&port_file) {
                let mut s = String::new();
                if file.read_to_string(&mut s).is_ok() {
                    if let Ok(p) = s.trim().parse() {
                        port = p;
                        break;
                    }
                }
            }
            std::thread::sleep(Duration::from_millis(100));
        }

        if port == 0 {
            child.kill().unwrap();
            panic!("Failed to get port from daemon");
        }

        Self {
            process: child,
            port,
            _temp_dir: temp_dir,
        }
    }
}

impl Drop for DaemonContext {
    fn drop(&mut self) {
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}

fn send_raw(port: u16, payload: &[u8]) -> String {
    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port)).expect("Failed to connect");
    stream.write_all(payload).expect("Failed to write");
    let mut response = String::new();
    let _ = stream.read_to_string(&mut response);
    response
}

#[test]
fn test_json_rpc_negative_payloads() {
    let daemon = DaemonContext::start();

    let payloads = vec![
        // Malformed JSON (missing bracket)
        (b"{\"jsonrpc\": \"2.0\", \"method\": \"status\", \"params\": {\"token\": \"123\"}".as_slice(), "-32700"),
        // Not JSON at all
        (b"HELLO DAEMON".as_slice(), "-32700"),
        // Wrong JSON-RPC version or missing ID (handled by serde parsing error if strict, or our token check)
        // Since we strictly parse JsonRpcRequest, missing required fields fail parse
        (b"{\"method\": \"status\"}".as_slice(), "-32700"), 
        // Array instead of object
        (b"[\"status\"]".as_slice(), "-32700"),
    ];

    for (payload, expected_code) in payloads {
        let res = send_raw(daemon.port, payload);
        assert!(res.contains(expected_code), "Response did not contain {}: {}", expected_code, res);
    }
}
