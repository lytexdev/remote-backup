use crate::config::Settings;
use std::fs;
use std::io;
use std::path::Path;
use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, CHACHA20_POLY1305};

fn load_key(key_path: &Path) -> io::Result<Vec<u8>> {
    println!("Loading encryption key from: {}", key_path.display());
    fs::read(key_path).map_err(|e| {
        println!("Error loading key: {:?}", e);
        e
    })
}

pub fn encrypt_data(archive_path: &str, encrypted_archive_path: &str, settings: &Settings) -> io::Result<()> {
    let key = load_key(&settings.encryption_key_path)?;
    if key.len() != 32 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Encryption key must be 32 bytes long"));
    }

    let unbound_key = UnboundKey::new(&CHACHA20_POLY1305, &key)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to create UnboundKey"))?;
    let less_safe_key = LessSafeKey::new(unbound_key);

    let archive_data = fs::read(archive_path)?;
    let nonce = Nonce::assume_unique_for_key([0u8; 12]);

    let mut encrypted_data = archive_data.clone();
    less_safe_key
        .seal_in_place_append_tag(nonce, Aad::empty(), &mut encrypted_data)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Encryption failed"))?;

    fs::write(encrypted_archive_path, encrypted_data)?;

    println!("Archive successfully encrypted and saved as {}", encrypted_archive_path);

    Ok(())
}

pub fn decrypt_data(encrypted_data: &[u8], settings: &Settings) -> io::Result<Vec<u8>> {
    let key = load_key(&settings.encryption_key_path)?;
    if key.len() != 32 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Encryption key must be 32 bytes long"));
    }

    let unbound_key = UnboundKey::new(&CHACHA20_POLY1305, &key)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to create UnboundKey"))?;
    let less_safe_key = LessSafeKey::new(unbound_key);

    let nonce = Nonce::assume_unique_for_key([0u8; 12]);
    let mut encrypted_data = encrypted_data.to_vec();
    let decrypted_data = less_safe_key
        .open_in_place(nonce, Aad::empty(), &mut encrypted_data)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Decryption failed"))?;

    Ok(decrypted_data.to_vec())
}
