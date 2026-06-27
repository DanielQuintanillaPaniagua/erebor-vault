use std::path::PathBuf;

pub struct Config {
    pub vault_path: PathBuf,
    pub key_path: PathBuf,
}

impl Config {
    pub fn default() -> Self {
        let home = dirs::home_dir()
            .expect("No se pudo encontrar el directorio home");

        Config {
            vault_path: home.join(".erebor").join("vault.enc"),
            key_path: home.join(".erebor").join("erebor.key"),
        }
    }
}
