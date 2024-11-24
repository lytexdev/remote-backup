use crate::config::Settings;
use crate::encryption::decrypt_data_blockwise;
use ssh2::Session;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::path::Path;

const UPLOAD_CHUNK_SIZE: usize = 10 * 1024 * 1024;

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

pub fn upload_file(file_path: &str, settings: &Settings) -> io::Result<()> {
    println!("Starting upload process for archive: {}", file_path);

    let sess = setup_ssh_session(settings)?;
    let sftp = sess.sftp()?;

    let file_name = Path::new(file_path)
        .file_name()
        .expect("Failed to extract file name");

    let remote_path = format!("{}/{}", settings.remote_backup_dir.display(), file_name.to_string_lossy());
    println!("Remote path for upload: {}", remote_path);

    let mut remote_file = sftp.create(Path::new(&remote_path)).map_err(|e| {
        println!("Failed to create remote file: {:?}", e);
        io::Error::new(io::ErrorKind::Other, "Failed to create remote file")
    })?;

    let mut local_file = File::open(file_path)?;
    let mut buffer = vec![0; UPLOAD_CHUNK_SIZE];

    while let Ok(bytes_read) = local_file.read(&mut buffer) {
        if bytes_read == 0 {
            break;
        }

        remote_file.write_all(&buffer[..bytes_read]).map_err(|e| {
            println!("Failed to upload file chunk: {:?}", e);
            io::Error::new(io::ErrorKind::Other, "Failed to upload file chunk")
        })?;
    }

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

    let local_encrypted_path = format!("{}/{}", settings.restore_path.display(), filename);
    let mut encrypted_file = File::create(&local_encrypted_path)?;

    let mut buffer = vec![0; UPLOAD_CHUNK_SIZE];
    while let Ok(bytes_read) = remote_file.read(&mut buffer) {
        if bytes_read == 0 {
            break;
        }

        encrypted_file.write_all(&buffer[..bytes_read])?;
    }

    println!("File downloaded successfully. Decrypting...");

    let local_decrypted_path = local_encrypted_path.trim_end_matches(".enc").to_string();
    decrypt_data_blockwise(
        Path::new(&local_encrypted_path),
        Path::new(&local_decrypted_path),
        settings,
    )?;

    fs::remove_file(&local_encrypted_path)?;
    println!(
        "Backup {} successfully decrypted and saved as {}.",
        filename, local_decrypted_path
    );

    Ok(())
}
