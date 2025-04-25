use crate::config::Settings;
use crate::ssh::{upload_file, list_backups, delete_backup};
use std::fs;
use std::io;
use std::path::Path;
use tar::Builder;
use xz2::write::XzEncoder;
use chrono::Local;
use walkdir::WalkDir;
use crate::encryption::encrypt_data;

pub fn create_backup(settings: &Settings) -> io::Result<()> {
    if !settings.tmp_path.exists() {
        fs::create_dir_all(&settings.tmp_path)?;
    }

    let archive_path = format!("{}/backup-{}.tar.xz", settings.tmp_path.display(), Local::now().format("%Y%m%d%H%M"));
    println!("Creating compressed archive at path: {}", archive_path);

    let tar_file = fs::File::create(&archive_path)?;
    let enc = XzEncoder::new(tar_file, settings.compression_level);
    let mut tar = Builder::new(enc);

    println!("Adding contents of {} to archive", settings.backup_folder.display());
    
    let walker = WalkDir::new(&settings.backup_folder)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok());
        
    for entry in walker {
        let path = entry.path();
        
        // Process the path only if it's different from the backup folder
        if path == settings.backup_folder {
            continue;
        }
        
        let file_name = path.file_name().unwrap_or_default().to_string_lossy();
        
        if settings.exclude_hidden && file_name.starts_with('.') {
            println!("Excluding hidden file or directory: {}", path.display());
            continue;
        }
        
        if settings.exclude_paths.iter().any(|exclude| path.starts_with(exclude)) {
            println!("Excluding {}", path.display());
            continue;
        }
        
        let relative_path = path.strip_prefix(&settings.backup_folder)
            .unwrap_or_else(|_| path);
            
        if path.is_file() {
            println!("Adding file {} to archive", path.display());
            match tar.append_path_with_name(path, relative_path) {
                Ok(_) => {},
                Err(e) => {
                    println!("Warning: Error adding file {}: {}", path.display(), e);
                }
            }
        } else if path.is_dir() {
            println!("Adding directory {} to archive", path.display());
            match tar.append_dir(relative_path, path) {
                Ok(_) => {},
                Err(e) => {
                    println!("Warning: Error adding directory {}: {}", path.display(), e);
                }
            }
        }
    }

    tar.finish().map_err(|e| {
        println!("Error finishing the archive: {}", e);
        e
    })?;
    println!("Archive created and compressed successfully.");

    let archive_size = fs::metadata(&archive_path)?.len();
    println!("Compressed archive size: {} Bytes", archive_size);

    let encrypted_archive_path = format!("{}.enc", archive_path);
    println!("Encrypting the archive...");
    encrypt_data(&archive_path, &encrypted_archive_path, settings)?;
    println!("Encryption completed: {}", encrypted_archive_path);

    manage_backup_limit(settings)?;

    println!("Uploading the encrypted archive...");
    upload_file(&encrypted_archive_path, settings)?;
    println!("Upload completed.");

    println!("Removing temporary files...");
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
