/// Atomic Writes & Data Integrity — Integration Tests
///
/// Run with:
///   cargo test --test atomic_write_tests
///   cargo test --test atomic_write_tests -- --include-ignored  (cross-FS test)

use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;
use tempfile::TempDir;

fn run_yomi_index(watch_dir: &PathBuf, data_dir: &PathBuf) -> std::process::Output {
    Command::new("cargo")
        .args(["run", "--bin", "yomi", "--", "index", "--path",
               watch_dir.to_str().unwrap()])
        .env("YOMI_DATA_DIR", data_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("Failed to run yomi index")
}

fn read_index(data_dir: &PathBuf) -> Vec<serde_json::Value> {
    let path = data_dir.join("index.json");
    let mut f = fs::File::open(&path).expect("index.json missing");
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    serde_json::from_str(&s).unwrap_or_default()
}

// ── Test 1: Cross-Filesystem Fallback ──────────────────────────────────────────
//
// The fallback function `copy_and_atomic_replace` is unit-tested inside
// src/file_ops.rs and always runs in CI without any multi-partition setup.
//
// This integration test verifies end-to-end behaviour with a real cross-FS
// setup. It is #[ignore] because it requires YOMI_CROSS_FS_DIR to point to
// a path on a different partition/volume than the temp dir.
//
// Run manually:
//   $env:YOMI_CROSS_FS_DIR = "D:\tmp_yomi"   # Windows second volume
//   cargo test --test atomic_write_tests test_cross_filesystem_fallback -- --include-ignored
#[test]
#[ignore]
fn test_cross_filesystem_fallback() {
    let cross_fs_dir = match std::env::var("YOMI_CROSS_FS_DIR") {
        Ok(d) => PathBuf::from(d),
        Err(_) => {
            eprintln!("YOMI_CROSS_FS_DIR not set — skipping");
            return;
        }
    };
    let data_tmp = TempDir::new().unwrap();
    let watch_tmp = TempDir::new().unwrap();
    fs::write(watch_tmp.path().join("file.txt"), b"hello").unwrap();

    let output = Command::new("cargo")
        .args(["run", "--bin", "yomi", "--", "index",
               "--path", watch_tmp.path().to_str().unwrap()])
        .env("YOMI_DATA_DIR", data_tmp.path())
        .env("YOMI_TMP_DIR", &cross_fs_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("Failed to run yomi index");

    assert!(output.status.success(), "yomi index failed on cross-FS setup: {}",
        String::from_utf8_lossy(&output.stderr));
    let records = read_index(&data_tmp.path().to_path_buf());
    assert!(!records.is_empty(), "No records indexed");
    assert!(!data_tmp.path().join("index.json.tmp").exists(), "Orphaned .tmp");
}

// ── Test 2: Disk Full — index.json must remain untouched ──────────────────────

#[test]
fn test_disk_full_index_untouched() {
    let data_tmp = TempDir::new().unwrap();
    let watch_tmp = TempDir::new().unwrap();
    let data_dir = data_tmp.path().to_path_buf();
    let index_path = data_dir.join("index.json");

    // Plant a known-good sentinel index
    let sentinel = b"[{\"path\":\"sentinel\",\"hash\":\"abc123\"}]";
    fs::write(&index_path, sentinel).unwrap();

    fs::write(watch_tmp.path().join("a.txt"), b"content").unwrap();

    // Inject disk-full via YOMI_FAIL_WRITE=1
    let output = Command::new("cargo")
        .args(["run", "--bin", "yomi", "--", "index",
               "--path", watch_tmp.path().to_str().unwrap()])
        .env("YOMI_DATA_DIR", &data_dir)
        .env("YOMI_FAIL_WRITE", "1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("Failed to run yomi index");

    // index.json must be byte-for-byte unchanged
    let after = fs::read(&index_path).expect("index.json disappeared");
    assert_eq!(after, sentinel, "index.json was modified despite simulated disk-full");

    // No orphaned .tmp
    assert!(!data_dir.join("index.json.tmp").exists(),
        "Orphaned .tmp after disk-full simulation");

    // Error must be logged
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("failed") || stderr.contains("Failed") || stderr.contains("error"),
        "No error logged on disk-full: stderr={}", stderr
    );
}

// ── Test 3: JSON Corruption Recovery ──────────────────────────────────────────

#[test]
fn test_json_corruption_recovery() {
    let data_tmp = TempDir::new().unwrap();
    let watch_tmp = TempDir::new().unwrap();
    let data_dir = data_tmp.path().to_path_buf();
    let index_path = data_dir.join("index.json");

    for i in 0..5 {
        fs::write(watch_tmp.path().join(format!("f{}.txt", i)), b"data").unwrap();
    }

    // Scenario A: truncated JSON
    fs::write(&index_path, b"{ truncated garbage [[[").unwrap();
    let out = run_yomi_index(&watch_tmp.path().to_path_buf(), &data_dir);
    assert!(out.status.success(), "Panicked on truncated index.json");
    let records = read_index(&data_dir);
    assert_eq!(records.len(), 5, "Not all files re-indexed after truncation: {}", records.len());

    // Scenario B: invalid UTF-8
    fs::write(&index_path, &[0xFF, 0xFE, 0xFD, 0x00, 0x01]).unwrap();
    let out = run_yomi_index(&watch_tmp.path().to_path_buf(), &data_dir);
    assert!(out.status.success(), "Panicked on invalid UTF-8 index.json");
    let records = read_index(&data_dir);
    assert_eq!(records.len(), 5, "Not all files re-indexed after UTF-8 corruption");

    // Scenario C: empty file
    fs::write(&index_path, b"").unwrap();
    let out = run_yomi_index(&watch_tmp.path().to_path_buf(), &data_dir);
    assert!(out.status.success(), "Panicked on empty index.json");
    let records = read_index(&data_dir);
    assert_eq!(records.len(), 5, "Not all files re-indexed after empty index.json");
}

// ── Test 4: Idempotency ────────────────────────────────────────────────────────

#[test]
fn test_idempotent_index() {
    let data_tmp = TempDir::new().unwrap();
    let watch_tmp = TempDir::new().unwrap();
    let data_dir = data_tmp.path().to_path_buf();
    let index_path = data_dir.join("index.json");

    for i in 0..8 {
        fs::write(watch_tmp.path().join(format!("stable_{:02}.txt", i)), b"stable").unwrap();
    }

    // First run
    let out1 = run_yomi_index(&watch_tmp.path().to_path_buf(), &data_dir);
    assert!(out1.status.success(), "First index run failed");

    let content1 = fs::read(&index_path).expect("index.json missing after first run");
    let mtime1 = fs::metadata(&index_path).unwrap().modified().unwrap();

    // Wait long enough for mtime to change on coarse-resolution systems (FAT32 = 2s)
    std::thread::sleep(Duration::from_millis(2100));

    // Second run — identical directory
    let out2 = run_yomi_index(&watch_tmp.path().to_path_buf(), &data_dir);
    assert!(out2.status.success(), "Second index run failed");

    let content2 = fs::read(&index_path).expect("index.json missing after second run");
    let mtime2 = fs::metadata(&index_path).unwrap().modified().unwrap();

    // Assert 1: logical content is identical (same paths + hashes)
    let mut r1: Vec<serde_json::Value> = serde_json::from_slice(&content1).unwrap();
    let mut r2: Vec<serde_json::Value> = serde_json::from_slice(&content2).unwrap();
    assert_eq!(r1.len(), r2.len(), "File count changed between identical runs");

    let key = |r: &serde_json::Value| (
        r["path"].as_str().unwrap_or("").to_string(),
        r["hash"].as_str().unwrap_or("").to_string(),
    );
    r1.sort_by_key(key); r2.sort_by_key(key);
    assert_eq!(r1, r2, "Record content differs between identical runs");

    // Assert 2: mtime unchanged — idempotency guard skipped the write
    assert_eq!(mtime1, mtime2,
        "index.json was rewritten on second run despite no changes (idempotency guard failed)");

    // Assert 3: second run logs the skip
    let stderr2 = String::from_utf8_lossy(&out2.stderr);
    assert!(
        stderr2.contains("unchanged") || stderr2.contains("skipping"),
        "Second run did not log idempotency skip. stderr={}", stderr2
    );
}
