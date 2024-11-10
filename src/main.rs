use crate::config::{load_config, Settings};
use structopt::StructOpt;
use rand::Rng;
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;

mod config;
mod backup;
mod encryption;
mod ssh;
mod cleanup;

#[derive(StructOpt)]
#[structopt(name = "remote-backup", about = "A CLI for remote backup management")]
struct Cli {
    #[structopt(subcommand)]
    command: Command,

    #[structopt(short = "c", long = "config", parse(from_os_str))]
    config_path: Option<PathBuf>,
}

#[derive(StructOpt)]
enum Command {
    List,
    Backup,
    Restore { filename: String },
    GenerateKey,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::from_args();

    let config_path = args.config_path.unwrap_or_else(|| PathBuf::from("config.toml"));
    let settings = load_config(config_path)?;

    match args.command {
        Command::List => {
            ssh::list_backups(&settings)?;
        }
        Command::Backup => {
            backup::create_backup(&settings)?;
            cleanup::cleanup_old_backups(&settings)?;
        }
        Command::Restore { filename } => {
            ssh::restore_backup(&filename, &settings)?;
        }
        Command::GenerateKey => {
            generate_key(&settings)?;
        }
    }
    Ok(())
}

fn generate_key(settings: &Settings) -> io::Result<()> {
    let key_path = &settings.encryption_key_path;
    let mut key_file = File::create(key_path)?;

    let mut rng = rand::thread_rng();
    let key: Vec<u8> = (0..32).map(|_| rng.gen()).collect();

    key_file.write_all(&key)?;
    println!("Encryption key successfully generated and saved to {}", key_path.display());
    Ok(())
}
