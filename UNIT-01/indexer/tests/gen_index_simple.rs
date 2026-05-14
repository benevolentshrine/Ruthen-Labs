use std::fs::File;
use std::io::Write;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <num_files> <output_path>", args[0]);
        std::process::exit(1);
    }

    let num_files: usize = args[1].parse().expect("Invalid number of files");
    let output_path = &args[2];

    let mut file = File::create(output_path).expect("Failed to create file");

    write!(file, "[").expect("Failed to write");
    for i in 0..num_files {
        let record = format!(
            r#"{{"path":"C:\\FakeIndex\\file_{}.txt","relative_path":"file_{}.txt","hash":"{:x}","size_bytes":1024,"mtime_unix":1700000000,"language":"text","extension":"txt","is_binary":false,"is_symlink":false,"permissions":"rw-r--r--","indexed_at":"2026-05-02T00:00:00Z"}}"#,
            i, i, i
        );
        write!(file, "{}", record).expect("Failed to write");
        if i < num_files - 1 {
            write!(file, ",").expect("Failed to write");
        }
    }
    write!(file, "]").expect("Failed to write");
    println!("Generated {} records at {}", num_files, output_path);
}
