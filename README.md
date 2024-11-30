# Remote Backup

## Overview
A Rust-based command-line application for creating, encrypting, and uploading backups to a remote server over SSH. This tool supports compression, encryption, and configurable backup retention policies.

## Installation

### Prerequisites
- Rust
- Cargo

**Clone the repository**
```bash
git clone https://github.com/lytexdev/remote-backup.git
cd remote-backup
```

**Edit the `config.toml` configuration file**
1. Copy and rename `config.example.toml` to `config.toml`
2. Edit the `config.toml` file with the appropriate paths, encryption key path, and server information

**Build the application**
```bash
cargo build --release
```
The binary will be available in `./target/release/remote-backup`.

## Configuration
Configure `config.toml` as follows:

```toml
[backup]
backup_folder = "/path/to/folder"           # Folder to backup
exclude_paths = ["/path/to/folder/cache"]   # Paths to exclude from backup
exclude_hidden = false                      # Exclude hidden files and directories (files/directories starting with .)
remote_backup_dir = "/path/to/remote/dir"   # Directory on the remote server
restore_path = "/path/to/restore"           # Directory to restore files locally
compression_level = 9                       # Compression level (1-9)
max_backups = 3                             # Maximum number of backups to retain
tmp_path = "/path/to/tmp"                   # Temporary file path for local backup

[ssh]
host = "127.0.0.1"                          # Remote server hostname or IP
port = 22                                   # SSH port
user = "your-username"                      # SSH user
identity_file = "/path/to/ssh/key"          # Path to SSH private key file

[encryption]
key_path = "/path/to/encryption/key"        # Path were the encryption key will be generated and stored
```

## Key Generation
Before running backups, generate a 32-byte encryption key file in the configured `key_path`:
```bash
./target/release/remote-backup generate-key
```
This will create a binary encryption key file at the specified `key_path`, used to securely encrypt and decrypt backups

## Usage

### Running a Backup
To create, encrypt, and upload a backup to the remote server:
```bash
./target/release/remote-backup backup
```

### Listing Backups
To list all backups stored on the remote server:
```bash
./target/release/remote-backup list
```

### Restoring a Backup
To download and decrypt a specific backup:
```bash
./target/release/remote-backup restore <backup-file-name>
```

### Automatic backups
Upload backups to the remote server every 24 hours with cron:
```bash
crontab -e
```
Add the following line to your crontab:
```bash
0 0 * * * /home/user/remote-backup/target/release/remote-backup -c /home/user/remote-backup/config.toml backup >> /var/log/remote_backup.log 2>&1
```

#### Automatic backup log
The log file can be found at `/var/log/remote_backup.log`

## Features
- **Compression**: Backups are compressed using XZ for storage efficiency
- **Encryption**: Ensures secure backup storage with encryption key specified in `config.toml`
- **Configurable Paths**: Custom paths for backup, restoration, and temporary files
- **Retention Policy**: Automatically deletes the oldest backup when the limit is reached

## License
This project is licensed under the GNU General Public License v2. See the [LICENSE](LICENSE) file for details.
