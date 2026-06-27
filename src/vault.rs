use crate::models::Vault;
use crate::config::Config;
use argon2::Argon2;
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Nonce,
};
use rand::RngCore;
use std::fs;

// Deriva la clave de cifrado desde password + keyfile
pub fn derive_key(password: &str, keyfile_bytes: &[u8]) -> [u8; 32] {
    let mut combined = password.as_bytes().to_vec();
    combined.extend_from_slice(keyfile_bytes);

    let salt = b"erebor-vault-salt-2026";
    let mut key = [0u8; 32];
    Argon2::default()
        .hash_password_into(&combined, salt, &mut key)
        .expect("Error derivando clave");
    key
}

// Cifra los datos del vault
pub fn encrypt(data: &[u8], key: &[u8; 32]) -> Vec<u8> {
    let cipher = ChaCha20Poly1305::new(key.into());
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let encrypted = cipher
        .encrypt(nonce, data)
        .expect("Error cifrando");

    // Guardamos nonce + datos cifrados juntos
    let mut result = nonce_bytes.to_vec();
    result.extend_from_slice(&encrypted);
    result
}

// Descifra los datos del vault
pub fn decrypt(data: &[u8], key: &[u8; 32]) -> Vec<u8> {
    let cipher = ChaCha20Poly1305::new(key.into());
    let nonce = Nonce::from_slice(&data[..12]);
    let encrypted = &data[12..];

    cipher
        .decrypt(nonce, encrypted)
        .expect("Error descifrando — password o keyfile incorrecto")
}

// Carga el vault desde disco
pub fn load_vault(config: &Config, key: &[u8; 32]) -> Vault {
    if !config.vault_path.exists() {
        return Vault::new();
    }
    let encrypted = fs::read(&config.vault_path)
        .expect("Error leyendo vault");
    let decrypted = decrypt(&encrypted, key);
    serde_json::from_slice(&decrypted)
        .expect("Error parseando vault")
}

// Guarda el vault en disco
pub fn save_vault(vault: &Vault, config: &Config, key: &[u8; 32]) {
    let json = serde_json::to_vec(vault)
        .expect("Error serializando vault");
    let encrypted = encrypt(&json, key);
    fs::create_dir_all(
        config.vault_path.parent().unwrap()
    ).expect("Error creando directorio");
    fs::write(&config.vault_path, encrypted)
        .expect("Error escribiendo vault");
}
