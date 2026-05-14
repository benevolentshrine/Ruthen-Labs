use std::fs::File;
use std::io::Write;
use serde_json::{json, Value};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <num_files> <output_path>", args[0]);
        std::process::exit(1);
    }

    let num_files: usize = args[1].parse().expect("Invalid number of files");
    let output_path = &args[2];

    let mut records = Vec::new();
    for i in 0..num_files {
        records.push(json!({
            "path": format!("C:\\FakeIndex\\file_{}.txt", i),
            "relative_path": format!("file_{}.txt", i),
            "hash": format!("{:x}", i),
            "size_bytes": 1024,
            "mtime_unix": 1700000000,
            "language": "text",
            "extension": "txt",
            "is_binary": false,
            "is_symlink": false,
            "permissions": "rw-r--r--",
            "indexed_at": "2026-05-02T00:00:00Z"
        }));
    }

    let file = File::create(output_path).expect("Failed to create file");
    serde_json::to_writer(file, &records).expect("Failed to write JSON");
    println!("Generated {} records at {}", num_files, output_path);
}
