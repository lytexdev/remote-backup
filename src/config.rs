use config::{Config, File};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Settings {
    pub backup_folder: PathBuf,
    pub exclude_paths: Vec<PathBuf>,
    pub remote_backup_dir: PathBuf,
    pub restore_path: PathBuf,
    pub compression_level: u32,
    pub max_backups: usize,
    pub tmp_path: PathBuf,
    pub ssh_host: String,
    pub ssh_port: u16,
    pub ssh_user: String,
    pub identity_file: PathBuf,
    pub encryption_key_path: PathBuf,
}

pub fn load_config(config_path: PathBuf) -> Result<Settings, config::ConfigError> {
    let settings = Config::builder()
        .add_source(File::from(config_path))
        .build()?;

    Ok(Settings {
        backup_folder: settings.get::<String>("backup.backup_folder")?.into(),
        exclude_paths: settings
            .get_array("backup.exclude_paths")?
            .into_iter()
            .map(|p| p.into_string().unwrap().into())
            .collect(),
        remote_backup_dir: settings.get::<String>("backup.remote_backup_dir")?.into(),
        restore_path: settings.get::<String>("backup.restore_path")?.into(),
        compression_level: settings.get::<u32>("backup.compression_level")?,
        max_backups: settings.get::<usize>("backup.max_backups")?,
        tmp_path: settings.get::<String>("backup.tmp_path")?.into(),
        ssh_host: settings.get::<String>("ssh.host")?,
        ssh_port: settings.get::<u16>("ssh.port")?,
        ssh_user: settings.get::<String>("ssh.user")?,
        identity_file: settings.get::<String>("ssh.identity_file")?.into(),
        encryption_key_path: settings.get::<String>("encryption.key_path")?.into(),
    })
}
