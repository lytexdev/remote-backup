use crate::config::Settings;
use crate::encryption::decrypt_data;
use ssh2::Session;
use std::fs;
use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::path::Path;

pub fn setup_ssh_session(settings: &Settings) -> Result<Session, io::Error> {
    let tcp = TcpStream::connect((settings.ssh_host.as_str(), settings.ssh_port))?;
    let mut sess = Session::new()?;
    sess.set_tcp_stream(tcp);
    sess.handshake()?;

    sess.userauth_pubkey_file(
        &settings.ssh_user,
        None,
        Path::new(&settings.identity_file),
        None,
    )?;

    if !sess.authenticated() {
        return Err(io::Error::new(io::ErrorKind::Other, "SSH authentication failed"));
    }

    Ok(sess)
}

pub fn upload_file(archive_path: &str, archive_data: &[u8], settings: &Settings) -> io::Result<()> {
    println!("Starting upload process for archive: {}", archive_path);

    let sess = setup_ssh_session(settings)?;
    let sftp = sess.sftp()?;

    let archive_filename = Path::new(archive_path)
        .file_name()
        .expect("Failed to extract archive filename");

    let remote_path = format!("{}/{}", settings.remote_backup_dir.display(), archive_filename.to_string_lossy());
    println!("Remote path for upload: {}", remote_path);

    let mut remote_file = sftp.create(Path::new(&remote_path)).map_err(|e| {
        println!("Failed to create remote file: {:?}", e);
        io::Error::new(io::ErrorKind::Other, "Failed to create remote file")
    })?;

    remote_file.write_all(archive_data).map_err(|e| {
        println!("Failed to upload file data: {:?}", e);
        io::Error::new(io::ErrorKind::Other, "Failed to upload file data")
    })?;

    println!("Archive successfully uploaded.");
    Ok(())
}

pub fn list_backups(settings: &Settings) -> io::Result<Vec<String>> {
    let sess = setup_ssh_session(settings)?;
    let sftp = sess.sftp()?;
    let remote_dir = Path::new(&settings.remote_backup_dir);

    let entries = sftp.readdir(remote_dir)?;
    let mut backups = vec![];

    println!("Available backups in remote directory:");
    for (path, _) in entries {
        if let Some(filename) = path.file_name() {
            let filename = filename.to_string_lossy().into_owned();
            println!("{}", filename);
            backups.push(filename);
        }
    }

    Ok(backups)
}

pub fn delete_backup(filename: &str, settings: &Settings) -> io::Result<()> {
    let sess = setup_ssh_session(settings)?;
    let sftp = sess.sftp()?;

    let remote_file_path = format!("{}/{}", settings.remote_backup_dir.display(), filename);
    sftp.unlink(Path::new(&remote_file_path))?;
    
    println!("Deleted backup: {}", filename);
    Ok(())
}

pub fn restore_backup(filename: &str, settings: &Settings) -> io::Result<()> {
    let sess = setup_ssh_session(settings)?;
    let sftp = sess.sftp()?;

    let remote_file_path = format!("{}/{}", settings.remote_backup_dir.display(), filename);
    println!("Downloading file: {}", remote_file_path);

    let mut remote_file = sftp.open(Path::new(&remote_file_path)).map_err(|e| {
        println!("Failed to access remote file: {:?}", e);
        io::Error::new(io::ErrorKind::NotFound, "Remote file not found")
    })?;

    let mut encrypted_data = vec![];
    remote_file.read_to_end(&mut encrypted_data)?;

    let decrypted_data = decrypt_data(&encrypted_data, settings)?;

    let local_filename = filename.trim_end_matches(".enc");
    let local_path = format!("{}/{}", settings.restore_path.display(), local_filename);
    fs::write(&local_path, decrypted_data)?;

    println!("Backup {} successfully downloaded, decrypted, and saved as {}.", filename, local_path);
    Ok(())
}
