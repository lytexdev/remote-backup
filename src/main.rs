use crate::config::Settings;
use structopt::StructOpt;
use rand::Rng;
use std::fs::File;
use std::io::{self, Write};
mod config;
mod backup;
mod encryption;
mod ssh;
mod cleanup;

#[derive(StructOpt)]
enum Command {
    List,
    Backup,
    Restore { filename: String },
    GenerateKey,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let command = Command::from_args();
    let settings = config::load_config()?;

    match command {
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
