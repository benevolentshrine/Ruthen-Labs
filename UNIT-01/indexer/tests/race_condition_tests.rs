/// Race Condition: Concurrent CLI + Watcher — Integration Tests
///
/// Run with:
///   cargo test --test race_condition_tests
///
/// Spawns 3 concurrent `yomi index` processes against the same index file while
/// a watcher is also running. The fs2 exclusive lock must prevent all corruption.
/// Asserts: all processes exit 0, final index.json is valid JSON, no .tmp leftover.

use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;
use tempfile::TempDir;

fn wait_for_file(path: &PathBuf, timeout_ms: u64) -> bool {
    let deadline = std::time::Instant::now() + Duration::from_millis(timeout_ms);
    while std::time::Instant::now() < deadline {
        if path.exists() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    false
}

#[test]
fn test_concurrent_index_no_corruption() {
    let data_tmp = TempDir::new().unwrap();
    let watch_tmp = TempDir::new().unwrap();
    let watch_dir = watch_tmp.path().to_path_buf();
    let data_dir  = data_tmp.path().to_path_buf();
    let index_path = data_dir.join("index.json");

    // Seed the watch dir with 20 files
    for i in 0..20 {
        fs::write(watch_dir.join(format!("seed_{:02}.txt", i)), b"seed").unwrap();
    }

    // Start watcher background process
    let mut watcher = Command::new("cargo")
        .args([
            "run", "--bin", "yomi", "--",
            "index",
            "--path", watch_dir.to_str().unwrap(),
            "--watch",
        ])
        .env("YOMI_DATA_DIR", &data_dir)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to spawn watcher");

    // Wait for initial index.json to be written
    assert!(wait_for_file(&index_path, 15_000), "Watcher never created index.json");

    // Spawn 3 concurrent `yomi index` processes
    let mut indexers: Vec<_> = (0..3)
        .map(|_| {
            Command::new("cargo")
                .args([
                    "run", "--bin", "yomi", "--",
                    "index",
                    "--path", watch_dir.to_str().unwrap(),
                ])
                .env("YOMI_DATA_DIR", &data_dir)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .expect("Failed to spawn indexer")
        })
        .collect();

    // Wait for all 3 indexers to finish
    let statuses: Vec<_> = indexers
        .iter_mut()
        .map(|c| c.wait().expect("Failed to wait on indexer"))
        .collect();

    // ── Assert 1: All processes exited successfully ────────────────────────
    for (i, status) in statuses.iter().enumerate() {
        assert!(
            status.success(),
            "Indexer {} exited with error: {:?}", i, status
        );
    }

    // Give watcher a moment to finish any in-flight write triggered by the indexers
    std::thread::sleep(Duration::from_millis(500));

    // ── Assert 2: Final index.json is valid JSON ───────────────────────────
    let mut f = fs::File::open(&index_path).expect("index.json missing after concurrent writes");
    let mut content = String::new();
    f.read_to_string(&mut content).unwrap();

    let parsed: Result<Vec<serde_json::Value>, _> = serde_json::from_str(&content);
    assert!(
        parsed.is_ok(),
        "index.json is corrupted (invalid JSON) after concurrent writes:\n{}",
        &content[..content.len().min(500)]
    );

    let records = parsed.unwrap();
    assert!(!records.is_empty(), "index.json is empty after concurrent writes");

    // ── Assert 3: No orphaned .tmp file (no abandoned partial write) ───────
    let tmp_path = index_path.with_extension("json.tmp");
    assert!(
        !tmp_path.exists(),
        "Orphaned .tmp file found — a write was abandoned mid-way: {:?}", tmp_path
    );

    // Tear down watcher
    let _ = watcher.kill();
    let _ = watcher.wait();
}
