use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Entry {
    pub username: String,
    pub password: String,
    pub notes: String,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Vault {
    pub version: u32,
    pub entries: std::collections::HashMap<String, Entry>,
}

impl Vault {
    pub fn new() -> Self {
        Vault {
            version: 1,
            entries: std::collections::HashMap::new(),
        }
    }
}
