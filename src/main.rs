use structopt::StructOpt;
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
    }
    Ok(())
}
