mod models;
mod walker;
mod hasher;
mod daemon;
mod watcher;
mod file_ops;
mod index;
mod output;
use crate::output::Formatter;
use clap::{Parser, Subcommand};
use tracing::{info, error, Level};
use tracing_subscriber::FmtSubscriber;
use walker::Walker;
use std::path::PathBuf;
use std::fs::File;
use std::io::Write;
use std::sync::{Arc, Mutex};


#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub enum DaemonAction {
    Start,
    Stop,
    Status,
}

#[derive(Subcommand)]
enum Commands {
    /// Full index of a directory
    Index {
        /// The path to index
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
        /// Watch directory for changes
        #[arg(short, long)]
        watch: bool,
    },
    /// List indexed files
    List,
    /// Create .indexer.toml configuration
    Init,
    /// Manage the Indexer background daemon
    Daemon {
        #[command(subcommand)]
        action: DaemonAction,
    },
    /// Query the indexed files
    Query {
        pattern: String,
        /// Filter by language
        #[arg(short, long)]
        lang: Option<String>,
        /// Filter by path glob
        #[arg(short, long)]
        path: Option<String>,
        /// Output as JSON array
        #[arg(long)]
        json: bool,
        /// Output as newline-delimited JSON
        #[arg(long)]
        ndjson: bool,
        /// Number of results to return
        #[arg(long, default_value = "20")]
        limit: usize,
        /// Number of results to skip
        #[arg(long, default_value = "0")]
        offset: usize,
    },
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Shared write lock: guards index.json writes in-process.
    // All disk writes also acquire the fs2 OS-level lock for cross-process safety.
    let write_lock: Arc<Mutex<()>> = Arc::new(Mutex::new(()));
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    let cli = Cli::parse();

    match &cli.command {
        Commands::Index { path, watch } => {
            info!("Starting index command for {:?}", path);
            let walker = Walker::new(path);
            let records = walker.walk();

            let index_dir = file_ops::get_index_dir();
            let storage = index::storage::Storage::open(&index_dir).expect("Failed to open index storage");
            let manager = file_ops::IndexManager::new(storage, write_lock.clone());

            match manager.write_index(records) {
                Ok(_) => info!("Successfully indexed files into Sled store at {:?}", index_dir),
                Err(e) => error!("Failed to save index: {}", e),
            }

            if *watch {
                info!("Starting file watcher on {:?}", path);
                watcher::start_watching(path.clone(), index_dir.join("index.json"), write_lock.clone()).await?;
            }
        }
        Commands::List => {
            let index_dir = file_ops::get_index_dir();
            let _storage = index::storage::Storage::open(&index_dir).expect("Failed to open index storage");

            println!("Listing indexed files (Sled metadata store):");
            // Note: List implementation will be fully fleshed out in query/storage modules
        }
        Commands::Init => {
            let config_path = PathBuf::from(".indexer.toml");
            if config_path.exists() {
                info!(".indexer.toml already exists!");
            } else {
                match File::create(&config_path) {
                    Ok(mut file) => {
                        let default_config = "[indexer]\nthreads = 4\n\n[ignore]\ncustom_ignore = \".indexerignore\"\n";
                        if let Err(e) = file.write_all(default_config.as_bytes()) {
                            error!("Failed to write to .indexer.toml: {}", e);
                        } else {
                            info!("Successfully created .indexer.toml configuration file.");
                        }
                    }
                    Err(e) => {
                        error!("Failed to create .indexer.toml: {}", e);
                    }
                }
            }
        }
        Commands::Daemon { action } => {
            daemon::handle_daemon_action(action).await?;
        }
        Commands::Query { pattern, lang, path, json, ndjson, limit, offset } => {
            let index_dir = file_ops::get_index_dir();
            let storage = index::storage::Storage::open(&index_dir).expect("Failed to open index storage");
            let engine = index::query::QueryEngine::new(storage);

            let results = engine.execute(pattern, lang.as_deref(), path.as_deref(), *limit, *offset)
                .expect("Query execution failed");

            if *json {
                let mut writer = std::io::BufWriter::new(std::io::stdout());
                let formatter = output::JsonFormatter;
                formatter.start_output(&mut writer).unwrap();
                for (i, record) in results.iter().enumerate() {
                    formatter.format_record(record, &mut writer).unwrap();
                    if i < results.len() - 1 {
                        writer.write_all(b",\n").unwrap();
                    }
                }
                formatter.end_output(&mut writer).unwrap();
            } else if *ndjson {
                let mut writer = std::io::BufWriter::new(std::io::stdout());
                let formatter = output::NdJsonFormatter;
                for record in results {
                    formatter.format_record(&record, &mut writer).unwrap();
                }
            } else {
                println!("Found {} matches for '{}':", results.len(), pattern);
                for record in results {
                    println!("- {}", record.path);
                }
            }
        }
    }

    Ok(())
}
