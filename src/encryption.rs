use crate::config::Settings;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;
use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, CHACHA20_POLY1305};

const CHUNK_SIZE: usize = 10 * 1024 * 1024;

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

    let mut input_file = File::open(archive_path)?;
    let mut output_file = File::create(encrypted_archive_path)?;

    let mut buffer = vec![0; CHUNK_SIZE];
    let mut counter: u64 = 0;

    while let Ok(bytes_read) = input_file.read(&mut buffer) {
        if bytes_read == 0 {
            break;
        }

        let mut chunk = buffer[..bytes_read].to_vec();
        
        let nonce_bytes = counter.to_le_bytes();
        let nonce = Nonce::assume_unique_for_key([
            nonce_bytes[0],
            nonce_bytes[1],
            nonce_bytes[2],
            nonce_bytes[3],
            nonce_bytes[4],
            nonce_bytes[5],
            nonce_bytes[6],
            nonce_bytes[7],
            0, 0, 0, 0,
        ]);

        less_safe_key
            .seal_in_place_append_tag(nonce, Aad::empty(), &mut chunk)
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "Encryption failed"))?;

        output_file.write_all(&chunk)?;
        counter += 1;
    }

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
