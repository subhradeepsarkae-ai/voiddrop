use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use anyhow::Result;
use rand::RngCore;
use sha2::{Digest, Sha256};

pub fn derive_key(code: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(code.as_bytes());
    let result = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&result);
    key
}

pub fn encrypt_chunk(key: &[u8; 32], plaintext: &[u8]) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| anyhow::anyhow!("encryption failed: {:?}", e))?;
    let mut frame = Vec::with_capacity(12 + ciphertext.len());
    frame.extend_from_slice(&nonce_bytes);
    frame.extend_from_slice(&ciphertext);
    Ok(frame)
}

pub fn decrypt_chunk(key: &[u8; 32], frame: &[u8]) -> Result<Vec<u8>> {
    if frame.len() < 12 {
        anyhow::bail!("invalid encrypted frame");
    }
    let (nonce_bytes, ciphertext) = frame.split_at(12);
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::from_slice(nonce_bytes);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| anyhow::anyhow!("decryption failed: {:?}", e))?;
    Ok(plaintext)
}
