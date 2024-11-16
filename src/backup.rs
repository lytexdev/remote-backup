use crate::config::Settings;
use crate::ssh::{upload_file, list_backups, delete_backup};
use std::fs;
use std::io;
use tar::Builder;
use xz2::write::XzEncoder;
use chrono::Local;
use std::process::Command;
use crate::encryption::encrypt_data;

pub fn create_backup(settings: &Settings) -> io::Result<()> {
    let archive_path = format!("{}/backup-{}.tar.xz", settings.tmp_path.display(), Local::now().format("%Y%m%d%H%M"));
    println!("Creating compressed archive at path: {}", archive_path);

    let tar_file = fs::File::create(&archive_path)?;
    let enc = XzEncoder::new(tar_file, settings.compression_level);
    let mut tar = Builder::new(enc);

    for entry in fs::read_dir(&settings.backup_folder)? {
        let entry = entry?;
        let path = entry.path();

        if settings.exclude_hidden {
            if let Some(name) = path.file_name() {
                if name.to_string_lossy().starts_with('.') {
                    println!("Excluding hidden file or directory: {}", path.display());
                    continue;
                }
            }
        }

        if settings.exclude_paths.iter().any(|exclude| path.starts_with(exclude)) {
            println!("Excluding {}", path.display());
            continue;
        }

        if path.is_dir() {
            println!("Adding directory {} to archive", path.display());
            tar.append_dir_all(path.strip_prefix(&settings.backup_folder).unwrap(), &path)?;
        } else if path.is_file() {
            println!("Adding file {} to archive", path.display());
            tar.append_path_with_name(&path, path.strip_prefix(&settings.backup_folder).unwrap())?;
        }
    }

    tar.finish()?;
    println!("Archive created and compressed successfully.");
    drop(tar);

    let archive_size = fs::metadata(&archive_path)?.len();
    println!("Compressed archive size: {} Bytes", archive_size);

    println!("Testing the compressed archive locally...");
    let output = Command::new("tar")
        .arg("-tvf")
        .arg(&archive_path)
        .output()
        .expect("Failed to execute tar command");

    if !output.status.success() {
        println!("Local integrity test failed. Archive might be corrupted.");
        return Err(io::Error::new(io::ErrorKind::Other, "Compressed file might be corrupted"));
    }

    println!("Local integrity test passed. Proceeding with encryption...");

    let encrypted_archive_path = format!("{}.enc", archive_path);
    encrypt_data(&archive_path, &encrypted_archive_path, settings)?;

    manage_backup_limit(settings)?;

    upload_file(&encrypted_archive_path, &fs::read(&encrypted_archive_path)?, settings)?;

    fs::remove_file(&archive_path)?;
    fs::remove_file(&encrypted_archive_path)?;

    println!("Temporary files removed from tmp directory.");
    Ok(())
}

fn manage_backup_limit(settings: &Settings) -> io::Result<()> {
    let mut backups = list_backups(settings)?;
    
    if backups.len() >= settings.max_backups {
        backups.sort();
        let oldest_backup = backups.first().unwrap();
        println!("Deleting oldest backup: {}", oldest_backup);
        delete_backup(oldest_backup, settings)?;
    }

    Ok(())
}
