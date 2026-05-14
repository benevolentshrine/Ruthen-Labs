/// File Watcher & Incremental Sync — Integration Tests
///
/// Run with:
///   cargo test --test watcher_tests
///
/// Each test spawns a real `yomi index --watch` process in a fresh YOMI_DATA_DIR
/// temp dir, exercises filesystem scenarios, and asserts index.json state.

use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::Duration;
use tempfile::TempDir;

// ── Helpers ────────────────────────────────────────────────────────────────────

struct WatcherProc {
    child: Child,
    pub index_path: PathBuf,
}

impl WatcherProc {
    fn start(watch_dir: PathBuf, data_dir: PathBuf) -> Self {
        let index_path = data_dir.join("index.json");

        let child = Command::new("cargo")
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

        // Poll until the initial index.json exists
        for _ in 0..60 {
            if index_path.exists() {
                break;
            }
            std::thread::sleep(Duration::from_millis(150));
        }
        assert!(index_path.exists(), "Watcher never wrote index.json");

        WatcherProc { child, index_path }
    }

    fn read_records(&self) -> Vec<serde_json::Value> {
        let mut f = fs::File::open(&self.index_path).expect("index.json missing");
        let mut s = String::new();
        f.read_to_string(&mut s).unwrap();
        serde_json::from_str::<Vec<serde_json::Value>>(&s).unwrap_or_default()
    }

    /// Poll until predicate returns true or timeout expires.
    fn poll_until<F: Fn(&Vec<serde_json::Value>) -> bool>(&self, pred: F, timeout_ms: u64) -> bool {
        let deadline = std::time::Instant::now() + Duration::from_millis(timeout_ms);
        while std::time::Instant::now() < deadline {
            let records = self.read_records();
            if pred(&records) {
                return true;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        false
    }
}

impl Drop for WatcherProc {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

// ── Test 1: Event Debouncing ───────────────────────────────────────────────────

#[test]
fn test_event_debouncing() {
    let data_tmp = TempDir::new().unwrap();
    let watch_tmp = TempDir::new().unwrap();
    let watch_dir = watch_tmp.path().to_path_buf();
    let data_dir = data_tmp.path().to_path_buf();

    let watcher = WatcherProc::start(watch_dir.clone(), data_dir);

    let target = watch_dir.join("debounce.txt");

    // Write the same file 3 times within 100ms — watcher should coalesce into 1 write cycle
    for i in 0..3u8 {
        fs::write(&target, vec![b'a' + i; 64]).unwrap();
        std::thread::sleep(Duration::from_millis(30));
    }

    // Wait for debounce + processing (DEBOUNCE_MS=200 + processing slack)
    let ok = watcher.poll_until(
        |records| records.iter().any(|r| r["path"].as_str().unwrap_or("").contains("debounce.txt")),
        2000,
    );
    assert!(ok, "debounce.txt never appeared in index");

    // Verify no duplicate entries for the same path
    let records = watcher.read_records();
    let matches: Vec<_> = records
        .iter()
        .filter(|r| r["path"].as_str().unwrap_or("").contains("debounce.txt"))
        .collect();
    assert_eq!(matches.len(), 1, "Duplicate entries found after debounce: {:?}", matches);
}

// ── Test 2: Rapid Churn Stress ─────────────────────────────────────────────────

#[test]
fn test_rapid_churn_stress() {
    let data_tmp = TempDir::new().unwrap();
    let watch_tmp = TempDir::new().unwrap();
    let watch_dir = watch_tmp.path().to_path_buf();
    let data_dir = data_tmp.path().to_path_buf();

    let watcher = WatcherProc::start(watch_dir.clone(), data_dir);

    // Create 300 files rapidly (simulates `npm install` burst)
    let count = 300usize;
    for i in 0..count {
        fs::write(watch_dir.join(format!("file_{:04}.txt", i)), b"data").unwrap();
    }

    // Watcher must stay alive and index all files within 30s
    let ok = watcher.poll_until(|records| records.len() >= count, 30_000);
    assert!(ok, "Watcher did not index all {} files. Got: {}", count, watcher.read_records().len());

    // Verify no duplicates
    let records = watcher.read_records();
    let mut paths: Vec<_> = records.iter().filter_map(|r| r["path"].as_str()).collect();
    let before = paths.len();
    paths.sort_unstable();
    paths.dedup();
    assert_eq!(before, paths.len(), "Duplicate index entries after churn");
}

// ── Test 3a: Symlink Loop Detection (Unix) ─────────────────────────────────────

#[test]
#[cfg(unix)]
fn test_symlink_loop_detection() {
    use std::os::unix::fs::symlink;

    let data_tmp = TempDir::new().unwrap();
    let watch_tmp = TempDir::new().unwrap();
    let watch_dir = watch_tmp.path().to_path_buf();
    let data_dir = data_tmp.path().to_path_buf();

    let sub = watch_dir.join("subdir");
    fs::create_dir(&sub).unwrap();

    // Create a symlink loop: subdir/loop -> subdir (points to parent dir)
    symlink(&sub, sub.join("loop")).unwrap();

    // Create a real file to ensure the watcher has something valid to process
    fs::write(watch_dir.join("real.txt"), b"hello").unwrap();

    let watcher = WatcherProc::start(watch_dir.clone(), data_dir);

    // Trigger a change event in the non-looped directory
    fs::write(watch_dir.join("trigger.txt"), b"trigger").unwrap();

    // Watcher must NOT hang; give it 5s to process trigger.txt
    let ok = watcher.poll_until(
        |records| records.iter().any(|r| r["path"].as_str().unwrap_or("").contains("trigger.txt")),
        5000,
    );
    assert!(ok, "Watcher hung or did not process trigger.txt after symlink loop was present");
}

// ── Test 3b: Restricted File — Warn & Continue (Windows) ──────────────────────

#[test]
#[cfg(windows)]
fn test_windows_inaccessible_file_warn() {
    use fs2::FileExt;

    let data_tmp = TempDir::new().unwrap();
    let watch_tmp = TempDir::new().unwrap();
    let watch_dir = watch_tmp.path().to_path_buf();
    let data_dir = data_tmp.path().to_path_buf();

    // Write a file that will be locked exclusively during watcher processing
    let locked_path = watch_dir.join("locked.txt");
    fs::write(&locked_path, b"initial").unwrap();

    let watcher = WatcherProc::start(watch_dir.clone(), data_dir);

    // Hold an exclusive lock on the file so hash_file inside process_file may
    // encounter an access issue, exercising the warn-and-skip path.
    let lock_handle = fs::OpenOptions::new()
        .write(true)
        .open(&locked_path)
        .unwrap();
    let _ = lock_handle.lock_exclusive(); // best-effort; may not block read on all Windows configs

    // Modify a second file to trigger a batch event
    let other = watch_dir.join("other.txt");
    fs::write(&other, b"data").unwrap();

    // Watcher must stay alive and index other.txt
    let ok = watcher.poll_until(
        |records| records.iter().any(|r| r["path"].as_str().unwrap_or("").contains("other.txt")),
        5000,
    );
    assert!(ok, "Watcher did not continue past locked file: other.txt never indexed");

    // Explicitly release lock
    let _ = lock_handle.unlock();
}

// ── Test 4: Cross-Platform Event Parity ───────────────────────────────────────

#[test]
fn test_cross_platform_event_parity() {
    let data_tmp = TempDir::new().unwrap();
    let watch_tmp = TempDir::new().unwrap();
    let watch_dir = watch_tmp.path().to_path_buf();
    let data_dir = data_tmp.path().to_path_buf();

    let watcher = WatcherProc::start(watch_dir.clone(), data_dir);

    let file = watch_dir.join("parity.txt");

    // CREATE — file must appear in index
    fs::write(&file, b"v1").unwrap();
    let ok = watcher.poll_until(
        |records| records.iter().any(|r| r["path"].as_str().unwrap_or("").contains("parity.txt")),
        5000,
    );
    assert!(ok, "File not indexed after create");

    let hash_v1 = watcher
        .read_records()
        .into_iter()
        .find(|r| r["path"].as_str().unwrap_or("").contains("parity.txt"))
        .and_then(|r| r["hash"].as_str().map(|s| s.to_string()))
        .unwrap();

    // MODIFY — hash must change
    fs::write(&file, b"v2_different_content").unwrap();
    let ok = watcher.poll_until(
        |records| {
            records
                .iter()
                .find(|r| r["path"].as_str().unwrap_or("").contains("parity.txt"))
                .and_then(|r| r["hash"].as_str())
                .map(|h| h != hash_v1)
                .unwrap_or(false)
        },
        5000,
    );
    assert!(ok, "Hash did not change after file modification");

    // DELETE — file must be removed from index
    fs::remove_file(&file).unwrap();
    let ok = watcher.poll_until(
        |records| !records.iter().any(|r| r["path"].as_str().unwrap_or("").contains("parity.txt")),
        5000,
    );
    assert!(ok, "File still in index after deletion");
}
