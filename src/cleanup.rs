use crate::config::Settings;
use ssh2::Session;
use std::net::TcpStream;
use std::path::Path;
use std::io;

pub fn cleanup_old_backups(settings: &Settings) -> io::Result<()> {
    let tcp = TcpStream::connect((&settings.ssh_host[..], settings.ssh_port))?;
    let mut sess = Session::new().unwrap();
    
    sess.set_tcp_stream(tcp);
    sess.handshake()?;
    sess.userauth_agent(&settings.ssh_user)?;

    let sftp = sess.sftp()?;
    let mut entries: Vec<_> = sftp
        .readdir(Path::new(&settings.remote_backup_dir))?
        .into_iter()
        .collect();

    entries.sort_by_key(|&(_, ref stat)| stat.mtime.unwrap_or(0));
    while entries.len() > settings.max_backups {
        if let Some((oldest_backup, _)) = entries.first() {
            sftp.unlink(&oldest_backup)?;
            entries.remove(0);
        }
    }
    Ok(())
}
