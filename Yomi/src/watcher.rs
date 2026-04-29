use std::path::PathBuf;
use tracing::{info, error};
use notify::{Watcher, RecursiveMode, EventKind};
use crate::models::FileRecord;
use std::fs::File;
use std::io::BufReader;
use crate::walker::process_file;

pub async fn start_watching(path: PathBuf, index_path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let (tx, rx) = std::sync::mpsc::channel();
    
    // Create a watcher object, delivering debounced events.
    // The notification back-end is selected based on the platform.
    let mut watcher = notify::recommended_watcher(tx)?;
    
    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(&path, RecursiveMode::Recursive)?;
    
    info!("Watcher is running. Listening for changes...");
    
    for res in rx {
        match res {
            Ok(event) => {
                match event.kind {
                    EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                        info!("File change detected: {:?}", event.paths);
                        // Load current index
                        let mut records: Vec<FileRecord> = match File::open(&index_path) {
                            Ok(file) => {
                                let reader = BufReader::new(file);
                                serde_json::from_reader(reader).unwrap_or_else(|_| Vec::new())
                            }
                            Err(_) => Vec::new(),
                        };

                        let mut changed = false;

                        for changed_path in &event.paths {
                            if !changed_path.exists() {
                                // File was deleted
                                records.retain(|r| r.path != changed_path.to_string_lossy().to_string());
                                changed = true;
                            } else if changed_path.is_file() {
                                // File was created or modified
                                if let Ok(Some(new_record)) = process_file(changed_path, &path) {
                                    // Remove old record if it exists
                                    records.retain(|r| r.path != new_record.path);
                                    // Add new record
                                    records.push(new_record);
                                    changed = true;
                                }
                            }
                        }

                        if changed {
                            // Atomic save
                            let tmp_path = index_path.with_extension("json.tmp");
                            if let Ok(file) = File::create(&tmp_path) {
                                if serde_json::to_writer_pretty(file, &records).is_ok() {
                                    if let Err(e) = std::fs::rename(&tmp_path, &index_path) {
                                        error!("Failed to save incremental update: {}", e);
                                    } else {
                                        info!("Incremental sync complete. Index updated.");
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            },
            Err(e) => error!("Watch error: {:?}", e),
        }
    }
    
    Ok(())
}
