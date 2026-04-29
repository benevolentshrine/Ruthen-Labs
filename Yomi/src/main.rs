mod models;
mod walker;
mod hasher;
mod daemon;
mod watcher;
use clap::{Parser, Subcommand};
use tracing::{info, error, Level};
use tracing_subscriber::FmtSubscriber;
use walker::Walker;
use std::path::PathBuf;
use std::fs::File;
use std::io::{BufReader, Write};
use directories::ProjectDirs;

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
    /// Create .yomi.toml configuration
    Init,
    /// Manage the Yomi background daemon
    Daemon {
        #[command(subcommand)]
        action: DaemonAction,
    },
    /// Query the indexed files
    Query {
        pattern: String,
    },
}

fn get_index_path() -> PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from("com", "momo", "yomi") {
        let data_dir = proj_dirs.data_dir();
        if !data_dir.exists() {
            std::fs::create_dir_all(data_dir).unwrap_or_else(|e| {
                error!("Failed to create data directory: {}", e);
            });
        }
        data_dir.join("index.json")
    } else {
        PathBuf::from("index.json")
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
            
            let index_path = get_index_path();
            
            // Atomic Write: Save to temp file first, then rename
            let tmp_path = index_path.with_extension("json.tmp");
            match File::create(&tmp_path) {
                Ok(file) => {
                    if let Err(e) = serde_json::to_writer_pretty(file, &records) {
                        error!("Failed to write to temp file {:?}: {}", tmp_path, e);
                    } else {
                        // Rename for atomic write
                        if let Err(e) = std::fs::rename(&tmp_path, &index_path) {
                            error!("Failed to atomically rename {:?} to {:?}: {}", tmp_path, index_path, e);
                        } else {
                            info!("Successfully processed {} files and saved atomically to {:?}", records.len(), index_path);
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to create temp file {:?}: {}", tmp_path, e);
                }
            }

            if *watch {
                info!("Starting file watcher on {:?}", path);
                watcher::start_watching(path.clone(), index_path.clone()).await?;
            }
        }
        Commands::List => {
            let index_path = get_index_path();
            match File::open(&index_path) {
                Ok(file) => {
                    let reader = BufReader::new(file);
                    match serde_json::from_reader::<_, Vec<models::FileRecord>>(reader) {
                        Ok(records) => {
                            println!("Found {} indexed files in {:?}:", records.len(), index_path);
                            for record in records {
                                println!("- {} (Hash: {})", record.path, record.hash);
                            }
                        }
                        Err(e) => error!("Failed to parse index.json: {}", e),
                    }
                }
                Err(e) => {
                    error!("Could not open {:?}: {}. Run 'yomi index' first.", index_path, e);
                }
            }
        }
        Commands::Init => {
            let config_path = PathBuf::from(".yomi.toml");
            if config_path.exists() {
                info!(".yomi.toml already exists!");
            } else {
                match File::create(&config_path) {
                    Ok(mut file) => {
                        let default_config = "[indexer]\nthreads = 4\n\n[ignore]\ncustom_ignore = \".yomiignore\"\n";
                        if let Err(e) = file.write_all(default_config.as_bytes()) {
                            error!("Failed to write to .yomi.toml: {}", e);
                        } else {
                            info!("Successfully created .yomi.toml configuration file.");
                        }
                    }
                    Err(e) => {
                        error!("Failed to create .yomi.toml: {}", e);
                    }
                }
            }
        }
        Commands::Daemon { action } => {
            daemon::handle_daemon_action(action).await?;
        }
        Commands::Query { pattern } => {
            // Read index and search
            let index_path = get_index_path();
            match File::open(&index_path) {
                Ok(file) => {
                    let reader = BufReader::new(file);
                    match serde_json::from_reader::<_, Vec<models::FileRecord>>(reader) {
                        Ok(records) => {
                            let matches: Vec<_> = records.iter().filter(|r| r.path.contains(pattern)).collect();
                            println!("Found {} matches for '{}':", matches.len(), pattern);
                            for record in matches {
                                println!("- {}", record.path);
                            }
                        }
                        Err(e) => error!("Failed to parse index: {}", e),
                    }
                }
                Err(e) => error!("Could not open index: {}", e),
            }
        }
    }

    Ok(())
}
